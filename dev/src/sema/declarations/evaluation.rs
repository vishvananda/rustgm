//! Const arguments suspend only for the selected initializer's declaration dependencies.
use super::{Checker, DefId, Error, NodeKey, TypeId, Work, regions};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    sema::{arguments, constants},
    types::arguments::{ConstId, Constant},
};

#[derive(Clone, Copy)]
pub(super) struct Request {
    key: NodeKey,
    ty: TypeId,
    owner: DefId,
    array_length: bool,
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
}

impl Checker<'_> {
    pub(super) fn schedule_typed_constant(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        context: regions::Context,
    ) -> Result<(), Error> {
        if self.constant_evaluations.completed.contains_key(&(key, ty)) {
            return Ok(());
        }
        let owner = context
            .binder
            .and_then(|binder| self.regions.owner(binder))
            .unwrap_or(DefId(0));
        self.work.push(Work::EvaluateConstant(
            Request {
                key,
                ty,
                owner,
                array_length: false,
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
        self.work.push(Work::EvaluateConstant(
            Request {
                key,
                ty,
                owner,
                array_length: false,
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
        let owner = context
            .binder
            .and_then(|binder| self.regions.owner(binder))
            .unwrap_or(DefId(0));
        self.work.push(Work::EvaluateConstant(
            Request {
                key,
                ty,
                owner,
                array_length: true,
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
            owner,
            array_length,
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
                Some(self.constant_module(key, owner)?)
            };
            let input = arguments::Constants {
                report: self.report,
                facts: &mut self.facts,
                captures: &mut self.constant_captures,
                scratch: &mut self.constant_scratch,
                discriminants: &mut self.discriminants,
            }
            .input(key, ty, module)?;
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
            constants::ArgumentProgress::Complete(bits) => {
                let id = self
                    .facts
                    .types
                    .intern_constant(Constant { ty, bits })
                    .map_err(|e| self.error(key, e))?;
                self.constant_evaluations.completed.insert((key, ty), id);
                if array_length {
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
