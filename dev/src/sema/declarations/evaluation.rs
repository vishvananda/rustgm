//! Const arguments suspend only for the selected initializer's declaration dependencies.
use super::{Checker, DefId, Error, NodeKey, TypeId, Work, regions};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    sema::{arguments, bodies::TypePrivacy, constants},
    types::arguments::ConstId,
};

#[derive(Clone, Copy)]
pub(super) struct Request {
    key: NodeKey,
    ty: TypeId,
    site: Site,
    array_length: bool,
    privacy: TypePrivacy,
}
/// Where the lexical module that types a constant is found.
#[derive(Clone, Copy)]
pub(super) enum Site {
    /// The scope of a declaration: a binder's owner, an item of its own
    /// annotation, or the item whose arguments a constant supplies.
    Declaration(DefId),
    /// The scope that a body, closure or constant block opens.
    Scope(NodeKey),
    /// The root of the crate that loaded the constant's file: the module
    /// that sees least, for a constant whose position names no owner.
    CrateRoot,
}
struct Suspended {
    input: constants::ArgumentInput,
    continuation: Box<constants::ArgumentContinuation>,
}
#[derive(Default)]
pub(super) struct Pending {
    completed: HashMap<(NodeKey, TypeId), ConstId>,
    suspended: HashMap<(NodeKey, TypeId), Suspended>,
    active_initializers: HashSet<DefId>,
    ready_initializers: HashSet<DefId>,
    /// Anonymous constants nested in a body, whose own nested constants are too.
    in_bodies: HashSet<NodeKey>,
    /// The site of each constant without a binder owner, which the constants
    /// nested in it share, and of each binderless const or static initializer.
    sites: HashMap<NodeKey, Site>,
}

#[cfg(test)]
#[path = "../../../tests/invariants/waited_copy_selections.rs"]
mod copy_wait_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/lengths_before_types.rs"]
mod length_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/binderless_constant_modules.rs"]
mod module_tests;

