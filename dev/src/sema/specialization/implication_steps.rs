//! Each directional query owns its typed goals and every suspended proof payment.
#[cfg(test)]
#[path = "../../../tests/invariants/inferred_parent_obligations.rs"]
mod fixpoint_tests;
use super::{
    proof_inputs::Inputs,
    registry_graph::{PairKey, Relation},
    retention_work::Budget,
};
use crate::{
    sema::{
        model::Declarations,
        traits::{implementations::EntryKey, selection},
    },
    types::{lifetime_proof::Obligations, parameters::predicates::Bound},
};
use std::sync::Arc;
pub(super) struct Attempt {
    pair: PairKey,
    budget: Budget,
    direction: Option<Direction>,
    first: Option<Direction>,
    forward_done: bool,
}
struct Direction {
    inputs: Inputs,
    /// The goal indexes this pass selects, in order.
    pass: Vec<usize>,
    cursor: usize,
    /// Goals this pass found ambiguous, retried after any goal is proven.
    deferred: Vec<usize>,
    progressed: bool,
    pending: Option<Box<selection::Pending>>,
    proof: Obligations,
    query_work: usize,
    paid_query_work: usize,
}
pub(super) enum Progress {
    Pending(selection::Dependency),
    Complete(Option<Relation>),
}
#[derive(Clone, Copy)]
enum Next {
    Pending(selection::Dependency),
    Proven,
    Absent,
}
impl Attempt {
    pub(super) fn new(pair: PairKey, budget: Budget) -> Result<Box<Self>, &'static str> {
        budget.charge(1, size_of::<Self>())?;
        Ok(Box::new(Self {
            pair,
            budget,
            direction: None,
            first: None,
            forward_done: false,
        }))
    }
    pub(super) fn advance(&mut self, facts: &mut Declarations) -> Result<Progress, String> {
        loop {
            let [first, second] = self.pair.entries();
            let (child, parent) = if self.forward_done {
                (second, first)
            } else {
                (first, second)
            };
            if self.direction.is_none() {
                self.direction = Direction::new(facts, child, parent, &self.budget)?;
            }
            let next = if let Some(direction) = &mut self.direction {
                direction.advance(facts, &self.budget)?
            } else {
                Next::Absent
            };
            match next {
                Next::Pending(dependency) => return Ok(Progress::Pending(dependency)),
                Next::Absent | Next::Proven if !self.forward_done => {
                    if matches!(next, Next::Proven) {
                        self.first = self.direction.take();
                    }
                    self.direction = None;
                    self.forward_done = true;
                }
                Next::Proven if self.first.is_some() => {
                    return Err(
                        "equivalent overlapping implementation headers cannot specialize".into(),
                    );
                }
                Next::Absent | Next::Proven => {
                    let selected = if matches!(next, Next::Proven) {
                        self.direction.as_mut()
                    } else {
                        self.first.as_mut()
                    };
                    return selected.map_or(Ok(Progress::Complete(None)), |direction| {
                        direction
                            .relation(facts, &self.budget)
                            .map(|relation| Progress::Complete(Some(relation)))
                    });
                }
            }
        }
    }
}
impl Direction {
    fn new(
        facts: &mut Declarations,
        child: EntryKey,
        parent: EntryKey,
        budget: &Budget,
    ) -> Result<Option<Self>, String> {
        let Some(inputs) = Inputs::new(facts, child, parent, budget)? else {
            return Ok(None);
        };
        // Each pass and its deferred goals hold a slot for every goal.
        let count = inputs.goals.len();
        budget.charge(count, count.saturating_mul(2 * size_of::<usize>()))?;
        let proof = Obligations::new(inputs.next_region);
        Ok(Some(Self {
            inputs,
            pass: (0..count).collect(),
            cursor: 0,
            deferred: Vec::with_capacity(count),
            progressed: false,
            pending: None,
            proof,
            query_work: 0,
            paid_query_work: 0,
        }))
    }
    fn pay(&mut self, budget: &Budget) -> Result<(), &'static str> {
        self.inputs.pay_context()?;
        let delta = self.query_work.saturating_sub(self.paid_query_work);
        self.paid_query_work = self.query_work;
        budget.charge(delta, 0)
    }
    /// Like rustc's `specializes`, every parent obligation is registered and
    /// solved to a fixpoint: an ambiguous goal waits until another is proven,
    /// and ambiguity that no proof resolves means the child does not specialize.
    fn advance(&mut self, facts: &mut Declarations, budget: &Budget) -> Result<Next, String> {
        loop {
            let Some(&index) = self.pass.get(self.cursor) else {
                if self.deferred.is_empty() {
                    return Ok(Next::Proven);
                }
                if !std::mem::take(&mut self.progressed) {
                    return Ok(Next::Absent);
                }
                budget.charge(self.deferred.len(), 0)?;
                std::mem::swap(&mut self.pass, &mut self.deferred);
                self.deferred.clear();
                self.cursor = 0;
                continue;
            };
            let goal = self
                .inputs
                .goals
                .get(index)
                .ok_or("specialization implication goal is missing")?;
            // Rustc's coherence implication registers region constraints but does
            // not resolve them before returning specializes. Keep this actual
            // typed goal for the separate executable header/body reproof.
            if matches!(goal.builtin_bound(), Some(Bound::Outlives(_))) {
                self.cursor += 1;
                continue;
            }
            let result = if let Some(pending) = self.pending.take() {
                let first = pending.allocation_end().max(self.proof.regions.end);
                pending.resume_counted(
                    facts,
                    &mut self.inputs.context,
                    goal,
                    first,
                    &mut self.query_work,
                )
            } else {
                selection::begin_declaration_counted(
                    facts,
                    &mut self.inputs.context,
                    goal,
                    selection::OwnedCaller {
                        assumptions: Arc::clone(&self.inputs.assumptions),
                        environment: self.inputs.environment.clone(),
                        first_region: self.proof.regions.end,
                        retain_sized: false,
                        nesting: selection::Nesting::ROOT,
                        placeholders: None,
                    },
                    &mut self.query_work,
                )
            };
            self.pay(budget)?;
            match result? {
                selection::Progress::Pending(dependency, pending) => {
                    self.pending = Some(pending);
                    return Ok(Next::Pending(dependency));
                }
                selection::Progress::Ready(selection::Selection::Proven(evidence)) => {
                    if evidence.provisional_size || !evidence.sized.is_empty() {
                        return Err(
                            "specialization implication requires finalized sized contracts".into(),
                        );
                    }
                    let allocation = evidence.proof.retained_allocation();
                    budget.charge(allocation.visits, allocation.bytes)?;
                    self.proof.extend(evidence.proof)?;
                    self.cursor += 1;
                    self.progressed = true;
                    self.query_work = 0;
                    self.paid_query_work = 0;
                }
                selection::Progress::Ready(
                    selection::Selection::Absent | selection::Selection::Refuted(_),
                ) => return Ok(Next::Absent),
                selection::Progress::Ready(selection::Selection::Deferred) => {
                    self.deferred.push(index);
                    self.cursor += 1;
                    self.query_work = 0;
                    self.paid_query_work = 0;
                }
                selection::Progress::Inferred { .. } => {
                    return Err(
                        "specialization implication cannot refine rigid child inputs".into(),
                    );
                }
            }
        }
    }
    fn relation(&mut self, facts: &mut Declarations, budget: &Budget) -> Result<Relation, String> {
        let count = self
            .inputs
            .parent_bindings
            .len()
            .saturating_add(self.inputs.parent_regions.len());
        budget.charge(
            1 + count,
            count.saturating_mul(4 * size_of::<crate::types::arguments::Argument>()),
        )?;
        let result = self.inputs.parent_arguments(facts);
        self.pay(budget)?;
        let arguments =
            super::argument_closure::close(facts, &self.inputs, &self.proof, result?, budget)?;
        Ok(Relation {
            child: self.inputs.child,
            parent: self.inputs.parent,
            parent_arguments: arguments,
        })
    }
}
