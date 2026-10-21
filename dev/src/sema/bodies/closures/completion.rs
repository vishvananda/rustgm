//! Finalize nested bodies iteratively after caller constraints determine their inputs.
use super::{Checker, Error, InputRefresh, Scheduler, Work};
use crate::{
    sema::{
        bodies::{Dependency, finalization::Finalization},
        closures::Body,
        traits::selection::Source,
    },
    types::{Kind, infer::Solved},
};
use std::collections::HashMap;

impl Checker<'_> {
    pub(in crate::sema::bodies) fn finish_closure_child(
        &mut self,
        index: usize,
        evidence: HashMap<usize, Source>,
    ) -> Result<Option<Dependency>, Error> {
        let body = self.state.body.node;
        let mut scheduler = self
            .state
            .closures
            .take()
            .ok_or_else(|| self.error(body, "missing closure completion scheduler"))?;
        let entry = &mut scheduler.entries[index];
        let metadata = entry
            .payload
            .closure_mut()
            .map_err(|e| self.error(entry.source, e))?;
        let mut child = entry
            .state
            .take()
            .ok_or_else(|| self.error(body, "missing collected closure body"))?;
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
        let signature = metadata.signature;
        self.state.finalization = Some(Box::new(Finalization::Closures(evidence)));
        let parent = std::mem::replace(&mut self.state, *child);
        scheduler.stack.push((scheduler.active, Box::new(parent)));
        scheduler.active = Some(index);
        self.state.closures = Some(scheduler);
        self.defer_normalization(
            source,
            crate::types::lifetime_proof::ScopeId::ROOT,
            signature,
        )?;
        self.state
            .work
            .push(Work::BodyInputs(Box::new(InputRefresh { source, pairs })));
        Ok(None)
    }

    pub(in crate::sema::bodies) fn export_closure_inputs(
        &mut self,
        scheduler: &mut Scheduler,
    ) -> Result<(), Error> {
        let Some(index) = scheduler.active else {
            return Ok(());
        };
        let body = self.state.body.node;
        let (_, parent) = scheduler
            .stack
            .last_mut()
            .ok_or_else(|| self.error(body, "missing closure inference parent"))?;
        let entry = &mut scheduler.entries[index];
        let metadata = entry
            .payload
            .closure_mut()
            .map_err(|e| self.error(entry.source, e))?;
        self.closure_result_boundary(&metadata.lifetimes)?;
        let mut children = Vec::new();
        let Some(super::Constructor::Function {
            parameters,
            unsafe_,
            convention,
            ..
        }) = self.head(body, metadata.signature)?
        else {
            return Err(self.error(body, "missing closure body signature"));
        };
        for index in 0..=parameters as usize {
            children.push(self.child(body, metadata.signature, index)?);
        }
        metadata.signature = self
            .state
            .infer
            .construct(
                super::Constructor::Function {
                    bound_regions: self.state.body_binders,
                    parameters,
                    unsafe_,
                    convention,
                },
                &children,
            )
            .map_err(|e| self.error(body, e))?;
        let inputs = &mut entry.inputs;
        // Defaults belong to completed body inference, after all caller constraints.
        self.state
            .infer
            .default_numbers()
            .map_err(|e| self.error(body, e))?;
        std::mem::swap(&mut inputs.context, &mut self.state.infer);
        let mut identity = metadata
            .lifetimes
            .closing(&self.state.call_types, false)
            .map_err(|e| self.error(body, e))?;
        let pairs = inputs
            .outgoing_regions(&mut self.facts.types, &mut parent.infer, &mut identity)
            .map_err(|e| self.error(body, e))?;
        parent.work.push(Work::BodyInputs(Box::new(InputRefresh {
            source: entry.source,
            pairs,
        })));
        std::mem::swap(&mut inputs.context, &mut self.state.infer);
        Ok(())
    }

    pub(in crate::sema::bodies) fn publish_closure_body(
        &mut self,
        solved: &Solved,
    ) -> Result<(), Error> {
        let Some(scheduler) = &self.state.closures else {
            return Ok(());
        };
        let Some(index) = scheduler.active else {
            return Ok(());
        };
        let entry = &scheduler.entries[index];
        let metadata = entry
            .payload
            .closure()
            .map_err(|e| self.error(entry.source, e))?;
        let source = entry.source;
        let identity = metadata.identity;
        let parameters = metadata.parameters;
        let inherited = metadata
            .lifetimes
            .inherited()
            .map_err(|e| self.error(source, e))?;
        let signature = solved.types[metadata.signature.0 as usize];
        let signature = self.closure_contract_signature(source, signature, solved)?;
        let Some(Kind::Function { result, .. }) = self.facts.types.kind(signature) else {
            return Err(self.error(source, "missing completed closure signature"));
        };
        self.state.result = result;
        let body = Body {
            identity,
            source: self.key(source),
            body: self.state.body,
            parameters,
            signature,
            owner: self.state.environment_owner,
            universals: self.state.body_binders,
            inherited,
            execution: None,
        };
        self.facts
            .closures
            .insert(body)
            .map_err(|e| self.error(source, e))
    }
}
