//! Only a sealed, actual defining body can resume its owned regional frontier.
use super::{Availability, Demand, Dependency, Port, Reserved, sealed};
use crate::{
    resolve::{
        body_origin::{ImportedBodyId, ImportedDeclarationId},
        crates::ReservationWork,
        model::DefId,
        package::Error,
    },
    sema::crates::typed_headers::unpaid_provided::{OrdinaryDomain, OriginalBody},
    support::crates::{generics::source as s, owner_catalog::DeclarationRef},
    types::{RegionId, Store, arguments::Argument, callable::Templates},
};
#[path = "applied_records.rs"]
mod applications;

pub(in crate::sema::crates::typed_headers) struct Recorded<'rows, 'maps, 'body> {
    pub(in crate::sema::crates::typed_headers) port: Reserved<'rows, 'maps>,
    pub(in crate::sema::crates::typed_headers) body: Domain<'body>,
    pub(in crate::sema::crates::typed_headers) callables: &'body Templates,
}
#[derive(Clone, Copy)]
pub(in crate::sema::crates::typed_headers) enum Domain<'a> {
    Selected(&'a OriginalBody<'a>),
    Ordinary(&'a OrdinaryDomain),
}
impl<'rows, 'maps, 'body> Recorded<'rows, 'maps, 'body> {
    pub(in crate::sema::crates::typed_headers) fn ordinary(
        port: Reserved<'rows, 'maps>,
        body: &'body OrdinaryDomain,
        callables: &'body Templates,
    ) -> Self {
        Self {
            port,
            body: Domain::Ordinary(body),
            callables,
        }
    }
    pub(in crate::sema::crates::typed_headers) fn selected(
        port: Reserved<'rows, 'maps>,
        body: &'body OriginalBody<'body>,
        callables: &'body Templates,
    ) -> Self {
        Self {
            port,
            body: Domain::Selected(body),
            callables,
        }
    }
}
impl sealed::Sealed for Recorded<'_, '_, '_> {}
impl<'rows> Port<'rows, ReservationWork> for Recorded<'rows, '_, '_> {
    fn rows(&self) -> s::ValidatedView<'rows> {
        self.port.rows()
    }
    fn definition(&self, row: DeclarationRef, work: &mut ReservationWork) -> Result<DefId, Error> {
        self.port.definition(row, work)
    }
    fn root(&self) -> ImportedDeclarationId {
        self.port.root()
    }
    fn availability(&self) -> Availability {
        Availability::DeclaredSignatures
    }
    fn require_context(&self, store: &Store, work: &mut ReservationWork) -> Result<(), Error> {
        match self.body {
            Domain::Selected(body) => {
                body.require_cursor(self.port.recipe_view(), store, self.callables, work)
            }
            Domain::Ordinary(body) => {
                body.require(self.port.recipe_view(), store, self.callables, work)
            }
        }
    }
    fn resume_body(&self) -> Option<ImportedBodyId> {
        match self.body {
            Domain::Selected(body) => Some(body.body()),
            Domain::Ordinary(body) => body.body().ok(),
        }
    }
    fn slot(
        &mut self,
        store: &mut Store,
        row: s::SlotRef,
        work: &mut ReservationWork,
    ) -> Result<Argument, Error> {
        self.port.slot(store, row, work)
    }
    fn region(
        &mut self,
        row: s::RegionRef,
        work: &mut ReservationWork,
    ) -> Result<Demand<RegionId>, Error> {
        // The caller authenticates the Store before every demand, including warm hits.
        // Variables are read from the selected record, never the global region map.
        let region = match self.body {
            Domain::Selected(body) => body.region_recorded(self.port.recipe_view(), row, work)?,
            Domain::Ordinary(body) => body.recorded_region(self.port.recipe_view(), row, work)?,
        };
        region.map_or_else(|| self.port.region(row, work), |id| Ok(Demand::Ready(id)))
    }
    fn application(
        &self,
        row: s::FunctionApplicationRef,
        work: &mut ReservationWork,
    ) -> Result<Dependency, Error> {
        self.port.application(row, work)
    }
    fn mapped_applications(&self) -> bool {
        matches!(self.body, Domain::Ordinary(_))
    }
    fn map_application(
        &mut self,
        store: &mut Store,
        row: s::Type,
        mapped: &dyn Fn(
            crate::support::crates::generics::TypeRef,
        ) -> Result<crate::types::TypeId, Error>,
        work: &mut ReservationWork,
    ) -> Result<crate::types::TypeId, Error> {
        self.require_context(store, work)?;
        applications::map(self, store, row, mapped, work)
    }
}
