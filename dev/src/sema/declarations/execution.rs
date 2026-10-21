//! Demand const function typing through the existing resumable declaration worklist.
use super::{Checker, DefId, Error, FileId, Work};
use crate::{
    sema::{bodies, values::execution::Requirement},
    types::Kind,
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct Pending {
    active: HashSet<DefId>,
    suspended: HashMap<DefId, Box<bodies::State>>,
}

impl Checker<'_> {
    pub(super) fn execution_body(
        &mut self,
        requirement: Requirement,
        resume: bool,
    ) -> Result<(), Error> {
        let definition = match requirement {
            Requirement::FunctionBody(instance) => {
                let Some(Kind::FunctionItem { item, .. }) = self.facts.types.kind(instance.ty())
                else {
                    return Err(Error::at(
                        FileId(0),
                        0,
                        "constant body requirement lost function identity",
                    ));
                };
                DefId(item.0)
            }
            Requirement::InitializerBody(definition) => {
                let source = self.report.resolution.definitions[definition.0 as usize]
                    .node
                    .ok_or_else(|| {
                        Error::at(FileId(0), 0, "missing constant declaration source")
                    })?;
                return self.require_initializer_readiness(source, definition);
            }
            Requirement::AssociatedBody(instance) => {
                let source = self.report.resolution.definitions[instance.definition.0 as usize]
                    .node
                    .ok_or_else(|| Error::at(FileId(0), 0, "missing associated constant source"))?;
                return self.require_initializer_readiness(source, instance.definition);
            }
        };
        if self.facts.typed_function_bodies.contains(&definition) {
            return Ok(());
        }
        if !resume && !self.execution_bodies.active.insert(definition) {
            return Err(Error::at(
                FileId(0),
                0,
                "cycle in const function body declaration dependencies",
            ));
        }
        let mut scratch = self.constant_scratch.take_typing();
        scratch.resumable();
        let mut checker = bodies::Checker::with_scratch(self.report, &mut self.facts, scratch);
        checker.availability = crate::sema::traits::implementations::Availability::Declarations;
        let progress = if resume {
            let state = self
                .execution_bodies
                .suspended
                .remove(&definition)
                .ok_or_else(|| Error::at(FileId(0), 0, "missing suspended const function body"))?;
            checker.restore_body(*state);
            checker.advance_body()?
        } else {
            checker.start_execution_function(definition)?
        };
        if progress.is_none() {
            checker.complete_constant_patterns()?;
        }
        let (scratch, state) = checker.into_storage();
        self.constant_scratch.retain_typing(scratch);
        if let Some(dependency) = progress {
            let state = state.ok_or_else(|| {
                Error::at(FileId(0), 0, "const body dependency has no continuation")
            })?;
            self.execution_bodies.suspended.insert(definition, state);
            self.work.push(Work::Execution(requirement, true));
            self.work
                .push(Work::body_dependency(dependency.requirement));
        } else {
            self.execution_bodies.active.remove(&definition);
            self.facts.typed_function_bodies.insert(definition);
        }
        Ok(())
    }
}
