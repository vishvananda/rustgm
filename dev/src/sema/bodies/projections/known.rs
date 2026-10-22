//! Known declaration roots retain one projection cursor across exact library readiness.
use super::{Checker, Error, NodeId, Obligations, ScopeId, TypeId};
impl Checker<'_> {
    pub(in crate::sema::bodies) fn normalize_known_projection(
        &mut self,
        node: NodeId,
        ty: TypeId,
    ) -> Result<TypeId, Error> {
        // A canonical declaration type already has determined inputs. Keep its
        // written body lifetimes while proving its complete nested binder tree;
        // replacing the whole type with caller aliases would change its return contract.
        let first = self.facts.body_regions.frontier(self.state.body)
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
                crate::sema::projections::Progress::Complete { ty, steps } => break (ty, steps),
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
                    return Err(self.error(
                        node,
                        "projection selection requires a prepared implementation header",
                    ));
                }
                crate::sema::projections::Progress::Definition(_) => {
                    return Err(self.error(node, "missing completed associated type definition"));
                }
            }
        };
        self.state
            .infer
            .account_work(steps)
            .map_err(|e| self.error(node, e))?;
        proof
            .retain_body(self.facts, self.state.body, self.key(node))
            .map_err(|e| self.error(node, e))?;
        Ok(ty)
    }
}
