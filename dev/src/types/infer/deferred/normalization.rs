//! Commit a proved normal form through its original graph without equating projection inputs.
use super::super::{Context, Error, Store, TermId, TypeId};

impl Context {
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
        self.terms[root.0 as usize].parent = result;
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
        match store.contains_projections(normalized) {
            Some(false) => {}
            Some(true) => return Err(Error::Normalization),
            None => return Err(Error::InvalidIdentity),
        }
        let result = self.known(store, normalized)?;
        self.pairs.clear();
        self.seen.clear();
        self.pairs.push((root, result));
        while let Some((original, result)) = self.pairs.pop() {
            self.step()?;
            let original = self.root(original)?;
            let result = self.root(result)?;
            if original == result || !self.seen.insert((original, result)) {
                continue;
            }
            if self.is_projection(store, original)? {
                self.terms[original.0 as usize].parent = result;
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
}
