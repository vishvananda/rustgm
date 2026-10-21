//! Finalize nested bodies iteratively after caller constraints determine their inputs.
use super::{Argument, Checker, Error, InputRefresh, Scheduler, Work};
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
    pub(in crate::sema::bodies) fn finish_closures(
        &mut self,
        evidence: HashMap<usize, Source>,
    ) -> Result<Option<Dependency>, Error> {
        let Some(&index) = self.state.closure_children.get(self.state.closure_cursor) else {
            return self.finish_inference(evidence);
        };
        self.state.closure_cursor += 1;
        let body = self.state.body.node;
        let mut scheduler = self
            .state
            .closures
            .take()
            .ok_or_else(|| self.error(body, "missing closure completion scheduler"))?;
        let entry = &mut scheduler.entries[index];
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
                &mut entry.payload.lifetimes.opening,
            )
            .map_err(|e| self.error(entry.source, e))?;
        std::mem::swap(&mut entry.inputs.context, &mut child.infer);
        child.body_binders = entry
            .payload
            .lifetimes
            .count()
            .map_err(|e| self.error(entry.source, e))?;
        let source = entry.source;
        let signature = entry.payload.signature;
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
            .push(Work::ClosureInputs(Box::new(InputRefresh {
                source,
                pairs,
            })));
        Ok(None)
    }

    /// Returned interface equations precede any paused finalization frontier.
    pub(in crate::sema::bodies) fn resume_closure_inputs(
        &mut self,
    ) -> Result<Option<Dependency>, Error> {
        while matches!(self.state.work.last(), Some(Work::ClosureInputs(_))) {
            let Some(Work::ClosureInputs(inputs)) = self.state.work.pop() else {
                return Err(self.error(self.state.body.node, "missing closure interface work"));
            };
            if let Some(dependency) = self.finish_closure_inputs(*inputs)? {
                return Ok(Some(dependency));
            }
        }
        Ok(None)
    }

    pub(in crate::sema::bodies) fn finish_closure_inputs(
        &mut self,
        inputs: InputRefresh,
    ) -> Result<Option<Dependency>, Error> {
        for &(left, right) in &inputs.pairs {
            for argument in [left, right] {
                if let Argument::Type(term) = argument {
                    self.defer_normalization(
                        inputs.source,
                        crate::types::lifetime_proof::ScopeId::ROOT,
                        term,
                    )?;
                }
            }
        }
        if let super::super::projections::Advance::Definition(dependency) =
            self.poll_projections(inputs.source)?
        {
            self.state.work.push(Work::ClosureInputs(Box::new(inputs)));
            return Ok(Some(dependency));
        }
        self.state
            .infer
            .equate_input_pairs(&self.facts.types, &inputs.pairs)
            .map_err(|e| self.error(inputs.source, e))?;
        Ok(None)
    }

    pub(in crate::sema::bodies) fn continue_closure_bodies(&mut self) -> Result<bool, Error> {
        let Some(mut scheduler) = self.state.closures.take() else {
            return Ok(false);
        };
        if self.pending_body() {
            self.state.closures = Some(scheduler);
            return Ok(true);
        }
        let Some(index) = scheduler.active else {
            self.state.closure_children.clear();
            self.state.closure_cursor = 0;
            return Ok(false);
        };
        let source = scheduler.entries[index].source;
        let (active, parent) = scheduler
            .stack
            .pop()
            .ok_or_else(|| self.error(self.state.body.node, "missing closure completion parent"))?;
        self.state = *parent;
        scheduler.active = active;
        self.state.closures = Some(scheduler);
        self.defer_normalization(
            source,
            crate::types::lifetime_proof::ScopeId::ROOT,
            self.term(source)?,
        )?;
        Ok(true)
    }

    pub(in crate::sema::bodies) fn publish_closure_progress(&mut self) -> Result<(), Error> {
        let Some(mut scheduler) = self.state.closures.take() else {
            return Ok(());
        };
        self.export_closure_inputs(&mut scheduler)?;
        self.state.closures = Some(scheduler);
        Ok(())
    }
    fn export_closure_inputs(&mut self, scheduler: &mut Scheduler) -> Result<(), Error> {
        let Some(index) = scheduler.active else {
            return Ok(());
        };
        let body = self.state.body.node;
        let (_, parent) = scheduler
            .stack
            .last_mut()
            .ok_or_else(|| self.error(body, "missing closure inference parent"))?;
        let entry = &mut scheduler.entries[index];
        self.closure_result_boundary(&entry.payload.lifetimes)?;
        let mut children = Vec::new();
        let Some(super::Constructor::Function {
            parameters,
            unsafe_,
            convention,
            ..
        }) = self.head(body, entry.payload.signature)?
        else {
            return Err(self.error(body, "missing closure body signature"));
        };
        for index in 0..=parameters as usize {
            children.push(self.child(body, entry.payload.signature, index)?);
        }
        entry.payload.signature = self
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
        let mut identity = entry
            .payload
            .lifetimes
            .closing(&self.state.call_types, false)
            .map_err(|e| self.error(body, e))?;
        let pairs = inputs
            .outgoing_regions(&mut self.facts.types, &mut parent.infer, &mut identity)
            .map_err(|e| self.error(body, e))?;
        parent.work.push(Work::ClosureInputs(Box::new(InputRefresh {
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
        let source = entry.source;
        let identity = entry.payload.identity;
        let parameters = entry.payload.parameters;
        let inherited = entry
            .payload
            .lifetimes
            .inherited()
            .map_err(|e| self.error(source, e))?;
        let signature = solved.types[entry.payload.signature.0 as usize];
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
        };
        self.facts
            .closures
            .insert(body)
            .map_err(|e| self.error(source, e))
    }
}
