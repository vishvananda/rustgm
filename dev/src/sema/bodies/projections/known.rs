//! Known declaration roots retain one projection cursor across exact library readiness.
use super::{Checker, Error, NodeId, Obligations, ScopeId, TypeId};
type KnownProjection = Result<(TypeId, Obligations), &'static str>;

impl Checker<'_> {
    pub(in crate::sema::bodies) fn normalize_known_projection(
        &mut self,
        node: NodeId,
        ty: TypeId,
    ) -> Result<TypeId, Error> {
        let (ty, proof) = self
            .query_known_projection(node, ty)?
            .map_err(|error| self.error(node, error))?;
        proof
            .retain_body(self.facts, self.state.body, self.key(node))
            .map_err(|e| self.error(node, e))?;
        Ok(ty)
    }

    pub(in crate::sema::bodies) fn projection_result_has_regions(
        &mut self,
        node: NodeId,
        ty: TypeId,
    ) -> Result<bool, Error> {
        let result = self.query_known_projection(node, ty)?;
        let types = &self.facts.types;
        Ok(result.is_ok_and(|(ty, _)| {
            types.contains_projections(ty) == Some(false)
                && types.has_free_regions(ty, 0) == Some(true)
        }))
    }

    fn query_known_projection(
        &mut self,
        node: NodeId,
        ty: TypeId,
    ) -> Result<KnownProjection, Error> {
        // A canonical declaration type already has determined inputs. Keep its
        // written body lifetimes while proving its complete nested binder tree;
        // replacing the whole type with caller aliases would change its return contract.
        let first = self
            .facts
            .body_regions
            .frontier(self.state.body)
            .map_err(|e| self.error(node, e))?;
        let mut proof = Obligations::new(first);
        // Call arguments already use free aliases for caller universals. Their
        // environment must use the same representation, without duplicate
        // original/aliased assumptions. Written body types keep their binders.
        let environment = self.resolved_projection_environment(node, ty)?;
        let mut cursor = crate::sema::projections::Cursor::new(
            environment,
            &proof,
            ScopeId::ROOT,
            self.availability,
        )
        .map_err(|e| self.error(node, e))?;
        cursor
            .start(self.facts, ty)
            .map_err(|e| self.error(node, e))?;
        let (ty, steps) = loop {
            match cursor
                .advance(self.facts, &mut proof, &mut |_, _| Ok(None))
                .map_err(|e| self.error(node, e))?
            {
                crate::sema::projections::Progress::Complete { ty, steps, .. } => {
                    break (ty, steps);
                }
                crate::sema::projections::Progress::Dependency(
                    crate::sema::traits::selection::Dependency::Header(owner),
                ) if crate::sema::declarations::iteration::range_contracts::owns_header(
                    self.report,
                    owner,
                ) =>
                {
                    crate::sema::declarations::iteration::range_contracts::prepare(
                        self.report,
                        self.facts,
                        owner,
                        matches!(
                            self.availability,
                            crate::sema::traits::implementations::Availability::Validated
                        ),
                    )?;
                    let first = cursor
                        .allocation_end(&proof)
                        .max(self.trait_region_start(node)?);
                    cursor
                        .resume_at(&mut proof, first)
                        .map_err(|e| self.error(node, e))?;
                    self.state
                        .infer
                        .account_work(1)
                        .map_err(|e| self.error(node, e))?;
                }
                crate::sema::projections::Progress::Dependency(_) => {
                    return Ok(Err(
                        "projection selection requires a prepared implementation header",
                    ));
                }
                crate::sema::projections::Progress::Definition(_) => {
                    return Ok(Err("missing completed associated type definition"));
                }
            }
        };
        self.state
            .infer
            .account_work(steps)
            .map_err(|e| self.error(node, e))?;
        Ok(Ok((ty, proof)))
    }
}
