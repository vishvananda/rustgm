//! Build normalized input views while keeping original formation roots and shared holes.
use super::super::{Constructor, Context, Error, Store, TermId, TypeId, Value};
use crate::support::hash::HashMap;
use crate::types::lifetime_proof::ScopeId;
use std::hash::Hash;

struct Rewriting<K> {
    results: HashMap<K, TermId>,
    work: Vec<(K, bool)>,
    children: Vec<TermId>,
}
impl<K> Default for Rewriting<K> {
    fn default() -> Self {
        Self {
            results: HashMap::default(),
            work: Vec::new(),
            children: Vec::new(),
        }
    }
}

pub(super) trait Scope {
    const COMPLETE: bool = false;
    type Key: Copy + Eq + Hash;
    fn key(&self, scope: ScopeId, term: TermId) -> Self::Key;
    fn parts(&self, key: Self::Key) -> (ScopeId, TermId);
    fn children(&self, key: Self::Key, kind: Constructor) -> Result<ScopeId, Error>;
    fn replacement(&self, key: Self::Key) -> Option<TypeId>;
}

impl Scope for HashMap<TermId, TypeId> {
    type Key = TermId;

    fn key(&self, _scope: ScopeId, term: TermId) -> Self::Key {
        term
    }

    fn parts(&self, key: Self::Key) -> (ScopeId, TermId) {
        (ScopeId::ROOT, key)
    }

    fn children(&self, _key: Self::Key, _kind: Constructor) -> Result<ScopeId, Error> {
        Ok(ScopeId::ROOT)
    }

    fn replacement(&self, key: Self::Key) -> Option<TypeId> {
        self.get(&key).copied()
    }
}

impl Context {
    /// Share one transformed graph across every input of a predicate. Inference
    /// variables and original projection roots remain independent of this view.
    /// # Errors
    /// Rejects invalid replacements, equality predicates and exhausted work.
    pub fn rewritten_projection_inputs(
        &mut self,
        store: &Store,
        predicate: &super::super::predicates::Predicate,
        replacements: &HashMap<TermId, TypeId>,
    ) -> Result<super::super::predicates::Predicate, Error> {
        self.rewrite_inputs(store, predicate, replacements)
    }

    pub(super) fn rewrite_inputs<S: Scope>(
        &mut self,
        store: &Store,
        predicate: &super::super::predicates::Predicate,
        replacements: &S,
    ) -> Result<super::super::predicates::Predicate, Error> {
        let mut rewriting = Rewriting::default();
        predicate.map_inputs(|root| {
            self.rewrite_projection_root(store, root, replacements, &mut rewriting)
        })
    }
    /// Replace independently proved projection roots without mutating the original
    /// input graph. Inference variables and const/lifetime identities remain shared.
    /// The caller owns each replacement's normalization and formation evidence.
    ///
    /// # Errors
    /// Rejects invalid identities, non-projection replacements and exhausted work.
    pub fn rewritten_projections(
        &mut self,
        store: &Store,
        root: TermId,
        replacements: &HashMap<TermId, TypeId>,
    ) -> Result<TermId, Error> {
        self.rewrite_projection_root(store, root, replacements, &mut Rewriting::default())
    }
    fn rewrite_projection_root<S: Scope>(
        &mut self,
        store: &Store,
        root: TermId,
        replacements: &S,
        rewriting: &mut Rewriting<S::Key>,
    ) -> Result<TermId, Error> {
        let root = self.root(root)?;
        let Rewriting {
            results,
            work,
            children,
        } = rewriting;
        let root = replacements.key(ScopeId::ROOT, root);
        work.push((root, false));
        while let Some((key, finish)) = work.pop() {
            self.step()?;
            let (scope, id) = replacements.parts(key);
            let id = self.root(id)?;
            let key = replacements.key(scope, id);
            if results.contains_key(&key) {
                continue;
            }
            if let Some(ty) = replacements.replacement(key) {
                if !self.is_projection(store, id)? {
                    return Err(Error::InvalidIdentity);
                }
                results.insert(key, self.known(store, ty)?);
                continue;
            }
            match self.terms[id.0 as usize].value {
                Value::Variable(_) => {
                    results.insert(key, id);
                    continue;
                }
                Value::Known(ty) if store.contains_projections(ty) == Some(false) => {
                    results.insert(key, id);
                    continue;
                }
                _ => {}
            }
            let kind = self.materialize(store, id)?;
            if S::COMPLETE && kind.normalizable(store) {
                return Err(Error::Normalization);
            }
            let scope = replacements.children(key, kind)?;
            let entry = self.terms[id.0 as usize];
            self.account_work((entry.end - entry.start) as usize)?;
            if !finish {
                work.push((key, true));
                work.extend(
                    self.edges[entry.start as usize..entry.end as usize]
                        .iter()
                        .rev()
                        .map(|&child| (replacements.key(scope, child), false)),
                );
                continue;
            }
            children.clear();
            let mut changed = false;
            for offset in entry.start..entry.end {
                let child = self.root(self.edges[offset as usize])?;
                let rewritten = *results
                    .get(&replacements.key(scope, child))
                    .ok_or(Error::InvalidIdentity)?;
                changed |= child != rewritten;
                children.push(rewritten);
            }
            let result = if changed {
                self.construct(kind, children)?
            } else {
                id
            };
            results.insert(key, result);
        }
        results.get(&root).copied().ok_or(Error::InvalidIdentity)
    }
}
