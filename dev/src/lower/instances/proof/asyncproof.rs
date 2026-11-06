//! The actual selected source application completes loans before generated poll mutation.
use super::{Builder, Declarations, borrow, templates};
use crate::body::instantiate::Materialized;

impl Builder {
    pub(in crate::lower::instances) fn prove_future_application(
        &mut self,
        facts: &mut Declarations,
        templates: &templates::Checked<'_>,
        application: &Materialized,
    ) -> Result<(borrow::incremental::Completed, borrow::regions::Body), borrow::Error> {
        let error = |message: &str| borrow::Error {
            node: Some(application.body.source.into()),
            message: message.into(),
        };
        self.require_current_store(&facts.types).map_err(error)?;
        facts
            .charge_caller_inputs(
                1,
                size_of::<crate::body::regions::Contract>()
                    .saturating_add(application.contract.retained_bytes())
                    .saturating_add(size_of::<borrow::destruction::Plan>())
                    .saturating_add(size_of::<borrow::incremental::Completed>())
                    .saturating_add(size_of::<borrow::loans::seal::Seal>()),
            )
            .map_err(error)?;
        let mut input = application.contract.clone();
        if facts.uses_source_copy() {
            application
                .prepare_source_copy(facts, &application.body, &mut input)
                .map_err(|message| error(&message))?;
        }
        let regions = self.regions(facts, &application.body, &input)?;
        if facts.aggregates.existing_arena_identity().is_none() {
            facts
                .charge_caller_inputs(1, crate::types::arena::Identity::allocation_bytes())
                .map_err(error)?;
            facts.aggregates.arena_identity();
        }
        let cache = self
            .recipes
            .take()
            .ok_or_else(|| error("future poll requires its actual cleanup catalog"))?;
        let mut catalog = cache.attach(&facts.types, &facts.aggregates, &facts.members);
        let mut environment = borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            source_copy: facts.uses_source_copy(),
            copies: &mut facts.copies,
        };
        let completed = self.verifier.prove_async_application(
            &mut environment,
            templates,
            &application.body,
            &input,
            &regions,
            &mut catalog,
        );
        self.recipes = Some(catalog.into_cache());
        Ok((completed?, regions))
    }
}