impl Checker<'_> {
    /// Where the lexical module that types the constant at `key` is found: the
    /// binder's owner; the item of a binderless const, static, nominal or
    /// alias annotation; the scope of a binderless body; or the site of the
    /// constant that encloses a nested one.
    pub(super) fn constant_site(&self, key: NodeKey, context: regions::Context) -> Site {
        if let Some(owner) = context.binder.and_then(|binder| self.regions.owner(binder)) {
            return Site::Declaration(owner);
        }
        match context.position {
            regions::Position::Static(item)
            | regions::Position::Nominal(item)
            | regions::Position::Alias(item)
            | regions::Position::Default(item) => Site::Declaration(item),
            regions::Position::Local(body) => {
                let body = NodeKey { node: body, ..key };
                self.constant_evaluations
                    .sites
                    .get(&body)
                    .copied()
                    .unwrap_or(Site::Scope(body))
            }
            regions::Position::Constant(root) => self
                .constant_evaluations
                .sites
                .get(&NodeKey { node: root, ..key })
                .copied()
                .unwrap_or(Site::CrateRoot),
            regions::Position::Input | regions::Position::Impl | regions::Position::Output => {
                Site::CrateRoot
            }
        }
    }
    /// A binderless const or static initializer opens no scope of its own:
    /// the constants in it are typed in the item's lexical module.
    pub(super) fn retain_initializer_site(
        &mut self,
        id: DefId,
        value: Option<crate::syntax::ast::NodeId>,
        context: regions::Context,
        key: NodeKey,
    ) {
        if let Some(node) = value
            && context.binder.is_none()
        {
            self.constant_evaluations
                .sites
                .insert(NodeKey { node, ..key }, Site::Declaration(id));
        }
    }
    /// Keep a binderless constant's site for the constants nested in it.
    pub(super) fn retain_constant_site(&mut self, root: NodeKey, context: regions::Context) {
        if context
            .binder
            .and_then(|binder| self.regions.owner(binder))
            .is_none()
        {
            let site = self.constant_site(root, context);
            self.constant_evaluations.sites.insert(root, site);
        }
    }
    /// A constant nested in a body is type-privacy checked like that body; one
    /// in a signature, discriminant or item annotation is not, nor are the
    /// constants nested in it.
    fn constant_privacy(&mut self, key: NodeKey, context: regions::Context) -> TypePrivacy {
        let in_body = match context.position {
            regions::Position::Local(_) => true,
            regions::Position::Constant(root) => self
                .constant_evaluations
                .in_bodies
                .contains(&NodeKey { node: root, ..key }),
            _ => false,
        };
        if in_body {
            self.constant_evaluations.in_bodies.insert(key);
            TypePrivacy::Checked
        } else {
            TypePrivacy::Signature
        }
    }
    pub(super) fn schedule_typed_constant(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        context: regions::Context,
    ) -> Result<(), Error> {
        if self.constant_evaluations.completed.contains_key(&(key, ty)) {
            return Ok(());
        }
        let site = self.constant_site(key, context);
        let privacy = self.constant_privacy(key, context);
        self.work.push(Work::EvaluateConstant(
            Request {
                key,
                ty,
                site,
                array_length: false,
                privacy,
            },
            false,
        ));
        self.constant_dependencies(key, context)
    }
    pub(super) fn schedule_constant_argument(
        &mut self,
        key: NodeKey,
        parameter: ConstId,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        let ty = self
            .facts
            .types
            .const_type(parameter)
            .ok_or_else(|| self.error(key, "missing const parameter type"))?;
        if self.constant_evaluations.completed.contains_key(&(key, ty)) {
            return Ok(());
        }
        let privacy = self.constant_privacy(key, context);
        self.work.push(Work::EvaluateConstant(
            Request {
                key,
                ty,
                site: Site::Declaration(owner),
                array_length: false,
                privacy,
            },
            false,
        ));
        self.constant_dependencies(key, context)
    }
    pub(super) fn schedule_array_length(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<(), Error> {
        if self.facts.array_lengths.contains_key(&key) {
            return Ok(());
        }
        let ty = TypeId::primitive(crate::types::Primitive::Usize);
        let site = self.constant_site(key, context);
        let privacy = self.constant_privacy(key, context);
        self.work.push(Work::EvaluateConstant(
            Request {
                key,
                ty,
                site,
                array_length: true,
                privacy,
            },
            false,
        ));
        self.constant_dependencies(key, context)
    }

    pub(super) fn generic_constant(&self, key: NodeKey, ty: TypeId) -> Result<ConstId, Error> {
        self.constant_evaluations
            .completed
            .get(&(key, ty))
            .copied()
            .ok_or_else(|| self.error(key, "missing completed const argument"))
    }

    pub(super) fn evaluate_constant(
        &mut self,
        request: Request,
        resume: bool,
    ) -> Result<(), Error> {
        let Request {
            key,
            ty,
            site,
            array_length,
            privacy,
        } = request;
        if self.constant_evaluations.completed.contains_key(&(key, ty)) {
            return Ok(());
        }
        if !resume && self.constant_evaluations.suspended.contains_key(&(key, ty)) {
            return Err(self.error(key, "cycle in const argument declaration dependencies"));
        }
        let (input, continuation) = if resume {
            let pending = self
                .constant_evaluations
                .suspended
                .remove(&(key, ty))
                .ok_or_else(|| self.error(key, "missing suspended const argument"))?;
            (pending.input, Some(pending.continuation))
        } else {
            let module = if self.const_parameter(key).is_some() {
                None
            } else {
                Some(self.constant_module(key, site)?)
            };
            let input = arguments::Constants {
                report: self.report,
                facts: &mut self.facts,
                captures: &mut self.constant_captures,
                scratch: &mut self.constant_scratch,
                discriminants: &mut self.discriminants,
            }
            .input(key, ty, module, privacy)?;
            match input {
                arguments::Input::Parameter(id) => {
                    self.constant_evaluations.completed.insert((key, ty), id);
                    return Ok(());
                }
                arguments::Input::Expression(input) => (input, None),
            }
        };
        let progress = constants::advance_argument(
            self.report,
            &mut self.facts,
            &mut self.constant_scratch,
            &mut self.discriminants,
            input,
            continuation,
            &self.constant_evaluations.ready_initializers,
        )?;
        match progress {
            constants::ArgumentProgress::Complete(id) => {
                self.constant_evaluations.completed.insert((key, ty), id);
                if array_length {
                    let bits = self
                        .facts
                        .types
                        .constant(id)
                        .filter(|value| {
                            value.ty == TypeId::primitive(crate::types::Primitive::Usize)
                        })
                        .ok_or_else(|| {
                            self.error(key, "array length requires its actual usize scalar")
                        })?
                        .bits;
                    let length = u64::try_from(bits)
                        .map_err(|_| self.error(key, "array length overflows usize"))?;
                    self.facts.array_lengths.insert(key, length);
                }
            }
            constants::ArgumentProgress::Prepare(dependency, continuation) => {
                self.constant_evaluations.suspended.insert(
                    (key, ty),
                    Suspended {
                        input,
                        continuation,
                    },
                );
                self.work.push(Work::EvaluateConstant(request, true));
                self.constant_dependency(key, dependency)?;
            }
        }
        Ok(())
    }

    pub(super) fn constant_dependency(
        &mut self,
        key: NodeKey,
        dependency: constants::ArgumentDependency,
    ) -> Result<(), Error> {
        match dependency {
            constants::ArgumentDependency::Body(dependency) => {
                self.work
                    .push(Work::body_dependency(dependency.requirement));
            }
            constants::ArgumentDependency::Initializer(definition) => {
                self.require_initializer_readiness(key, definition)?;
            }
            constants::ArgumentDependency::Members(members) => {
                self.work.extend(
                    members
                        .into_iter()
                        .rev()
                        .map(|request| Work::ConstantMember(request, false)),
                );
            }
            constants::ArgumentDependency::Execution(
                constants::values::execution::Requirement::NominalFields(item),
            ) => self.require_nominal_fields(key, item)?,
            constants::ArgumentDependency::Execution(
                constants::values::execution::Requirement::Selection(wait),
            ) => {
                // A published fact that still fails to resume its constant
                // would otherwise repeat this wait indefinitely.
                if wait.ready(&self.facts) {
                    return Err(self.error(key, "missing selected declaration fact"));
                }
                self.work.push(Work::selection_dependency(wait.0));
            }
            constants::ArgumentDependency::Execution(requirement) => {
                self.work.push(Work::Execution(
                    super::execution::Request::Constant(requirement),
                    super::execution::Phase::Begin,
                ));
            }
        }
        Ok(())
    }
    pub(super) fn complete_discriminants(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let mut continuation = None;
        loop {
            match constants::advance_discriminants(
                self.report,
                &mut self.facts,
                &mut self.discriminants,
                &mut self.constant_scratch,
                continuation,
            )? {
                constants::DiscriminantProgress::Complete => return Ok(()),
                constants::DiscriminantProgress::Prepare {
                    source,
                    dependency,
                    continuation: saved,
                } => {
                    continuation = Some(saved);
                    self.constant_dependency(source, dependency)?;
                    self.drain(steps, limit)?;
                }
            }
        }
    }

    pub(super) fn require_initializer_readiness(
        &mut self,
        key: NodeKey,
        definition: DefId,
    ) -> Result<(), Error> {
        if !self
            .constant_evaluations
            .active_initializers
            .insert(definition)
        {
            return Err(self.error(
                key,
                "cycle in constant initializer declaration dependencies",
            ));
        }
        let (source, context) = self
            .constant_initializer(definition)?
            .ok_or_else(|| self.error(key, "selected constant has no initializer declaration"))?;
        self.work.push(Work::InitializerReady(definition));
        self.constant_dependencies_in(source, context)
    }

    pub(super) fn complete_initializer_readiness(&mut self, definition: DefId) {
        self.constant_evaluations
            .active_initializers
            .remove(&definition);
        self.constant_evaluations
            .ready_initializers
            .insert(definition);
    }
}
