//! Selected consumer signatures use one persistent cursor and actual table.
use super::{SchemaAuthority, header_demands::State, owner_stages, template_demands};
use crate::{
    resolve::{
        crates::{ReservationWork, unpaid_provided::RecipeView},
        package::Error,
    },
    sema::{crates::typed_headers::recipe_cursor::Demand, model::Declarations},
    support::crates::owner_catalog::DeclarationRef,
};

#[path = "declared_headers.rs"]
mod declarations;

/// A private append cursor, with no completed header, role or body authority.
#[derive(Default)]
pub(in crate::sema::crates::typed_headers) struct SelectedHeaders {
    recipes: Option<Box<State>>,
    owners: Option<Box<owner_stages::OwnerTransaction>>,
}
impl SelectedHeaders {
    pub(in crate::sema::crates::typed_headers) fn trait_owner(
        &mut self,
        facts: &mut Declarations,
        authority: &SchemaAuthority<'_>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<Demand<()>, Error> {
        if let Demand::Blocked(dependency) =
            declarations::prepare(self, facts, authority, owner, work)?
        {
            return Ok(Demand::Blocked(dependency));
        }
        owner_stages::install(
            &mut self.owners,
            &mut self.recipes,
            facts,
            authority,
            owner,
            work,
        )
    }
    pub(in crate::sema::crates::typed_headers) fn trait_declaration(
        &mut self,
        facts: &mut Declarations,
        authority: &SchemaAuthority<'_>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<Demand<()>, Error> {
        declarations::prepare(self, facts, authority, owner, work)
    }
    pub(in crate::sema::crates::typed_headers) fn schema(
        &mut self,
        facts: &mut Declarations,
        authority: &SchemaAuthority<'_>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<Demand<()>, Error> {
        owner_stages::prepare(
            &mut self.owners,
            &mut self.recipes,
            facts,
            authority,
            owner,
            work,
        )
    }
    pub(in crate::sema::crates::typed_headers) fn callable<'a>(
        &mut self,
        facts: &mut Declarations,
        owner: DeclarationRef,
        work: &mut ReservationWork,
        view: impl FnMut(&mut ReservationWork) -> Result<RecipeView<'a>, Error>,
    ) -> Result<Demand<()>, Error> {
        template_demands::prepare(
            &mut self.recipes,
            &mut facts.types,
            Some(&mut facts.lifetimes.callables),
            owner,
            work,
            view,
        )
    }
}
