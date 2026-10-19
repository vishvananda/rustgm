//! A predicate's normalization plan owns its callable occurrences and replacement identities.
use super::super::{Constructor, Context, Error, Store, TermId, TypeId, predicates::Predicate};
use crate::types::lifetime_proof::ScopeId;
use std::collections::HashMap;

/// Structural input rewriting retains the original inference graph. Every
/// replacement belongs to one callable occurrence; this plan grants no proof.
pub struct ProjectionInputs {
    original: Predicate,
    roots: Vec<(ScopeId, TermId)>,
    binders: Vec<(ScopeId, u32)>,
    callables: HashMap<(ScopeId, TermId), ScopeId>,
    replacements: HashMap<(ScopeId, TermId), Option<TypeId>>,
}

impl Context {
    /// Collect a predicate's projected occurrences with their complete binder tree.
    /// Shared terms in different scopes require independent normalization evidence.
    /// # Errors
    /// Rejects malformed terms, invalid scopes and exhausted traversal work.
    pub fn projected_inputs(
        &mut self,
        store: &Store,
        predicate: &Predicate,
    ) -> Result<ProjectionInputs, Error> {
        let mut binders = Vec::new();
        let mut callables = HashMap::new();
        let roots =
            self.predicate_roots(store, predicate, ScopeId::ROOT, |parent, term, count| {
                binders.push((parent, count));
                let scope = ScopeId(u32::try_from(binders.len()).map_err(|_| Error::Overflow)?);
                callables.insert((parent, term), scope);
                Ok(scope)
            })?;
        self.account_work(roots.len())?;
        let replacements = roots.iter().map(|&key| (key, None)).collect();
        Ok(ProjectionInputs {
            original: predicate.map_inputs(Ok)?,
            roots,
            binders,
            callables,
            replacements,
        })
    }
}

impl ProjectionInputs {
    #[must_use]
    pub fn original(&self) -> &Predicate {
        &self.original
    }

    #[must_use]
    pub fn roots(&self) -> &[(ScopeId, TermId)] {
        &self.roots
    }

    #[must_use]
    pub fn binder(&self, scope: ScopeId) -> Option<(ScopeId, u32)> {
        scope
            .0
            .checked_sub(1)
            .and_then(|index| self.binders.get(index as usize))
            .copied()
    }

    /// Record a normal form in the original root's lexical binder context.
    /// The semantic caller must retain its formation and normalization evidence.
    /// # Errors
    /// Rejects foreign occurrences and conflicting replacement identities.
    pub fn replace(&mut self, scope: ScopeId, term: TermId, ty: TypeId) -> Result<(), Error> {
        let value = self
            .replacements
            .get_mut(&(scope, term))
            .ok_or(Error::InvalidIdentity)?;
        if value.is_some_and(|previous| previous != ty) {
            return Err(Error::InvalidIdentity);
        }
        *value = Some(ty);
        Ok(())
    }

    /// Rebuild completed input views while preserving shared unresolved holes.
    /// The original predicate and projection roots remain unchanged.
    /// # Errors
    /// Rejects incomplete replacement sets, changed graphs and exhausted work.
    pub fn rewrite(&self, context: &mut Context, store: &Store) -> Result<Predicate, Error> {
        context.account_work(self.replacements.len())?;
        if self.replacements.values().any(Option::is_none) {
            return Err(Error::Normalization);
        }
        context.rewrite_inputs(store, &self.original, self)
    }
}

impl super::rewrite::Scope for ProjectionInputs {
    const COMPLETE: bool = true;
    type Key = (ScopeId, TermId);

    fn key(&self, scope: ScopeId, term: TermId) -> Self::Key {
        (scope, term)
    }

    fn parts(&self, key: Self::Key) -> (ScopeId, TermId) {
        key
    }

    fn children(&self, key: Self::Key, kind: Constructor) -> Result<ScopeId, Error> {
        if matches!(kind, Constructor::Function { .. }) {
            self.callables
                .get(&key)
                .copied()
                .ok_or(Error::InvalidIdentity)
        } else {
            Ok(key.0)
        }
    }

    fn replacement(&self, key: Self::Key) -> Option<TypeId> {
        self.replacements.get(&key).copied().flatten()
    }
}
