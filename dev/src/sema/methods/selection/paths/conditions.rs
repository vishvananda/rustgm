//! Deferred applicability goals retain isolated equations and their private region frontier.
use super::{Context, Declarations, DefId, OwnedCaller};
#[cfg(test)]
#[path = "../../../../../tests/invariants/path_readiness.rs"]
mod tests;
use crate::{
    sema::traits::{
        assumptions::Assumptions,
        selection::{self, Selection},
    },
    types::infer::predicates::Predicate,
};

pub(super) enum Progress {
    Ready(bool),
    Pending(DefId),
}
pub(super) struct Pending {
    goals: Vec<usize>,
    cursor: usize,
    next: Vec<usize>,
    first: u32,
    query: Option<Box<selection::Pending>>,
}
impl Pending {
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
        goals: &[Predicate],
        caller: &mut OwnedCaller,
    ) -> Result<Progress, &'static str> {
        loop {
            while let Some(&index) = self.goals.get(self.cursor) {
                match self.select(facts, context, &goals[index], caller)? {
                    selection::Progress::Pending(owner, query) => {
                        self.first = self.first.max(query.allocation_end());
                        self.query = Some(query);
                        return Ok(Progress::Pending(owner));
                    }
                    selection::Progress::Ready(Selection::Absent) => {
                        return Ok(Progress::Ready(false));
                    }
                    selection::Progress::Ready(Selection::Deferred) => self.next.push(index),
                    selection::Progress::Ready(Selection::Proven(proof)) => {
                        self.first = self.first.max(proof.proof.regions.end);
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
        goal: &Predicate,
        caller: &mut OwnedCaller,
    ) -> Result<selection::Progress, &'static str> {
        let selected = if let Some(query) = self.query.take() {
            // These IDs belong to a disposable probe, never a body proof. Its
            // immutable prefix and caller predate the wait. Fresh header trials
            // allocate beyond both its own watermark and any intervening bodies.
            self.first = self.first.max(frontier(facts)?);
            query.resume(facts, context, goal, self.first)
        } else if facts
            .generics
            .implementations
            .as_ref()
            .is_none_or(|registry| registry.constant_headers_ready())
        {
            let empty = Assumptions::default();
            selection::select_available(
                facts,
                context,
                goal,
                selection::Caller {
                    assumptions: caller.traits.as_deref().unwrap_or(&empty),
                    environment: caller.environment.as_ref(),
                    first_region: self.first,
                    retain_sized: false,
                },
                caller.availability,
            )
            .map(selection::Progress::Ready)
        } else {
            selection::begin(
                facts,
                context,
                goal,
                selection::OwnedCaller {
                    assumptions: caller.traits.get_or_insert_with(Default::default).clone(),
                    environment: caller.environment.clone(),
                    first_region: self.first,
                    retain_sized: false,
                },
                caller.availability,
            )
        };
        selected.map_err(|_| "invalid or exhausted method trait selection")
    }
}
fn frontier(facts: &Declarations) -> Result<u32, &'static str> {
    facts
        .body_regions
        .len()
        .checked_add(1)
        .and_then(|n| u32::try_from(n).ok())
        .filter(|n| *n < 0x8000_0000)
        .ok_or("method trait region identity space exhausted")
}
