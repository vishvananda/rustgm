//! Commit a proved normal form through its original graph without equating projection inputs.
use super::super::{Context, Error, Store, TermId, TypeId, Value};

impl Context {
    /// A shape relation compares independently proved opaque normal forms. The
    /// existing directed lifetime proof must verify their complete input identity.
    pub(in crate::types::infer) fn opaque_shape(
        &mut self,
        store: &Store,
        left: TypeId,
        right: TypeId,
    ) -> Result<(), Error> {
        let mut shape = crate::types::structural::Normalized::<false>::new(store);
        let result = shape.require(left, right);
        self.account_work(shape.steps)?;
        result.map_err(|e| match e {
            crate::types::region_graph::Error::Budget => Error::Budget,
            _ => Error::Mismatch,
        })
    }

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
        if self.occurs(store, root, result)? {
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

    /// Record a finished search fold output's rigid projection roots. Every
    /// projection left in such an output is a normal form of that frame's
    /// search, so only the complete input rewrite calls this. Each root is
    /// this context's one term for that canonical type. Only the certified
    /// premise relation and the candidate header relation read the marks:
    /// every other operation, including `head`, `structural` and
    /// normalization waits, still defers them.
    pub(in crate::types::infer) fn retain_rigid_results(
        &mut self,
        store: &Store,
        ty: TypeId,
    ) -> Result<(), Error> {
        for root in self.projection_subterms(store, ty)? {
            let term = self.known(store, root)?;
            self.dependencies
                .get_or_insert_with(Default::default)
                .equations
                .rigid
                .insert(term, root);
        }
        Ok(())
    }

    /// The outermost projection subterms of a canonical type, in source order.
    /// A projection-free type costs nothing, as its interned flag says so; a
    /// projection itself needs no traversal; any other type is walked once in
    /// a fresh graph, so a term this context has already materialized cannot
    /// hide its identity, and the walk is charged to this context's work.
    pub(in crate::types::infer) fn projection_subterms(
        &mut self,
        store: &Store,
        ty: TypeId,
    ) -> Result<Vec<TypeId>, Error> {
        if !store
            .contains_projections(ty)
            .ok_or(Error::InvalidIdentity)?
        {
            return Ok(Vec::new());
        }
        self.step()?;
        let normalizable = match store.kind(ty).ok_or(Error::InvalidIdentity)? {
            crate::types::Kind::Projection(_) => true,
            crate::types::Kind::Opaque(id, _) => {
                store.opaque_declaration(id).is_some_and(|d| d.trait_result)
            }
            _ => false,
        };
        if normalizable {
            return Ok(vec![ty]);
        }
        let mut probe = Self::default();
        let found = probe.known_projection_roots(store, ty);
        self.account_work(probe.steps)?;
        found
    }

    fn known_projection_roots(&mut self, store: &Store, ty: TypeId) -> Result<Vec<TypeId>, Error> {
        let mut found = Vec::new();
        let mut work = vec![self.known(store, ty)?];
        while let Some(term) = work.pop() {
            self.step()?;
            // Every term of a fresh graph starts known; one already expanded
            // here was reached earlier through a shared canonical child.
            let Value::Known(ty) = self.terms[term.0 as usize].value else {
                continue;
            };
            if !store
                .contains_projections(ty)
                .ok_or(Error::InvalidIdentity)?
            {
                continue;
            }
            if self.materialize(store, term)?.normalizable(store) {
                found.push(ty);
                continue;
            }
            let entry = self.terms[term.0 as usize];
            self.account_work((entry.end - entry.start) as usize)?;
            work.extend(
                self.edges[entry.start as usize..entry.end as usize]
                    .iter()
                    .rev(),
            );
        }
        Ok(found)
    }

    pub(in crate::types::infer) fn retain_opaque_results(
        &mut self,
        store: &mut Store,
        root: TermId,
    ) -> Result<(), Error> {
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
