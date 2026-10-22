//! Normalize projected occurrences without waiting for unrelated output holes.
use super::super::{Constructor, Context, Error, Store, TermId, Value};
use crate::support::hash::HashSet;
use crate::types::lifetime_proof::ScopeId;

impl Context {
    /// Traverse all predicate inputs together so repeated root DAGs share work.
    /// The source input order is retained, including each nested callable scope.
    /// # Errors
    /// Rejects malformed input identities, scopes and exhausted traversal work.
    pub fn predicate_normalization_roots(
        &mut self,
        store: &Store,
        predicate: &super::super::predicates::Predicate,
        ambient: ScopeId,
        mut scope: impl FnMut(ScopeId, u32) -> Result<ScopeId, Error>,
    ) -> Result<Vec<(ScopeId, TermId)>, Error> {
        self.predicate_roots(store, predicate, ambient, |parent, _, count| {
            scope(parent, count)
        })
    }

    pub(super) fn predicate_roots(
        &mut self,
        store: &Store,
        predicate: &super::super::predicates::Predicate,
        ambient: ScopeId,
        scope: impl FnMut(ScopeId, TermId, u32) -> Result<ScopeId, Error>,
    ) -> Result<Vec<(ScopeId, TermId)>, Error> {
        let mut work = predicate
            .input_roots()
            .map(|root| (ambient, root))
            .collect::<Vec<_>>();
        self.account_work(work.len())?;
        work.reverse();
        self.collect_normalization_roots(store, work, scope)
    }
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
        let root = self.root(root)?;
        let value = self.terms[root.0 as usize].value;
        if matches!(value, Value::Variable(_))
            || matches!(value, Value::Known(ty) if store.contains_projections(ty) == Some(false))
        {
            // Preserve the traversal charge without allocating a worklist or
            // visited set for the common case with no present projection.
            self.step()?;
            return Ok(Vec::new());
        }
        self.collect_normalization_roots(store, vec![(ambient, root)], |parent, _, count| {
            scope(parent, count)
        })
    }

    /// Find projected input boundaries which must expose their results before
    /// the enclosing application can match a concrete implementation header.
    /// # Errors
    /// Rejects malformed applications, invalid scopes and exhausted inference work.
    pub fn normalization_input_roots(
        &mut self,
        store: &Store,
        root: TermId,
        ambient: ScopeId,
        scope: impl FnMut(ScopeId, TermId, u32) -> Result<ScopeId, Error>,
    ) -> Result<Vec<(ScopeId, TermId)>, Error> {
        let root = self.root(root)?;
        if !self.materialize(store, root)?.normalizable(store) {
            return Err(Error::InvalidIdentity);
        }
        let entry = self.terms[root.0 as usize];
        self.account_work((entry.end - entry.start) as usize)?;
        let work = self.edges[entry.start as usize..entry.end as usize]
            .iter()
            .rev()
            .map(|&term| (ambient, term))
            .collect();
        self.collect_normalization_roots(store, work, scope)
    }

    fn collect_normalization_roots(
        &mut self,
        store: &Store,
        mut work: Vec<(ScopeId, TermId)>,
        mut scope: impl FnMut(ScopeId, TermId, u32) -> Result<ScopeId, Error>,
    ) -> Result<Vec<(ScopeId, TermId)>, Error> {
        let mut roots = Vec::new();
        let mut seen = HashSet::default();
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
            if kind.normalizable(store) {
                roots.push((ambient, term));
                continue;
            }
            let ambient = if let Constructor::Function { bound_regions, .. } = kind {
                scope(ambient, term, bound_regions)?
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
