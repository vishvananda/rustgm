//! Code selects real recipes from the same initialized owner before ordinary loans.
use super::{Error, Initial, Prepared, Verifier};
use crate::borrow::{destruction, ownership::initialization, regions};

impl Verifier {
    pub(crate) fn select_execution<'a, S: std::hash::BuildHasher>(
        &mut self,
        environment: &crate::borrow::Environment<'_, S>,
        mut initial: Initial<'a>,
        contract: &regions::Body,
        catalog: &mut destruction::glue::Catalog<'_>,
    ) -> Result<Prepared<'a, destruction::Plan>, Error> {
        let body = initial.body();
        let error = |message: &str| Error {
            node: Some(body.source.into()),
            message: message.into(),
        };
        initial
            .require(environment, contract, self.owner.as_ref())
            .map_err(error)?;
        let refinement = initial
            .facts
            .refine_variants(
                environment.types,
                environment.definitions,
                &|item| catalog.custom(item),
                initialization::Refinement::EligibilityOnly,
                &|ty, query| catalog.member_position(ty, query),
            )
            .map_err(error)?;
        let mut plan =
            destruction::select_reusing_defining(catalog, &mut initial.facts, &mut Vec::new())
                .map_err(|error| Error {
                    node: Some(error.node),
                    message: error.message.into(),
                })?;
        plan.steps = plan
            .steps
            .checked_add(refinement)
            .filter(|n| *n <= 32_000_000)
            .ok_or_else(|| error("destruction planning work budget exceeded"))?;
        Ok(initial.select(Some(plan)))
    }
}
