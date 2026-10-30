//! Numeric preferences follow only graphs already exported by ordinary publication.
use super::{Bound, Context, Error, Import, Refinement, Store, TermId};
use crate::support::{hash::HashSet, retained::Growth};
use crate::types::infer::Domain;

#[cfg(test)]
#[path = "../../../../tests/invariants/preference_exports.rs"]
mod tests;

impl Refinement {
    /// Map requested query floats through this publication's existing imports.
    /// This creates no inference roots, numeric watches or proof permission.
    /// Unconnected or already resolved roots supply no caller preference.
    /// The caller owns the preference's scope, lifetime and eventual cancellation.
    /// # Errors
    /// Retains the ordinary publication failure contract, including discarding
    /// the receiving context after a resource error during commit progress.
    pub(crate) fn publish_with_numeric_hints(
        &mut self,
        store: &mut Store,
        source: &mut Context,
        closing: &mut Bound,
        hints: &[TermId],
    ) -> Result<(bool, Vec<TermId>), Error> {
        self.publish_inputs(store, source, closing, hints)
    }
}

pub(super) fn validate_hints(
    query: &mut Context,
    source: &mut Context,
    hints: &[TermId],
) -> Result<(), Error> {
    if hints.is_empty() {
        return Ok(());
    }
    source.account_work(hints.len())?;
    for &hint in hints {
        query.root(hint)?;
    }
    Ok(())
}

pub(super) fn mapped_hints(
    query: &mut Context,
    validation: &mut Context,
    source: &mut Context,
    selected: &Import,
    reverse: &Import,
    hints: &[TermId],
) -> Result<Vec<TermId>, Error> {
    let mut result = Vec::new();
    let mut seen = HashSet::default();
    for &hint in hints {
        source.account_work(7)?;
        let hint = query.root(hint)?;
        if query.unresolved_domain(hint)? != Some(Domain::Float) {
            continue;
        }
        let Some(&mapped) = selected.types.get(&hint) else {
            continue;
        };
        let mapped = validation.root(mapped)?;
        let Some(&caller) = reverse.types.get(&mapped) else {
            continue;
        };
        let caller = source.root(caller)?;
        if source.unresolved_domain(caller)? != Some(Domain::Float) || seen.contains(&caller) {
            continue;
        }
        let vector = Growth::vector::<TermId>(result.len(), result.capacity(), 1);
        let set = Growth::map::<TermId, ()>(seen.len(), seen.capacity(), 1);
        source.account_work(vector.work.saturating_add(set.work).saturating_add(2))?;
        result.try_reserve(1).map_err(|_| Error::Overflow)?;
        seen.try_reserve(1).map_err(|_| Error::Overflow)?;
        seen.insert(caller);
        result.push(caller);
    }
    Ok(result)
}
