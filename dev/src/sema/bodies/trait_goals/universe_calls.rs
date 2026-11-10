//! Closed universal calls consume their actual scoped evidence without exporting permission.
use super::{Checker, Error, NodeId};
use crate::sema::traits::selection::{Evidence, Source};
use crate::types::{infer, lifetime_proof::Obligations, outlives::Queries, relations::Constraints};

impl Checker<'_> {
    pub(super) fn check_closed_universal_call(
        &mut self,
        node: NodeId,
        index: usize,
        evidence: &Evidence,
    ) -> Result<(), Error> {
        // Builtin, equality and computed language results retain their own
        // scoped/capture duties. This consumer closes ordinary trait calls.
        if !matches!(
            evidence.source,
            Source::Implementation { .. }
                | Source::Assumption(_)
                | Source::QuantifiedAssumption { .. }
        ) {
            return Ok(());
        }
        if self
            .state
            .call_types
            .bound_regions(index)
            .is_none_or(|n| n == 0)
        {
            return Ok(());
        }
        let predicate = match self.state.infer.resolved_predicate(
            &mut self.facts.types,
            &self.state.call_types.obligations[index].1,
        ) {
            Ok(predicate) => predicate,
            Err(infer::Error::Unresolved | infer::Error::Normalization) => return Ok(()),
            Err(error) => return Err(self.error(node, error)),
        };
        if predicate
            .contains_parameters(&self.facts.types)
            .map_err(|e| self.error(node, e))?
            || predicate
                .has_regions_outside(&self.facts.types, 1)
                .map_err(|e| self.error(node, e))?
            || predicate
                .contains_opaques(&self.facts.types)
                .map_err(|e| self.error(node, e))?
        {
            return Ok(());
        }
        self.discharge_closed_call_scope(node, &evidence.proof)
    }

    fn discharge_closed_call_scope(
        &mut self,
        node: NodeId,
        proof: &Obligations,
    ) -> Result<(), Error> {
        let proof_payload = proof.allocation_bytes().saturating_add(1);
        self.state
            .infer
            .account_work(proof_payload.saturating_mul(4))
            .map_err(|e| self.error(node, e))?;
        let needs_caller = !proof
            .self_contained(&self.facts.types)
            .map_err(|e| self.error(node, e))?;
        if needs_caller {
            self.prepare_universal_call_inputs(node)?;
        }
        let caller = self
            .state
            .call_types
            .caller_inputs()
            .filter(|_| needs_caller);
        let caller_payload = caller.map_or(0, super::CallerInputs::payload);
        let payload = proof_payload.saturating_add(caller_payload);
        self.state
            .infer
            .account_work(caller_payload)
            .map_err(|e| self.error(node, e))?;
        let limit = payload.saturating_mul(64).clamp(1024, 1_000_000);
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        let seeded = if let Some(caller) = caller {
            caller
                .graph(self.facts, &mut queries, limit)
                .map_err(|e| self.error(node, e))
        } else {
            Constraints::new(&self.facts.types, &self.facts.lifetimes.variances, 0, limit)
                .map_err(|e| self.error(node, e))
        };
        let seeded_steps = queries.steps();
        self.state
            .infer
            .account_work(seeded_steps)
            .map_err(|e| self.error(node, e))?;
        let mut constraints = seeded?;
        let required = super::universe_inputs::foreign_regions(
            self.facts,
            self.state.body,
            proof,
            caller,
            &mut constraints,
            &mut queries,
        );
        let required = required.and_then(|()| proof.require(&mut constraints, &mut queries));
        let query_steps = queries.steps().saturating_sub(seeded_steps);
        self.state
            .infer
            .account_work(query_steps)
            .map_err(|e| self.error(node, e))?;
        required.map_err(|e| self.error(node, e))?;
        let answer = constraints.check_aliases_counted();
        let steps = answer.steps;
        self.state
            .infer
            .account_work(steps)
            .map_err(|e| self.error(node, e))?;
        answer.result.map_err(|error| self.error(node, error))
    }
}
