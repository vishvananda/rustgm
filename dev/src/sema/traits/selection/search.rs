//! Iterative candidate search; each child proof commits only its resolved goal.
mod allocations;
mod builtins;
#[cfg(test)]
#[path = "../../../../tests/invariants/implementation_conditions.rs"]
mod condition_tests;
pub(super) mod conditions;
mod continuation;
mod equalities;
mod frames;
#[cfg(test)]
#[path = "../../../../tests/invariants/selection_headers.rs"]
mod header_tests;
mod input_universe;
mod inputs;
#[cfg(test)]
#[path = "../../../../tests/invariants/formation_clauses.rs"]
mod nominal_tests;
mod normalization;
mod partial;
#[cfg(test)]
#[path = "../../../../tests/invariants/retained_candidates.rs"]
mod partial_tests;
mod result_bounds;
mod scopes;
mod suspension;
#[cfg(test)]
#[path = "../../../../tests/invariants/universal_inference.rs"]
mod universal_inference_tests;
mod wakeups;
pub(in crate::sema) use suspension::{Pending, Progress, begin, begin_normalized, begin_refining};
#[cfg(test)]
#[path = "../../../../tests/invariants/universal_premises.rs"]
mod assumption_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/quantified_obligations.rs"]
mod quantified_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/proof_cache.rs"]
mod tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/universal_work.rs"]
mod universal_work;
use super::{
    Caller, Context, Declarations, Evidence, Goal, Predicate, Selection, Source, inference,
};
use crate::types::{infer, lifetime_proof::ScopeId, outlives::Pair};
use continuation::Query;
use frames::{Frame, Trial};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decision {
    Absent,
    Deferred,
    Proven(usize),
}
enum Action {
    Continue,
    Descend,
    Complete(Decision),
    Wait(crate::resolve::model::DefId),
    Yield,
}
struct Node {
    goal: Predicate,
    source: Option<Source>,
    children: Vec<usize>,
    first: u32,
    end: u32,
    equalities: Vec<Pair>,
    outlives: Vec<Pair>,
    type_outlives: Vec<(super::TypeId, super::RegionId)>,
    sized: Vec<super::TypeId>,
}
struct Search<'a> {
    caller: Caller<'a>,
    query: Query,
}

pub(super) fn prove(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: Caller<'_>,
    availability: super::Availability,
    normalized_inputs: bool,
) -> Result<Selection, String> {
    let mut first = Frame::new(facts, context, goal, caller.first_region)?;
    if normalized_inputs {
        // The enclosing normalizer owns these input proofs and retains them
        // even when the search discards a candidate or reuses a child result.
        first.normalized_inputs = Some(frames::NormalizedInputs { proof: None });
    }
    let mut search = Search::new(caller, availability);
    let decision = search.run(facts, first)?;
    search.result(facts, context, goal, decision)
}

