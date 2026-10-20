//! Const arguments suspend only for the selected initializer's declaration dependencies.
use super::{Checker, DefId, Error, NodeKey, TypeId, Work, regions};
use crate::{
    sema::{arguments, constants},
    types::arguments::{ConstId, Constant},
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
pub(super) struct Request {
    key: NodeKey,
    ty: TypeId,
    owner: DefId,
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
        self.work
            .push(Work::EvaluateConstant(Request { key, ty, owner }, false));
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
        let Request { key, ty, owner } = request;
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
                match dependency {
                    constants::ArgumentDependency::Body(dependency) => {
                        self.work.push(match dependency.requirement {
                            crate::sema::bodies::Requirement::Nominal(item) => {
                                Work::ValueType(DefId(item.0))
                            }
                            crate::sema::bodies::Requirement::Definition(definition) => {
                                Work::Definition(definition, false)
                            }
                            crate::sema::bodies::Requirement::ImplHeader(owner) => {
                                Work::ImplHeader(owner, super::impl_headers::Phase::Begin)
                            }
                            crate::sema::bodies::Requirement::Fields(item) => {
                                Work::Fields(item, false)
                            }
                            crate::sema::bodies::Requirement::Opaque(id) => Work::OpaqueHidden(id),
                        });
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
                }
            }
        }
        Ok(())
    }

    fn require_initializer_readiness(
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
