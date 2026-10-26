//! Selected consumer signatures use one persistent cursor and actual table.
use super::{header_demands::State, template_demands};
use crate::{
    resolve::{
        crates::{ReservationWork, unpaid_provided::RecipeView},
        package::Error,
    },
    sema::{crates::typed_headers::recipe_cursor::Demand, model::Declarations},
    support::crates::owner_catalog::DeclarationRef,
};

/// A private append cursor, with no completed header, role or body authority.
#[derive(Default)]
pub(in crate::sema::crates::typed_headers) struct SelectedHeaders {
    recipes: Option<Box<State>>,
}
impl SelectedHeaders {
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
