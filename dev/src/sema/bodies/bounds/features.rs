//! A call's non-type obligation is independent of unresolved type arguments.
use super::super::{Checker, Error, NodeId};
use crate::types::parameters::features::FeatureId;
impl Checker<'_> {
    pub(in crate::sema::bodies) fn prove_feature_bound(
        &mut self,
        node: NodeId,
        id: FeatureId,
    ) -> Result<(), Error> {
        let environment = self.trait_query_environment(node)?;
        let work = std::cell::Cell::new(0_usize);
        let result = self
            .facts
            .prove_feature(id, environment.as_deref(), &|steps, _| {
                work.set(work.get().saturating_add(steps));
                Ok(())
            });
        self.facts.body_work.canonical_steps = self
            .facts
            .body_work
            .canonical_steps
            .saturating_add(work.get());
        if !result.map_err(|error| self.error(node, error))? {
            return Err(self.error(
                node,
                "non-type feature requirement is not satisfied by this caller",
            ));
        }
        Ok(())
    }
}
