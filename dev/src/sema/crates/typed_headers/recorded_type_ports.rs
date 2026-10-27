//! Only a sealed, actual defining body can resume its owned regional frontier.
use super::{Availability, Demand, Dependency, Port, Reserved, sealed};
use crate::{
    resolve::{
        body_origin::{ImportedBodyId, ImportedDeclarationId},
        crates::ReservationWork,
        model::DefId,
        package::Error,
    },
    sema::crates::typed_headers::unpaid_provided::template_demands::OriginalBody,
    support::crates::{generics::source as s, owner_catalog::DeclarationRef},
    types::{RegionId, Store, arguments::Argument, callable::Templates},
};

pub(in crate::sema::crates::typed_headers) struct Recorded<'rows, 'maps, 'body> {
    pub(in crate::sema::crates::typed_headers) port: Reserved<'rows, 'maps>,
    pub(in crate::sema::crates::typed_headers) body: &'body OriginalBody<'body>,
    pub(in crate::sema::crates::typed_headers) callables: &'body Templates,
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
        self.body
            .require_cursor(self.port.recipe_view(), store, self.callables, work)
    }
    fn resume_body(&self) -> Option<ImportedBodyId> {
        self.body.body().ok()
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
        self.body
            .region_recorded(self.port.recipe_view(), row, work)?
            .map_or_else(|| self.port.region(row, work), |id| Ok(Demand::Ready(id)))
    }
    fn application(
        &self,
        row: s::FunctionApplicationRef,
        work: &mut ReservationWork,
    ) -> Result<Dependency, Error> {
        self.port.application(row, work)
    }
}
