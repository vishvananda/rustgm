//! Iterative candidate search; each child proof commits only its resolved goal.
mod allocations;
#[cfg(test)]
#[path = "../../../../tests/invariants/auto_cycle_work.rs"]
mod auto_cycle_tests;
mod auto_obligations;
mod builtins;
mod callable_queries;
mod callables;
mod coinduction;
mod completed_proofs;
#[cfg(test)]
#[path = "../../../../tests/invariants/marker_evidence.rs"]
mod marker_evidence_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/marker_readiness.rs"]
mod marker_readiness_tests;
mod primitive_methods;
pub(in crate::sema) use completed_proofs::Reuse;
#[cfg(test)]
#[path = "../../../../tests/invariants/erased_formation.rs"]
mod callable_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/implementation_conditions.rs"]
mod condition_tests;
pub(super) mod conditions;
mod continuation;
mod equalities;
#[cfg(test)]
#[path = "../../../../tests/invariants/equality_search.rs"]
mod equality_tests;
mod forwarding;
#[cfg(test)]
#[path = "../../../../tests/invariants/forwarded_calls.rs"]
mod forwarding_tests;
mod frames;
#[cfg(test)]
#[path = "../../../../tests/invariants/selection_headers.rs"]
mod header_tests;
mod input_universe;
mod inputs;
mod member_conditions;
#[cfg(test)]
#[path = "../../../../tests/invariants/formation_clauses.rs"]
mod nominal_tests;
mod normalization;
mod partial;
#[cfg(test)]
#[path = "../../../../tests/invariants/retained_candidates.rs"]
mod partial_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/pending_closures.rs"]
mod pending_closure_tests;
mod result_bounds;
#[cfg(test)]
#[path = "../../../../tests/invariants/contract_evidence.rs"]
mod result_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/reused_selections.rs"]
mod reused_selection_tests;
mod scopes;
#[cfg(test)]
#[path = "../../../../tests/invariants/size_evidence.rs"]
mod size_evidence_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/size_readiness.rs"]
mod size_tests;
mod sized_obligations;
mod suspension;
#[cfg(test)]
#[path = "../../../../tests/invariants/universal_inference.rs"]
mod universal_inference_tests;
mod wakeups;
pub(in crate::sema) use suspension::{
    Pending, Progress, begin, begin_canonical, begin_normalized, begin_projection, begin_refining,
    begin_scoped_refining,
};
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
    Caller, Context, Declarations, Dependency, Evidence, Goal, Predicate, Selection, Source,
    inference,
};
use crate::types::{infer, outlives::Pair};
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
    Wait(Dependency),
    Yield,
}
struct Node {
    cycle_dependent: bool,
    provisional_size: bool,
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
    reuse: Option<&mut Reuse>,
) -> Result<Selection, String> {
    let mut first = Frame::new(facts, context, goal, caller.first_region)?;
    if normalized_inputs {
        // The enclosing normalizer owns these input proofs and retains them
        // even when the search discards a candidate or reuses a child result.
        first.normalized_inputs = Some(frames::NormalizedInputs { proof: None });
    }
    let mut search = Search::new(caller, availability);
    // Only completed declaration schemas may support reusable physical conditions.
    if reuse.is_some() && !facts.types.nominal_sizes_sealed() {
        return Err("proof reuse requires sealed nominal sizes".into());
    }
    let mut reuse = reuse;
    if let Some(reuse) = reuse.as_deref_mut() {
        search.query.reuse = Some(completed_proofs::Active::new(std::mem::take(reuse)));
    }
    let decision = search.run(facts, first);
    if !normalized_inputs && let Ok(decision) = &decision {
        search.query.export_completed(facts, *decision)?;
    }
    if let Some(reuse) = reuse
        && let Some(active) = search.query.reuse.take()
    {
        *reuse = active.reuse;
    }
    let decision = decision?;
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
    fn remember(&mut self, frame: &Frame, decision: Decision) {
        match decision {
            Decision::Proven(index) => {
                let node = &self.query.nodes[index];
                let goal = node.goal;
                // Ordinary cached answers cannot capture private witnesses or
                // open cycles. Auto proofs can share witnesses in one segment.
                if !node.cycle_dependent && node.first == node.end && self.query.cacheable(index) {
                    self.query.remember(frame.scope, goal, decision);
                }
                if !self.query.nodes[index].provisional_size {
                    self.query.remember_auto(frame, goal, decision);
                }
            }
            Decision::Absent if frame.cycles == self.query.cycles => {
                // A cycle cutoff is relative to the active proof stack, not a
                // negative fact. Neither it nor any ancestor may cache failure.
                if let Some(goal) = frame.canonical {
                    self.query.remember(frame.scope, goal, decision);
                }
            }
            Decision::Absent | Decision::Deferred => {}
        }
    }
    fn advance(&mut self, facts: &mut Declarations, frame: &mut Frame) -> Result<Action, String> {
        if frame.forming() {
            return self.advance_formation(frame);
        }
        if frame.pending_headers.is_some() {
            return self.await_header(facts, frame);
        }
        if !frame.started {
            return self.start(facts, frame);
        }
        if frame.normalizing() {
            return self.advance_normalization(facts, frame);
        }
        if frame.callable.is_some() {
            return self.complete_callable(facts, frame);
        }
        if frame.equality_hint.is_some() {
            return self.start_equality(facts, frame);
        }
        if frame.goal.builtin_bound().is_some() && !frame.explicit_auto {
            return self.complete_builtin(facts, frame);
        }
        if frame.trial.is_some() {
            return self.advance_trial(facts, frame);
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
        self.language_result(facts, frame)
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
            return if let Some(action) = self.start_inputs(facts, frame)? {
                Ok(action)
            } else {
                self.complete_builtin(facts, frame)
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
            && let Some(action) = self.start_inputs(facts, frame)?
        {
            return Ok(action);
        }
        if needs_normalization {
            return Ok(Action::Complete(Decision::Deferred));
        }
        let declared = (!facts.opaques.declarations.is_empty() || facts.types.has_dynamic_shapes())
            && matches!(
                frame
                    .inputs
                    .context
                    .head(&facts.types, frame.goal.subject())
                    .map_err(inference)?,
                Some(infer::Constructor::Opaque(..) | infer::Constructor::Dynamic { .. })
            );
        if (frame.normalized_inputs.is_some() || declared)
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
                        cycle_dependent: false,
                        provisional_size: false,
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
        if let Some(action) = self.start_callable(facts, frame)? {
            return Ok(action);
        }
        let Some(registry) = &mut facts.generics.implementations else {
            return self.language_result(facts, frame);
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
                // A marker on a type with no contextual lifetimes has the same
                // proof in every nested universe of this immutable caller.
                if matches!(
                    goal.bound,
                    crate::types::parameters::predicates::Bound::Auto(_)
                ) && facts.types.needs_region_context(goal.ty) == Some(false)
                {
                    frame.scope = self.query.input_scope();
                }
                if let Some(decision) = self.query.completed(frame.scope, goal) {
                    self.query.reused |= matches!(decision, Decision::Proven(_));
                    return Ok(Some(decision));
                }
                if let Some(decision) = self.query.reuse_completed(frame, goal) {
                    return Ok(Some(decision));
                }
                if matches!(
                    goal.bound,
                    crate::types::parameters::predicates::Bound::Auto(_)
                ) {
                    return self.auto_cycle(facts, frame, goal);
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
        if !matches!(decision, Decision::Proven(_)) {
            self.query.invalidate_auto(frame);
        }
        if frame.forming() {
            return Self::accept_formation(frame, decision);
        }
        if frame.normalizing() {
            return self.accept_normalization(facts, frame, decision);
        }
        if matches!(
            frame.callable.as_deref(),
            Some(callables::Pending::Forward(_))
        ) {
            return self.accept_forwarding(facts, frame, decision);
        }
        if frame.storage.is_some() {
            return self.accept_storage(frame, decision);
        }
        if frame.autos.is_some() {
            return self.accept_auto(frame, decision);
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
                    trial.end = trial.end.max(self.query.nodes[node].end);
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
            provisional_size: self.query.nodes[root].provisional_size,
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
