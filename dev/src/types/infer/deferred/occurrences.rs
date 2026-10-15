//! Normalize projected occurrences without waiting for unrelated output holes.
use super::super::{Constructor, Context, Error, Store, TermId, Value};
use crate::types::lifetime_proof::ScopeId;
use std::collections::HashSet;

impl Context {
    /// Find the present projected roots and retain their enclosing callable scopes.
    /// Projection inputs are normalized by that root's semantic query. An input
    /// that also occurs outside its projection receives its own independent root.
    /// Unknown variables introduce no roots; callers register later transformed
    /// or newly constructed graphs separately. This neither selects nor infers.
    ///
    /// # Errors
    /// Rejects invalid terms/scopes and exhausted inference work or binder storage.
    pub fn normalization_roots(
        &mut self,
        store: &Store,
        root: TermId,
        ambient: ScopeId,
        mut scope: impl FnMut(ScopeId, u32) -> Result<ScopeId, Error>,
    ) -> Result<Vec<(ScopeId, TermId)>, Error> {
        let mut roots = Vec::new();
        let mut work = vec![(ambient, root)];
        let mut seen = HashSet::new();
        while let Some((ambient, term)) = work.pop() {
            self.step()?;
            let term = self.root(term)?;
            if !seen.insert((ambient, term)) {
                continue;
            }
            match self.terms[term.0 as usize].value {
                Value::Variable(_) => continue,
                Value::Known(ty) if store.contains_projections(ty) == Some(false) => continue,
                _ => {}
            }
            let kind = self.materialize(store, term)?;
            if matches!(kind, Constructor::Projection(_)) {
                roots.push((ambient, term));
                continue;
            }
            let ambient = if let Constructor::Function { bound_regions, .. } = kind {
                scope(ambient, bound_regions)?
            } else {
                ambient
            };
            let entry = self.terms[term.0 as usize];
            self.account_work((entry.end - entry.start) as usize)?;
            work.extend(
                self.edges[entry.start as usize..entry.end as usize]
                    .iter()
                    .rev()
                    .map(|&child| (ambient, child)),
            );
        }
        Ok(roots)
    }
}
