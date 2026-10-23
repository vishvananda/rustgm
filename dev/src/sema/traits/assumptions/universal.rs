//! Universal premises supply fresh existential instances in the requesting proof.
use super::{Assumptions, Clause, Context, Goal, Selection, Store, alternatives, infer};
use crate::support::hash::HashMap;
use crate::types::{
    RegionId,
    parameters::{clauses::QuantifiedId, predicates::Predicate},
    substitute::Bound,
};
use std::ops::Range;

pub(in crate::sema) struct ScopedEvidence {
    pub clause: Clause,
    pub predicate: Predicate,
    pub regions: Range<u32>,
}
impl ScopedEvidence {
    pub fn same_clause(
        &self,
        store: &mut Store,
        context: &mut Context,
        other: &Self,
    ) -> Result<bool, infer::Error> {
        let (same, work) = store
            .same_clause(self.clause, other.clause)
            .map_err(|_| infer::Error::InvalidIdentity)?;
        context.account_work(work)?;
        Ok(same)
    }
}
pub(in crate::sema) type ScopedSelection = Selection<ScopedEvidence>;

/// Alternatives in one query can share opening work at the same arity/frontier.
/// Only the unique winner publishes that interval; no opening supplies permission.
#[derive(Default)]
pub(in crate::sema) struct Openings {
    binders: HashMap<u32, (Bound, u32)>,
}
impl Openings {
    pub fn prepare(
        &mut self,
        store: &mut Store,
        context: &mut Context,
        clause: Clause,
        first: u32,
    ) -> Result<(Predicate, ScopedEvidence), infer::Error> {
        let id = match clause {
            Clause::Predicate(predicate) => {
                return Ok((
                    predicate,
                    ScopedEvidence {
                        clause,
                        predicate,
                        regions: first..first,
                    },
                ));
            }
            Clause::ForAll(id) => id,
        };
        if first == 0 || first >= 0x8000_0000 {
            return Err(infer::Error::InvalidIdentity);
        }
        let quantified = store.quantified(id).ok_or(infer::Error::InvalidIdentity)?;
        let (fold, end) = match self.binders.entry(quantified.bound_regions) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let end = first
                    .checked_add(quantified.bound_regions)
                    .filter(|end| *end < 0x8000_0000)
                    .ok_or(infer::Error::Budget)?;
                context.account_work(quantified.bound_regions as usize)?;
                entry.insert((Bound::new((first..end).map(RegionId).collect()), end))
            }
        };
        let before = fold.steps;
        let predicate = fold
            .predicate(store, quantified.predicate)
            .map_err(|_| infer::Error::InvalidIdentity);
        context.account_work(fold.steps.saturating_sub(before).saturating_add(1))?;
        let predicate = predicate?;
        Ok((
            predicate,
            ScopedEvidence {
                clause,
                predicate,
                regions: first..*end,
            },
        ))
    }
}
impl Assumptions {
    pub fn has_quantified(&self) -> bool {
        self.quantified().is_some()
    }

    /// Check an independently formed declaration requirement against caller
    /// premises only. Scratch equations never become executable proof evidence.
    pub fn unambiguous_scoped_declaration(
        &self,
        store: &mut Store,
        context: &mut Context,
        source: Clause,
        predicate: Predicate,
        first: u32,
    ) -> Result<bool, infer::Error> {
        context.account_work(1)?;
        let present = match source {
            Clause::Predicate(value) => value == predicate && self.contains(value),
            Clause::ForAll(id) => self.contains_quantified(id),
        };
        if !present || first == 0 || first >= 0x8000_0000 {
            return Err(infer::Error::InvalidIdentity);
        }
        let mut scratch = Context::default();
        let mut substitution = infer::Instantiation::new(&mut scratch, store, &[])?;
        let goal = scratch.instantiate_predicate(store, predicate, &mut substitution)?;
        let result = self.select_scoped(store, &mut scratch, &goal, first);
        context.account_work(scratch.steps)?;
        Ok(matches!(result?, Selection::Proven(_)))
    }

    pub fn contains_quantified(&self, clause: QuantifiedId) -> bool {
        self.quantified()
            .is_some_and(|index| index.seen.contains(&clause))
    }

    pub fn select_scoped(
        &self,
        store: &mut Store,
        context: &mut Context,
        goal: &Goal,
        first: u32,
    ) -> Result<ScopedSelection, infer::Error> {
        let Some(index) = self.quantified() else {
            return Ok(match self.select(store, context, goal)? {
                Selection::Absent => Selection::Absent,
                Selection::Deferred => Selection::Deferred,
                Selection::Proven(evidence) => Selection::Proven(ScopedEvidence {
                    clause: Clause::Predicate(evidence.assumption()),
                    predicate: evidence.assumption(),
                    regions: first..first,
                }),
            });
        };
        context.account_work(1)?;
        if first == 0 || first >= 0x8000_0000 {
            return Err(infer::Error::InvalidIdentity);
        }
        let Some((item, _)) = goal.trait_arguments() else {
            return Err(infer::Error::InvalidIdentity);
        };
        let Some(subject) = alternatives::subject(store, context, goal)? else {
            return Ok(Selection::Deferred);
        };
        let candidates = subject.heads().flat_map(|head| {
            self.candidates(item, head)
                .iter()
                .copied()
                .map(Clause::Predicate)
                .chain(
                    index
                        .candidates
                        .get(&(item, head))
                        .into_iter()
                        .flatten()
                        .copied()
                        .map(Clause::ForAll),
                )
        });
        let mut openings = Openings::default();
        alternatives::choose(
            store,
            context,
            goal,
            candidates,
            |store, context, clause| openings.prepare(store, context, clause, first),
            |store, context, left, right| {
                if matches!(
                    (left.clause, right.clause),
                    (Clause::Predicate(_), Clause::Predicate(_))
                ) && alternatives::ordinary_consequence(goal, left.predicate, right.predicate)
                {
                    return Ok(true);
                }
                left.same_clause(store, context, right)
            },
        )
    }
}
