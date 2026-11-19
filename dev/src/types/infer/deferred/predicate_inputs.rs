//! A predicate's normalization plan owns its callable occurrences and replacement identities.
use super::{
    super::{Constructor, Context, Error, Store, TermId, TypeId, predicates::Predicate},
    rewrite::{Replacement, Scope},
};
use crate::support::hash::HashMap;
use crate::types::lifetime_proof::ScopeId;

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
        let mut callables = HashMap::default();
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

    /// Broaden an existential coherence question to every possible output of
    /// its unresolved projections. The original type graph stays available.
    pub(crate) fn abstract_coherence_projections(
        &mut self,
        store: &Store,
        predicate: &Predicate,
    ) -> Result<Option<Predicate>, Error> {
        let plan = self.projected_inputs(store, predicate)?;
        if plan.roots.is_empty() || plan.roots.iter().any(|&(scope, _)| scope != ScopeId::ROOT) {
            return Ok(None);
        }
        self.rewrite_inputs(store, predicate, &Existential(&plan))
            .map(Some)
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

    fn replacement(&self, key: Self::Key) -> Option<Replacement> {
        self.replacements
            .get(&key)
            .copied()
            .flatten()
            .map(Replacement::Normal)
    }
}

struct Existential<'a>(&'a ProjectionInputs);
impl Scope for Existential<'_> {
    type Key = (ScopeId, TermId);

    fn key(&self, scope: ScopeId, term: TermId) -> Self::Key {
        self.0.key(scope, term)
    }

    fn parts(&self, key: Self::Key) -> (ScopeId, TermId) {
        self.0.parts(key)
    }

    fn children(&self, key: Self::Key, kind: Constructor) -> Result<ScopeId, Error> {
        self.0.children(key, kind)
    }

    fn replacement(&self, key: Self::Key) -> Option<Replacement> {
        self.0
            .replacements
            .contains_key(&key)
            .then_some(Replacement::Existential)
    }
}
