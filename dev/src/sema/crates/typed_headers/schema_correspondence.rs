//! Actual consumer authority lends the same source rows and canonical memo.
use super::super::{
    error,
    mapped_source::checked_rows::{self, Mapping, MemberRows},
    recipe_cursor::{Maps, MemoLookup, MemoReader},
};
use crate::{
    resolve::{
        body_origin::Catalog,
        crates::{ConsumerReservation, ReservationWork, unpaid_provided::RecipeView},
        expansion::Report,
        model::DefKind,
        package::Error,
    },
    sema::crates::authored_export::CollectedReceipt,
    support::crates::{
        SourceDescriptor,
        generics::{self as g, source as s},
    },
    types::{RegionId, Store, TypeId},
};

/// These are borrowed from the one move-only current consumer, never producer facts.
pub(in crate::sema::crates::typed_headers) struct Authority<'a> {
    pub(in crate::sema::crates::typed_headers) report: &'a Report,
    pub(in crate::sema::crates::typed_headers) reservation: &'a ConsumerReservation,
    pub(in crate::sema::crates::typed_headers) completion: &'a CollectedReceipt,
}
pub(super) struct Borrowed<'rows, 'memo, 'work> {
    catalog: &'rows Catalog,
    descriptor: SourceDescriptor<'rows>,
    view: &'memo RecipeView<'rows>,
    reader: &'memo MemoReader,
    maps: &'memo Maps,
    store: &'memo Store,
    work: &'work mut ReservationWork,
}
impl<'rows, 'memo, 'work> Borrowed<'rows, 'memo, 'work> {
    pub(super) fn new(
        authority: &Authority<'rows>,
        view: &'memo RecipeView<'rows>,
        reader: &'memo MemoReader,
        maps: &'memo Maps,
        store: &'memo Store,
        work: &'work mut ReservationWork,
    ) -> Result<Self, Error> {
        authority.reservation.require(authority.report, work)?;
        authority.completion.columns(authority.report, work)?;
        reader.require(view, store, work)?;
        maps.require(view, store, work)?;
        // Fixed selected-root/catalog and common owner comparisons. Variable
        // members/bounds are prepaid by their selected producer before entry.
        work.charge(12)?;
        let catalog = authority.reservation.catalog().as_ref();
        let definition = catalog
            .declaration_identity(view.root(), DefKind::Module)
            .map_err(error)?;
        let (provider, row, edition) =
            catalog.declaration_coordinate(view.root()).map_err(error)?;
        let descriptor = view.rows()?.descriptor();
        if descriptor.identity() != provider
            || descriptor.edition() != edition
            || descriptor.profile() != s::Profile::Source2024Provided
            || catalog.definition_at(provider, row).map_err(error)? != definition
            || descriptor
                .rows()
                .declarations
                .get(row.0 as usize)
                .is_none_or(|root| {
                    root.definition.kind != s::DefinitionKind::Module
                        || root.definition.parent.is_some()
                })
        {
            return Err(error(
                "private schema changed its actual consumer descriptor/catalog root",
            ));
        }
        Ok(Self {
            catalog,
            descriptor,
            view,
            reader,
            maps,
            store,
            work,
        })
    }
}
impl checked_rows::sealed::Sealed for Borrowed<'_, '_, '_> {}
impl<'rows> Mapping<'rows> for Borrowed<'rows, '_, '_> {
    fn catalog(&self) -> &'rows Catalog {
        self.catalog
    }
    fn descriptor(&self) -> Result<SourceDescriptor<'rows>, &'static str> {
        Ok(self.descriptor)
    }
    fn require_store(&mut self, store: &Store) -> Result<(), &'static str> {
        self.reader
            .require(self.view, store, self.work)
            .map_err(|_| "private schema changed its actual canonical arena")
    }
    fn ty(&mut self, root: g::TypeRef) -> Result<TypeId, &'static str> {
        match self
            .reader
            .get(self.view, self.store, root, self.work)
            .map_err(|_| "private schema lost its actual canonical memo")?
        {
            MemoLookup::Ready(ty) => {
                if let Some(s::Type::Parameter(slot)) =
                    self.descriptor.rows().types.get(root.0 as usize)
                {
                    self.work
                        .charge(1)
                        .map_err(|_| "private slot read work exceeded")?;
                    match self.maps.slot(slot.0 as usize) {
                        Ok(Some(crate::types::arguments::Argument::Type(actual)))
                            if actual == ty => {}
                        Ok(None) => {
                            return Err("private schema requires its actual pending formal slot");
                        }
                        _ => return Err("private schema changed its actual canonical formal slot"),
                    }
                }
                Ok(ty)
            }
            MemoLookup::Pending { .. } => {
                Err("private schema requires its actual pending type producer")
            }
        }
    }
    fn members(
        &self,
        rows: s::ValidatedView<'rows>,
        header: s::HeaderRef,
    ) -> Result<MemberRows<'rows>, &'static str> {
        match header {
            s::HeaderRef::Trait(index) => Ok(MemberRows::Declared(
                rows.declared_members(rows.traits[index as usize].members)?,
            )),
            s::HeaderRef::Implementation(index) => Ok(MemberRows::Implementation(
                rows.implementation_members(rows.implementations[index as usize].members)?,
            )),
            s::HeaderRef::Function(_) => {
                Err("private schema requires its actual trait or implementation")
            }
        }
    }
    fn region(&mut self, root: s::RegionRef) -> Result<RegionId, &'static str> {
        self.work
            .charge(1)
            .map_err(|_| "private schema region work exceeded")?;
        if !matches!(
            self.descriptor.rows().regions.get(root.0 as usize),
            Some(s::Region::Static | s::Region::Bound { .. })
        ) {
            return Err("private schema needs its actual body-owned regional producer");
        }
        match self.maps.region(root.0 as usize) {
            Ok(Some(region)) => Ok(region),
            Ok(None) => Err("private schema requires its actual pending regional input"),
            Err(message) => Err(message),
        }
    }
}
