//! A selected inventory borrows the same immutable artifact and canonical cells.
use super::super::{
    error,
    mapped_source::checked_rows::{self, Mapping},
    recipe_cursor::{Maps, MemoReader},
};
use super::{
    header_demands::State,
    schema_mapping::{Authority, Borrowed},
};
use crate::{
    resolve::{
        bindings::consumer_policy::{HeaderDataIdentity, HeaderDataOwner},
        body_origin::{Catalog, ImportedDeclarationId},
        crates::{ReservationWork, unpaid_provided::RecipeView},
        model::DefKind,
        package::Error,
    },
    sema::{
        associated::{Owner, OwnerKind, Schema},
        compiler_items::Role,
    },
    support::crates::{Dependencies, SourceDescriptor, generics::source as s},
    types::{Store, arena::Identity},
};
use std::{rc::Rc, sync::Arc};

pub(in crate::sema) struct ProvidedOwners {
    inventory: Arc<Dependencies>,
    catalog: Rc<Catalog>,
    root: ImportedDeclarationId,
    index: u32,
    arena: Identity,
    data: HeaderDataOwner,
    data_identity: HeaderDataIdentity,
    reader: MemoReader,
    maps: Rc<Maps>,
}
impl ProvidedOwners {
    pub(super) fn new(
        authority: &Authority<'_>,
        state: &State,
        store: &Store,
        work: &mut ReservationWork,
    ) -> Result<Rc<Self>, Error> {
        authority.reservation.require(authority.report, work)?;
        let data = authority.completion.retain_data(authority.report, work)?;
        authority.completion.retained_columns(
            authority.reservation.inventory(),
            &data,
            authority.reservation.index(),
            work,
        )?;
        let view = authority.reservation.retained_recipe_view(work)?;
        let reader = state.retained_reader(&view, store, work)?;
        let arena = store
            .existing_arena_identity()
            .ok_or_else(|| error("selected owner lost its actual current arena"))?
            .clone();
        work.charge(5)?;
        work.reserve(size_of::<Self>().saturating_add(2 * size_of::<usize>()))?;
        Ok(Rc::new(Self {
            inventory: Arc::clone(authority.reservation.inventory()),
            catalog: Rc::clone(authority.reservation.catalog()),
            root: view.root(),
            index: authority.reservation.index(),
            arena,
            data_identity: data.identity(work)?,
            data,
            reader,
            maps: Rc::clone(&state.maps),
        }))
    }
    pub(super) fn require_source(
        &self,
        authority: &Authority<'_>,
        state: &State,
        store: &Store,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        authority.reservation.require(authority.report, work)?;
        work.charge(4)?;
        if !Arc::ptr_eq(&self.inventory, authority.reservation.inventory())
            || !Rc::ptr_eq(&self.catalog, authority.reservation.catalog())
            || !Rc::ptr_eq(&self.maps, &state.maps)
            || self.index != authority.reservation.index()
        {
            return Err(error(
                "selected owner changed its actual consumer allocations",
            ));
        }
        authority
            .completion
            .retained_columns(&self.inventory, &self.data, self.index, work)?;
        let view = authority.reservation.retained_recipe_view(work)?;
        state
            .cursor
            .require_reader(&self.reader, &view, store, work)?;
        self.require_store(store).map_err(error)
    }
    fn view(&self) -> RecipeView<'_> {
        RecipeView::retained_parts(&self.inventory, self.index, &self.catalog, self.root)
    }
    fn descriptor(&self) -> Result<SourceDescriptor<'_>, &'static str> {
        let mut work = ReservationWork::default();
        self.data_identity
            .require_owner(&self.data, &mut work)
            .map_err(|_| "selected owner changed its actual sidecar allocation")?;
        let descriptor = self
            .inventory
            .artifacts()
            .get(self.index as usize)
            .and_then(crate::support::crates::Interface::source_descriptor)
            .ok_or("selected owner lost its immutable source descriptor")?;
        let definition = self
            .catalog
            .declaration_identity(self.root, DefKind::Module)?;
        let (provider, row, edition) = self.catalog.declaration_coordinate(self.root)?;
        if descriptor.profile() != s::Profile::Source2024Provided
            || descriptor.identity() != provider
            || descriptor.edition() != edition
            || self.catalog.definition_at(provider, row)? != definition
        {
            return Err("selected owner changed its actual catalog/provider root");
        }
        Ok(descriptor)
    }
    pub(in crate::sema) fn require_store(&self, store: &Store) -> Result<(), &'static str> {
        self.descriptor()?;
        if !store.has_identity(&self.arena) {
            return Err("selected owner belongs to another current type arena");
        }
        let mut work = ReservationWork::default();
        let view = self.view();
        self.reader
            .require(&view, store, &mut work)
            .map_err(|_| "selected owner lost its canonical reader")?;
        self.maps
            .require(&view, store, &mut work)
            .map_err(|_| "selected owner lost its canonical slot maps")
    }
    pub(in crate::sema) fn owner(&self, owner: Owner) -> Result<(), &'static str> {
        checked_rows::require_owner(&self.catalog, || self.descriptor(), owner)
    }
    pub(in crate::sema) fn associated_row(
        &self,
        store: &Store,
        owner: Owner,
        schema: &Schema,
        members: &[crate::resolve::model::DefId],
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        self.require_store(store)?;
        let view = self.view();
        let mut work = ReservationWork::default();
        let result = Borrowed::retained(
            &self.catalog,
            &view,
            &self.reader,
            &self.maps,
            store,
            &mut work,
        )
        .map_err(|_| "selected inventory lost its actual checked mapping")
        .and_then(|mut mapped| mapped.require_schema(store, owner, schema, members, steps));
        *steps = steps.saturating_add(work.steps);
        if *steps > crate::support::crates::MAX_COMPONENTS {
            return Err("selected owner validation work budget exceeded");
        }
        result
    }
    pub(in crate::sema) fn role_origin(&self, role: Role) -> Result<(), &'static str> {
        let crate::resolve::body_origin::DeclarationOrigin::Imported(id) = role.origin else {
            return Err("selected role requires its actual imported origin");
        };
        self.catalog
            .validate_declaration(id, role.definition, DefKind::Trait)?;
        let (provider, _, _) = self.catalog.declaration_coordinate(id)?;
        if provider != self.descriptor()?.identity() {
            return Err("selected role belongs to another actual artifact");
        }
        Ok(())
    }
    pub(in crate::sema) fn trait_flags(&self, owner: Owner) -> Result<u16, &'static str> {
        self.owner(owner)?;
        let OwnerKind::Trait { definition, .. } = owner.kind else {
            return Err("selected trait policy requires its actual trait owner");
        };
        let id = self.catalog.declaration(definition)?;
        let (_, row, _) = self.catalog.declaration_coordinate(id)?;
        let descriptor = self.descriptor()?;
        let Some(s::HeaderRef::Trait(index)) = descriptor.rows().header(row)? else {
            return Err("selected trait policy lost its actual header");
        };
        Ok(descriptor.rows().traits[index as usize].flags.bits())
    }
}
