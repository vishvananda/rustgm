//! Clause comparison forgets unused local slots, never captures or proof ownership.
use super::{Clause, QuantifiedId};
use crate::types::{Store, parameters::predicates::Predicate, substitute::Bound};
use std::collections::HashMap;
#[cfg(test)]
#[path = "../../../../tests/invariants/clause_equivalence.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Comparison(Predicate);

#[derive(Default)]
pub(super) struct Cache {
    keys: HashMap<QuantifiedId, Comparison>,
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
        if let Some(&key) = self
            .arguments
            .as_ref()
            .and_then(|storage| storage.clauses.as_ref())
            .and_then(|clauses| clauses.comparisons.as_ref())
            .and_then(|cache| cache.keys.get(&id))
        {
            return Ok((key, 1));
        }
        let value = self
            .quantified(id)
            .ok_or("invalid clause comparison identity")?;
        let mut fold = Bound::canonical_locals();
        let key = Comparison(fold.predicate(self, value.predicate)?);
        let work = fold.steps.saturating_add(1);
        let cache = self
            .arguments
            .as_mut()
            .and_then(|storage| storage.clauses.as_mut())
            .ok_or("missing clause comparison storage")?
            .comparisons
            .get_or_insert_with(Default::default);
        cache.steps = cache.steps.saturating_add(work);
        if cache.steps.saturating_add(cache.keys.len()) > 1_000_000 {
            return Err("clause comparison work budget exceeded");
        }
        cache.keys.insert(id, key);
        Ok((key, work))
    }
}
