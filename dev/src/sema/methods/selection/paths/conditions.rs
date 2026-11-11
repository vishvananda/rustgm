//! Deferred applicability goals retain isolated equations and their private region frontier.
use super::{Context, Declarations, OwnedCaller};
#[cfg(test)]
#[path = "../../../../../tests/invariants/path_readiness.rs"]
mod tests;
use super::super::lookup_goals::Goal;
use crate::sema::traits::{
    assumptions::Assumptions,
    selection::{self, Selection},
};

pub(super) enum Progress {
    Ready(bool),
    Pending(selection::Dependency),
}
pub(super) struct Pending {
    goals: Vec<usize>,
    cursor: usize,
    next: Vec<usize>,
    first: u32,
    query: Option<Box<selection::Pending>>,
}
impl Pending {
    /// A newly exposed property goal shares this trial's private region frontier.
    pub fn retry(&mut self, count: usize) {
        self.goals.clear();
        self.goals.extend(0..count);
        self.cursor = 0;
        self.next.clear();
    }

    pub fn new(count: usize, facts: &Declarations) -> Result<Self, &'static str> {
        Ok(Self {
            goals: (0..count).collect(),
            cursor: 0,
            next: Vec::new(),
            first: frontier(facts)?,
            query: None,
        })
    }
    pub fn poll(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        goals: &[Goal],
        caller: &mut OwnedCaller,
    ) -> Result<Progress, &'static str> {
        loop {
            while let Some(&index) = self.goals.get(self.cursor) {
                match self.select(facts, context, &goals[index], caller)? {
                    selection::Progress::Inferred { .. } => {
                        return Err("non-refining applicability returned partial inputs");
                    }
                    selection::Progress::Pending(owner, query) => {
                        self.first = self.first.max(query.allocation_end());
                        self.query = Some(query);
                        return Ok(Progress::Pending(owner));
                    }
                    selection::Progress::Ready(Selection::Refuted(refutation)) => {
                        refutation.reject()?;
                        return Ok(Progress::Ready(false));
                    }
                    selection::Progress::Ready(Selection::Absent) => {
                        return Ok(Progress::Ready(false));
                    }
                    selection::Progress::Ready(Selection::Deferred) => self.next.push(index),
                    selection::Progress::Ready(Selection::Proven(proof)) => {
                        self.first = self.first.max(proof.proof.regions.end);
                        if !goals[index].possible(facts, context, &proof.proof)? {
                            return Ok(Progress::Ready(false));
                        }
                    }
                }
                self.cursor += 1;
            }
            if self.next.len() == self.goals.len() {
                // Remaining inference can make the candidate possible. Only the
                // selected ordinary callable can authorize its complete bounds.
                return Ok(Progress::Ready(true));
            }
            self.goals = std::mem::take(&mut self.next);
            self.cursor = 0;
        }
    }

    fn select(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        goal: &Goal,
        caller: &mut OwnedCaller,
    ) -> Result<selection::Progress, &'static str> {
        let selected = if let Some(query) = self.query.take() {
            // These IDs belong to a disposable probe, never a body proof. Its
            // immutable prefix and caller predate the wait. Fresh header trials
            // allocate beyond both its own watermark and any intervening bodies.
            self.first = self.first.max(frontier(facts)?);
            query.resume(facts, context, &goal.predicate, self.first)
        } else if let Some(count) = goal.bound_regions {
            selection::begin_canonical(
                facts,
                context,
                &goal.predicate,
                caller.selection(self.first),
                (caller.availability, Some(count)),
                None,
            )
        } else if facts
            .generics
            .implementations
            .as_ref()
            .map(|registry| registry.selection_goal_ready(&facts.types, context, &goal.predicate))
            .transpose()
            .map_err(|_| "invalid pending method header readiness")?
            .unwrap_or(true)
            && !facts.types.nominal_sizes_pending()
        {
            let empty = Assumptions::default();
            selection::select_available(
                facts,
                context,
                &goal.predicate,
                selection::Caller {
                    assumptions: caller.traits.as_deref().unwrap_or(&empty),
                    environment: caller.environment.as_ref(),
                    first_region: self.first,
                    retain_sized: false,
                    nesting: selection::Nesting::ROOT,
                    placeholders: None,
                },
                caller.availability,
            )
            .map(selection::Progress::Ready)
        } else {
            selection::begin(
                facts,
                context,
                &goal.predicate,
                caller.selection(self.first),
                caller.availability,
            )
        };
        selected.map_err(|error| {
            crate::sema::census::fold_cause(
                &error,
                "method trait selection work exhausted",
                "invalid method trait selection",
            )
        })
    }
}
fn frontier(facts: &Declarations) -> Result<u32, &'static str> {
    facts
        .body_regions
        .probe_frontier()
        .map_err(|_| "method trait region identity space exhausted")
}
