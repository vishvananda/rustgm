//! A selected declaration lends actual raw body inputs only after their producers.
use super::{SelectedHeaders, template_demands};
use crate::{
    resolve::{crates::ReservationWork, model::DefId, package::Error},
    sema::{
        crates::{
            ImportedOwners, SourceOriginal,
            typed_headers::{error, recipe_cursor::Demand, unpaid_provided::SchemaAuthority},
        },
        model::Declarations,
    },
    support::crates::owner_catalog::DeclarationRef,
};
impl SelectedHeaders {
    pub(in crate::sema::crates::typed_headers) fn with_original(
        &mut self,
        facts: &mut Declarations,
        authority: &SchemaAuthority<'_>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
        consume: &mut dyn FnMut(&Declarations, &SourceOriginal<'_>) -> Result<(), Error>,
    ) -> Result<Demand<()>, Error> {
        if let Demand::Blocked(dependency) =
            self.method_declaration(facts, authority, owner, work)?
        {
            return Ok(Demand::Blocked(dependency));
        }
        let view = authority.reservation.recipe_view(authority.report, work)?;
        let definition: DefId = view.definition(owner, work)?;
        if let Demand::Blocked(dependency) =
            template_demands::defining_body(&mut self.recipes, facts, owner, work, |work| {
                authority.reservation.retained_recipe_view(work)
            })?
        {
            return Ok(Demand::Blocked(dependency));
        }
        let state = self
            .recipes
            .as_deref()
            .ok_or_else(|| error("selected original lost its actual recipe owner"))?;
        let Some(ImportedOwners::Provided(mapping)) = facts.associated.imported_owner() else {
            return Err(error("selected original lost its actual installed mapping"));
        };
        mapping.require_source(authority, state, &facts.types, work)?;
        let prepared = state
            .callables
            .as_deref()
            .ok_or_else(|| error("selected original lost its actual callable owner"))?;
        let body = prepared.original_body(&view, facts, owner, work)?;
        body.require(&view, facts, work)?;
        let mut steps = work.steps;
        let result = SourceOriginal::selected(
            authority.reservation.catalog(),
            mapping,
            facts,
            &body,
            &view,
            definition,
            &mut steps,
        )
        .map_err(error);
        work.charge(steps.saturating_sub(work.steps))?;
        consume(facts, &result?)?;
        Ok(Demand::Ready(()))
    }
}
