//! Commit a proved normal form through its original graph without equating projection inputs.
use super::super::{Context, Error, Store, TermId, TypeId, Value};

impl Context {
    /// Retain an independent operation for an occurrence's formation proof.
    /// A determined application keeps its original canonical inputs even when
    /// another occurrence subsequently publishes a result through the shared graph.
    /// Unresolved applications retain their actual shared inference variables.
    /// # Errors
    /// Rejects non-projections, invalid identities and exhausted inference resources.
    pub fn retain_projection(&mut self, store: &mut Store, root: TermId) -> Result<TermId, Error> {
        let root = self.root(root)?;
        if !self.is_projection(store, root)? {
            return Err(Error::InvalidIdentity);
        }
        match self.resolved(store, root) {
            Ok(ty) => self.push(Value::Known(ty), &[]),
            Err(Error::Unresolved) => {
                let kind = self.materialize(store, root)?;
                let entry = self.terms[root.0 as usize];
                self.account_work((entry.end - entry.start) as usize)?;
                let children = self.edges[entry.start as usize..entry.end as usize].to_vec();
                self.construct(kind, &children)
            }
            Err(error) => Err(error),
        }
    }

    /// Publish a semantically selected symbolic result, retaining an independent
    /// application for the caller's eventual formation and implementation proof.
    /// The retained root shares every original input; it is not a second set of
    /// inference variables. Result equations may subsequently refine those inputs.
    ///
    /// # Errors
    /// Rejects non-projections, cyclic results and exhausted inference resources.
    pub fn publish_projection(
        &mut self,
        store: &Store,
        root: TermId,
        result: TermId,
    ) -> Result<TermId, Error> {
        let root = self.root(root)?;
        let result = self.root(result)?;
        if !self.is_projection(store, root)? {
            return Err(Error::InvalidIdentity);
        }
        if self.occurs(root, result)? {
            return Err(Error::Occurs);
        }
        let kind = self.materialize(store, root)?;
        let entry = self.terms[root.0 as usize];
        self.account_work((entry.end - entry.start) as usize)?;
        let children = self.edges[entry.start as usize..entry.end as usize].to_vec();
        let original = self.construct(kind, &children)?;
        self.link_root(root, result)?;
        self.normalized_projection(root)?;
        Ok(original)
    }

    /// Install the result of semantic normalization of one fully determined root.
    /// The caller must first prove every projection in its actual binder scope.
    /// Only associated-result nodes may change; ordinary structure, const inputs
    /// and lifetime spellings must remain exactly the same. No holes are inferred.
    ///
    /// # Errors
    /// Rejects unresolved inputs/results, changed ordinary structure and exhausted work.
    pub fn apply_normalization(
        &mut self,
        store: &mut Store,
        root: TermId,
        normalized: TypeId,
    ) -> Result<(), Error> {
        self.resolved(store, root)?;
        store.kind(normalized).ok_or(Error::InvalidIdentity)?;
        let result = self.known(store, normalized)?;
        self.retain_opaque_results(store, result)?;
        self.pairs.clear();
        self.seen.clear();
        self.pairs.push((root, result));
        while let Some((original, result)) = self.pairs.pop() {
            self.step()?;
            let original = self.root(original)?;
            let result = self.root(result)?;
            if original == result {
                if self.opaque_projection(result).is_some() {
                    self.normalized_projection(original)?;
                }
                continue;
            }
            if !self.seen.insert((original, result)) {
                continue;
            }
            if self.is_projection(store, original)? {
                if self.opaque_projection(original).is_some() {
                    self.structural(store, original, result, false)?;
                    continue;
                }
                self.link_root(original, result)?;
                self.normalized_projection(original)?;
            } else {
                self.structural(store, original, result, false)?;
            }
        }
        if self.resolved(store, root)? != normalized {
            return Err(Error::Mismatch);
        }
        Ok(())
    }

    fn retain_opaque_results(&mut self, store: &mut Store, root: TermId) -> Result<(), Error> {
        if let Value::Known(ty) = self.terms[root.0 as usize].value
            && store.contains_projections(ty) == Some(false)
        {
            return Ok(());
        }
        let roots = self.normalization_roots(
            store,
            root,
            crate::types::lifetime_proof::ScopeId::ROOT,
            |parent, _| Ok(parent),
        )?;
        for (_, term) in roots {
            let ty = self.resolved(store, term)?;
            if !self.is_projection(store, term)? {
                return Err(Error::InvalidIdentity);
            }
            self.dependencies
                .get_or_insert_with(Default::default)
                .equations
                .opaque
                .insert(term, ty);
        }
        Ok(())
    }
}
