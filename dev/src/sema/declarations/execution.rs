//! Demand const body typing through the existing resumable declaration worklist.
use super::{Checker, DefId, Error, FileId, Work};
use crate::{
    sema::{bodies, values::execution::Requirement},
    types::Kind,
};
use std::collections::{HashMap, HashSet};

#[cfg(test)]
#[path = "../../../tests/invariants/provider_execution.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Phase {
    Begin,
    Prepared,
    Resume,
}

#[derive(Default)]
pub(super) struct Pending {
    active: HashSet<DefId>,
    suspended: HashMap<DefId, Box<bodies::State>>,
}

impl Checker<'_> {
    pub(super) fn execution_body(
        &mut self,
        requirement: Requirement,
        phase: Phase,
    ) -> Result<(), Error> {
        let Some((definition, initializer)) = self.execution_owner(requirement, phase)? else {
            return Ok(());
        };
        if initializer.map_or_else(
            || self.facts.typed_function_bodies.contains(&definition),
            |index| self.facts.initializers[index].type_checked(),
        ) {
            return Ok(());
        }
        if phase == Phase::Begin && !self.execution_bodies.active.insert(definition) {
            return Err(Error::at(
                FileId(0),
                0,
                "cycle in const function body declaration dependencies",
            ));
        }
        if phase == Phase::Begin && initializer.is_none() {
            let function = self
                .facts
                .function_indices
                .get(&definition)
                .and_then(|&i| self.facts.functions.get(i))
                .copied()
                .ok_or_else(|| Error::at(FileId(0), 0, "missing demanded function annotations"))?;
            if let Some(body) = function.body {
                self.work
                    .push(Work::Execution(requirement, Phase::Prepared));
                let context = self.local_context(function.node, super::regions::Context::ALIAS);
                self.constant_dependencies_in(body, context)?;
                return Ok(());
            }
        }
        if phase == Phase::Prepared
            && (!self.execution_bodies.active.contains(&definition)
                || self.execution_bodies.suspended.contains_key(&definition))
        {
            return Err(Error::at(
                FileId(0),
                0,
                "prepared const body has no exact pending owner",
            ));
        }
        let mut scratch = self.constant_scratch.take_typing();
        scratch.resumable();
        let mut checker = bodies::Checker::with_scratch(self.report, &mut self.facts, scratch);
        checker.availability = crate::sema::traits::implementations::Availability::Declarations;
        let progress = if phase == Phase::Resume {
            let state = self
                .execution_bodies
                .suspended
                .remove(&definition)
                .ok_or_else(|| Error::at(FileId(0), 0, "missing suspended const function body"))?;
            checker.restore_body(*state);
            checker.advance_body()?
        } else if let Some(index) = initializer {
            checker.start_initializer(index)?
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
            self.work.push(Work::Execution(requirement, Phase::Resume));
            self.work
                .push(Work::body_dependency(dependency.requirement));
        } else {
            self.execution_bodies.active.remove(&definition);
            if initializer.is_none() {
                self.facts.typed_function_bodies.insert(definition);
            }
        }
        Ok(())
    }
    fn execution_owner(
        &mut self,
        requirement: Requirement,
        phase: Phase,
    ) -> Result<Option<(DefId, Option<usize>)>, Error> {
        let owner = match requirement {
            Requirement::DiscriminantValue(_) => {
                return Err(Error::at(
                    FileId(0),
                    0,
                    "enum value readiness must remain in its constant dependency graph",
                ));
            }
            Requirement::AnonymousBody { instance, .. } => {
                return Err(self.error(
                    instance.source(),
                    "anonymous template must complete with its enclosing declaration",
                ));
            }
            Requirement::InlineBody { source, .. } => {
                return Err(self.error(
                    source,
                    "inline constant body must complete with its owning declaration",
                ));
            }
            Requirement::FunctionBody(instance) => {
                let Some(Kind::FunctionItem { item, .. }) = self.facts.types.kind(instance.ty())
                else {
                    return Err(Error::at(
                        FileId(0),
                        0,
                        "constant body requirement lost function identity",
                    ));
                };
                (DefId(item.0), None)
            }
            Requirement::InitializerBody(definition) => {
                let source = self.report.resolution.definitions[definition.0 as usize]
                    .node
                    .ok_or_else(|| {
                        Error::at(FileId(0), 0, "missing constant declaration source")
                    })?;
                if phase == Phase::Begin {
                    if !self.execution_bodies.active.insert(definition) {
                        return Err(self.error(source, "cycle in required initializer bodies"));
                    }
                    self.work
                        .push(Work::Execution(requirement, Phase::Prepared));
                    self.require_initializer_readiness(source, definition)?;
                    return Ok(None);
                }
                let index = self.facts.initializer_index(definition).ok_or_else(|| {
                    self.error(source, "missing required initializer declaration")
                })?;
                (definition, Some(index))
            }
            Requirement::SourceAssociatedBody(source) => {
                let (_, original) = source
                    .identity(&self.facts)
                    .map_err(|e| Error::at(FileId(0), 0, e))?;
                let result = source
                    .result(&self.facts)
                    .map_err(|e| Error::at(FileId(0), 0, e))?;
                let selected = source
                    .resolve(&mut self.facts, result)
                    .map_err(|e| self.error(original, e))?;
                self.require_initializer_readiness(original, selected.selected.definition)?;
                return Ok(None);
            }
            Requirement::AssociatedBody { instance, .. } => {
                let source = self.report.resolution.definitions[instance.definition.0 as usize]
                    .node
                    .ok_or_else(|| Error::at(FileId(0), 0, "missing associated constant source"))?;
                self.require_initializer_readiness(source, instance.definition)?;
                return Ok(None);
            }
        };
        Ok(Some(owner))
    }
}
