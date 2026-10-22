//! Clause comparison forgets unused local slots, never captures or proof ownership.
use super::{Clause, QuantifiedId};
use crate::support::hash::HashMap;
use crate::types::{Store, parameters::predicates::Predicate, substitute::Bound};
#[cfg(test)]
#[path = "../../../../tests/invariants/clause_equivalence.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Comparison(Predicate);

#[derive(Default)]
pub(super) struct Cache {
    keys: HashMap<QuantifiedId, Comparison>,
    preserved: HashMap<(QuantifiedId, u32), Comparison>,
    steps: usize,
}

impl Store {
    /// Only complete quantified clauses may compare modulo their local numbering.
    /// Ordinary/quantified alternatives stay distinct. Return charged work, and
    /// leave the original clause, binder arity and selected witnesses unchanged.
    pub(crate) fn same_clause(
        &mut self,
        left: Clause,
        right: Clause,
    ) -> Result<(bool, usize), &'static str> {
        let (Clause::ForAll(left), Clause::ForAll(right)) = (left, right) else {
            return Ok((left == right, 1));
        };
        let (left, work) = self.clause_comparison(left)?;
        let (right, more) = self.clause_comparison(right)?;
        Ok((left == right, work.saturating_add(more)))
    }

    pub(crate) fn clause_comparison(
        &mut self,
        id: QuantifiedId,
    ) -> Result<(Comparison, usize), &'static str> {
        self.clause_comparison_preserving(id, 0)
    }

    /// Appended supertrait binders may rename their locals; parent slots keep
    /// their meaning in the surrounding principal application.
    pub(crate) fn clause_comparison_preserving(
        &mut self,
        id: QuantifiedId,
        fixed: u32,
    ) -> Result<(Comparison, usize), &'static str> {
        if let Some(cache) = self
            .arguments
            .as_ref()
            .and_then(|storage| storage.clauses.as_ref())
            .and_then(|clauses| clauses.comparisons.as_ref())
            && let Some(&key) = if fixed == 0 {
                cache.keys.get(&id)
            } else {
                cache.preserved.get(&(id, fixed))
            }
        {
            return Ok((key, 1));
        }
        let value = self
            .quantified(id)
            .ok_or("invalid clause comparison identity")?;
        if fixed > value.bound_regions {
            return Err("fixed clause prefix exceeds its binder");
        }
        let (locals, work) = self.clause_local_slots(value.predicate)?;
        let (key, work) = if locals <= fixed {
            // Captures cannot change when only appended locals are renamed.
            // Reuse the occurrence index instead of walking a shared captured
            // graph once for every enclosing trait application.
            (Comparison(value.predicate), work)
        } else {
            let mut fold = Bound::canonical_locals_preserving(fixed);
            let key = Comparison(fold.predicate(self, value.predicate)?);
            (key, work.saturating_add(fold.steps))
        };
        let cache = self
            .arguments
            .as_mut()
            .and_then(|storage| storage.clauses.as_mut())
            .ok_or("missing clause comparison storage")?
            .comparisons
            .get_or_insert_with(Default::default);
        cache.steps = cache.steps.saturating_add(work);
        if cache
            .steps
            .saturating_add(cache.keys.len())
            .saturating_add(cache.preserved.len())
            > 1_000_000
        {
            return Err("clause comparison work budget exceeded");
        }
        if fixed == 0 {
            cache.keys.insert(id, key);
        } else {
            cache.preserved.insert((id, fixed), key);
        }
        Ok((key, work))
    }
}
