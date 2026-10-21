//! Caller constraints settle before an inline result is published under its own body.
use super::{Checker, Error, InputRefresh, Work};
use crate::{
    sema::{
        bodies::{Dependency, closures::Scheduler, finalization::Finalization},
        traits::selection::Source,
    },
    types::infer::Solved,
};
use std::collections::HashMap;

impl Checker<'_> {
    pub(in crate::sema::bodies) fn finish_inline_child(
        &mut self,
        index: usize,
        evidence: HashMap<usize, Source>,
    ) -> Result<Option<Dependency>, Error> {
        let body = self.state.body.node;
        let mut scheduler = self
            .state
            .closures
            .take()
            .ok_or_else(|| self.error(body, "missing inline completion scheduler"))?;
        let entry = &mut scheduler.entries[index];
        let metadata = entry
            .payload
            .inline_mut()
            .map_err(|e| self.error(entry.source, e))?;
        let mut child = entry
            .state
            .take()
            .ok_or_else(|| self.error(body, "missing collected inline body"))?;
        std::mem::swap(&mut entry.inputs.context, &mut child.infer);
        let pairs = entry
            .inputs
            .incoming_regions(
                &mut self.facts.types,
                &mut self.state.infer,
                &mut metadata.lifetimes.opening,
            )
            .map_err(|e| self.error(entry.source, e))?;
        std::mem::swap(&mut entry.inputs.context, &mut child.infer);
        child.body_binders = metadata
            .lifetimes
            .count()
            .map_err(|e| self.error(entry.source, e))?;
        let source = entry.source;
        let result = metadata.result;
        self.state.finalization = Some(Box::new(Finalization::Closures(evidence)));
        let parent = std::mem::replace(&mut self.state, *child);
        scheduler.stack.push((scheduler.active, Box::new(parent)));
        scheduler.active = Some(index);
        self.state.closures = Some(scheduler);
        self.defer_normalization(source, crate::types::lifetime_proof::ScopeId::ROOT, result)?;
        self.state
            .work
            .push(Work::BodyInputs(Box::new(InputRefresh { source, pairs })));
        Ok(None)
    }

    pub(in crate::sema::bodies) fn export_inline_inputs(
        &mut self,
        scheduler: &mut Scheduler,
    ) -> Result<(), Error> {
        let index = scheduler
            .active
            .ok_or_else(|| self.error(self.state.body.node, "missing active inline result"))?;
        let (_, parent) = scheduler
            .stack
            .last_mut()
            .ok_or_else(|| self.error(self.state.body.node, "missing inline inference parent"))?;
        let entry = &mut scheduler.entries[index];
        let metadata = entry
            .payload
            .inline_mut()
            .map_err(|e| self.error(entry.source, e))?;
        self.state
            .infer
            .default_numbers()
            .map_err(|e| self.error(entry.source, e))?;
        std::mem::swap(&mut entry.inputs.context, &mut self.state.infer);
        let mut closing = metadata
            .lifetimes
            .closing(&self.state.call_types, false)
            .map_err(|e| self.error(entry.source, e))?;
        let pairs = entry
            .inputs
            .outgoing_regions(&mut self.facts.types, &mut parent.infer, &mut closing)
            .map_err(|e| self.error(entry.source, e))?;
        std::mem::swap(&mut entry.inputs.context, &mut self.state.infer);
        parent.work.push(Work::BodyInputs(Box::new(InputRefresh {
            source: entry.source,
            pairs,
        })));
        Ok(())
    }

    pub(in crate::sema::bodies) fn publish_inline_body(
        &mut self,
        solved: &Solved,
    ) -> Result<(), Error> {
        let scheduler = self.state.closures.as_ref().ok_or_else(|| {
            self.error(self.state.body.node, "missing inline publication scheduler")
        })?;
        let index = scheduler
            .active
            .ok_or_else(|| self.error(self.state.body.node, "missing inline publication entry"))?;
        let entry = &scheduler.entries[index];
        let metadata = entry
            .payload
            .inline()
            .map_err(|e| self.error(entry.source, e))?;
        let result = *solved
            .types
            .get(metadata.result.0 as usize)
            .ok_or_else(|| self.error(entry.source, "missing solved inline result"))?;
        self.state.result = result;
        self.facts
            .inline_constants
            .as_mut()
            .ok_or_else(|| {
                super::super::Declarations::error(
                    self.report,
                    self.state.body,
                    "missing inline registry",
                )
            })?
            .complete(metadata.id, result)
            .map_err(|e| super::super::Declarations::error(self.report, self.state.body, e))?;
        self.facts
            .anonymous_constants
            .insert(self.state.body, result);
        Ok(())
    }
}