impl Search<'_> {
    fn result(
        mut self,
        facts: &mut Declarations,
        context: &mut Context,
        goal: &Goal,
        decision: Decision,
    ) -> Result<Selection, String> {
        if let Decision::Proven(root) = decision
            && self.query.universal.is_some()
        {
            self.complete_universal(facts, context, root)?;
        }
        context.account_work(self.query.steps).map_err(inference)?;
        match decision {
            Decision::Absent => Ok(Selection::Absent),
            Decision::Deferred => Ok(Selection::Deferred),
            Decision::Proven(root) => {
                if self.query.universal.is_none() {
                    context
                        .equate_proved_predicate(
                            &mut facts.types,
                            goal,
                            self.query.nodes[root].goal,
                        )
                        .map_err(inference)?;
                }
                self.evidence(root).map(Selection::Proven)
            }
        }
    }
    fn charge(&mut self, context: &mut Context) -> Result<(), String> {
        self.query.steps = self
            .query
            .steps
            .saturating_add(1 + std::mem::take(&mut context.steps));
        if self.query.steps > 1_000_000 {
            return Err("trait selection work budget exceeded".into());
        }
        Ok(())
    }
    fn remember(
        &mut self,
        canonical: Option<Predicate>,
        cycles: usize,
        decision: Decision,
        scope: ScopeId,
    ) {
        match decision {
            Decision::Proven(index) => {
                let node = &self.query.nodes[index];
                // Reusing existential evidence requires fresh region rebasing.
                if node.first == node.end && self.query.cacheable(index) {
                    self.query.remember(scope, node.goal, decision);
                }
            }
            Decision::Absent if cycles == self.query.cycles => {
                // A cycle cutoff is relative to the active proof stack, not a
                // negative fact. Neither it nor any ancestor may cache failure.
                if let Some(goal) = canonical {
                    self.query.remember(scope, goal, decision);
                }
            }
            Decision::Absent | Decision::Deferred => {}
        }
    }
    fn advance(&mut self, facts: &mut Declarations, frame: &mut Frame) -> Result<Action, String> {
        if frame.pending_headers.is_some() {
            return self.await_header(facts, frame);
        }
        if !frame.started {
            return self.start(facts, frame);
        }
        if frame.normalizing() {
            return self.advance_normalization(facts, frame);
        }
        if let Some(trial) = &mut frame.trial {
            if trial.cursor == trial.pending.len() {
                trial.pending.clear();
                trial.cursor = 0;
            }
            trial.poll_condition_changes(facts, None)?;
            if let Some(&index) = trial.pending.get(trial.cursor) {
                trial.wakeups.start(index);
                return Ok(Action::Descend);
            }
            let result = if trial.wakeups.waiting() {
                Decision::Deferred
            } else {
                let Some(result) = self.finish(facts, trial)? else {
                    return Ok(Action::Descend);
                };
                result
            };
            self.charge(&mut trial.inputs.context)?;
            let retained = frame.trial.take();
            match result {
                Decision::Absent => {}
                Decision::Deferred
                    if self.query.refining
                        && frame.inference != frames::Inference::Disabled
                        && frame.scope == self.query.input_scope()
                        && !frame.normalizing() =>
                {
                    if frame.inference == frames::Inference::Committed {
                        frame.trial = retained;
                        return Ok(Action::Yield);
                    }
                    if frame.partial.is_some() {
                        frame.deferred = true;
                    } else {
                        frame.partial = retained.map(Box::new);
                    }
                }
                Decision::Deferred => frame.deferred = true,
                Decision::Proven(node) => {
                    if frame.selected.replace(node).is_some() {
                        return Ok(Action::Complete(Decision::Deferred));
                    }
                }
            }
            return Ok(Action::Continue);
        }
        if let Some(&owner) = frame.candidates.get(frame.cursor) {
            frame.cursor += 1;
            let mut trial = Trial::new(facts, frame, owner, self.query.allocations.floor())?;
            self.query.allocations.record(trial.end);
            self.query.reserve(trial.scope, trial.first..trial.end)?;
            let result = trial.inputs.context.equate_predicates(
                &facts.types,
                &trial.goal,
                &trial.instance.goal,
            );
            self.charge(&mut trial.inputs.context)?;
            match result {
                Ok(()) => frame.trial = Some(trial),
                Err(infer::Error::Mismatch | infer::Error::Occurs) => {}
                Err(infer::Error::Normalization) => frame.deferred = true,
                Err(error) => return Err(inference(error)),
            }
            return Ok(Action::Continue);
        }
        if !frame.deferred
            && frame.selected.is_none()
            && let Some(trial) = frame.partial.take()
        {
            frame.trial = Some(*trial);
            frame.inference = frames::Inference::Committed;
            return Ok(Action::Yield);
        }
        Ok(Action::Complete(
            if frame.deferred || frame.partial.is_some() {
                Decision::Deferred
            } else {
                frame.selected.map_or(Decision::Absent, Decision::Proven)
            },
        ))
    }
    fn start(&mut self, facts: &mut Declarations, frame: &mut Frame) -> Result<Action, String> {
        frame.started = true;
        frame.cycles = self.query.cycles;
        if frame.goal.projection_result().is_some() {
            return self.start_equality(facts, frame);
        }
        if frame.goal.builtin_bound().is_some() {
            if let Some(decision) = self.cached_query(facts, frame)? {
                return Ok(Action::Complete(decision));
            }
            return if let Some(action) = Self::start_inputs(facts, frame)? {
                Ok(action)
            } else {
                self.complete_builtin(facts, frame).map(Action::Complete)
            };
        }
        let needs_normalization = match self.assume(facts, frame) {
            Ok(Some(action)) => return Ok(action),
            Ok(None) => false,
            Err(infer::Error::Normalization) => true,
            Err(error) => return Err(inference(error)),
        };
        if let Some(decision) = self.cached_query(facts, frame)? {
            return Ok(Action::Complete(decision));
        }
        if frame.normalized_inputs.is_none()
            && let Some(action) = Self::start_inputs(facts, frame)?
        {
            return Ok(action);
        }
        if needs_normalization {
            return Ok(Action::Complete(Decision::Deferred));
        }
        if frame.normalized_inputs.is_some()
            && let Some(action) = self.derive_result(facts, frame)?
        {
            return Ok(action);
        }
        self.candidates(facts, frame)
    }
    fn assume(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, infer::Error> {
        use super::Assumed;
        let first = frame.available_region().max(self.query.allocations.floor());
        match self.caller.assumptions.select_scoped(
            &mut facts.types,
            &mut frame.inputs.context,
            &frame.goal,
            first,
        )? {
            Assumed::Proven(proof) => {
                let goal = frame
                    .inputs
                    .context
                    .resolved_predicate(&mut facts.types, &frame.goal)?;
                self.query
                    .reserve(frame.scope, first..proof.regions.end)
                    .map_err(|_| infer::Error::InvalidIdentity)?;
                let index = self.query.push(
                    Node {
                        goal,
                        source: Some(Source::assumed(&proof)?),
                        children: Vec::new(),
                        first: frame.first,
                        end: proof.regions.end,
                        equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                        outlives: Vec::new(),
                        type_outlives: Vec::new(),
                        sized: Vec::new(),
                    },
                    frame.scope,
                    first..proof.regions.end,
                );
                self.query.allocations.record(proof.regions.end);
                return Ok(Some(Action::Complete(Decision::Proven(index))));
            }
            Assumed::Deferred => return Ok(Some(Action::Complete(Decision::Deferred))),
            Assumed::Absent => {}
        }
        Ok(None)
    }
    fn candidates(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let Some(registry) = &mut facts.generics.implementations else {
            return Ok(Action::Complete(Decision::Absent));
        };
        let candidates = registry
            .candidates(
                &mut facts.types,
                &mut frame.inputs.context,
                &frame.goal,
                self.query.availability,
            )
            .map_err(inference)?;
        match candidates {
            crate::sema::traits::implementations::Candidates::Ready(owners) => {
                frame.candidates = owners;
                Ok(Action::Continue)
            }
            crate::sema::traits::implementations::Candidates::Pending(owners) => {
                frame.pending_headers = Some(owners.into_iter());
                self.await_header(facts, frame)
            }
        }
    }
    fn cached_query(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Decision>, String> {
        match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => {
                frame.canonical = Some(goal);
                if let Some(decision) = self.query.completed(frame.scope, goal) {
                    self.query.reused |= matches!(decision, Decision::Proven(_));
                    return Ok(Some(decision));
                }
                let (key, work) = facts.types.shape_key(goal)?;
                frame.inputs.context.account_work(work).map_err(inference)?;
                if !self.query.active.insert(key) {
                    self.query.cycles += 1;
                    return Ok(Some(Decision::Absent));
                }
                frame.key = Some(key);
            }
            Err(infer::Error::Unresolved) => {}
            Err(error) => return Err(inference(error)),
        }
        Ok(None)
    }
    fn accept(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        if frame.normalizing() {
            return self.accept_normalization(facts, frame, decision);
        }
        let trial = frame
            .trial
            .as_mut()
            .ok_or("missing parent trait candidate")?;
        if trial
            .instance
            .quantified
            .as_ref()
            .is_some_and(|conditions| conditions.waiting())
        {
            return self.accept_condition(frame, decision);
        }
        let index = *trial
            .pending
            .get(trial.cursor)
            .ok_or("missing pending trait condition")?;
        trial.cursor += 1;
        match decision {
            Decision::Absent => {
                self.charge(&mut trial.inputs.context)?;
                frame.trial = None;
            }
            Decision::Deferred => trial.defer_condition(facts, index)?,
            Decision::Proven(node) => {
                trial
                    .inputs
                    .context
                    .equate_proved_predicate(
                        &mut facts.types,
                        &trial.instance.goals[index],
                        self.query.nodes[node].goal,
                    )
                    .map_err(inference)?;
                if self.query.nodes[node].first != self.query.nodes[node].end {
                    trial.end = self.query.nodes[node].end;
                }
                trial.children.push(node);
                trial.complete_condition(index)?;
                self.charge(&mut trial.inputs.context)?;
            }
        }
        Ok(())
    }
    fn evidence(mut self, root: usize) -> Result<Evidence, String> {
        let source = self.query.nodes[root]
            .source
            .take()
            .ok_or("missing selected trait evidence")?;
        let mut evidence = Evidence {
            source,
            proof: self.query.allocations.finish(self.query.nodes[root].end),
            sized: Vec::new(),
        };
        let mut scopes = self.query.scopes.take();
        let mut work = vec![root];
        let mut visited = self
            .query
            .reused
            .then(|| vec![false; self.query.nodes.len()]);
        while let Some(index) = work.pop() {
            if let Some(visited) = &mut visited {
                if visited[index] {
                    continue;
                }
                visited[index] = true;
            }
            let node = &mut self.query.nodes[index];
            work.extend(std::mem::take(&mut node.children));
            if let Some(scopes) = &mut scopes {
                scopes.collect(index, node, &mut evidence)?;
            } else {
                evidence.proof.equalities.append(&mut node.equalities);
                evidence.proof.outlives.append(&mut node.outlives);
                evidence.proof.types.append(&mut node.type_outlives);
                evidence.sized.append(&mut node.sized);
            }
        }
        if let Some(scopes) = scopes {
            scopes.finish(&mut evidence.proof)?;
        }
        Ok(evidence)
    }
}
