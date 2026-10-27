//! A selected inventory borrows the same immutable artifact and canonical cells.
use super::super::{
    error,
    mapped_source::checked_rows::{self, Mapping},
    recipe_cursor::{Maps, MemoReader},
};
use super::{
    header_demands::State,
    schema_mapping::{Authority, Borrowed},
    template_demands,
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

#[path = "checked_callables.rs"]
mod callables;
#[path = "reconstruction_maps.rs"]
mod originals;

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
    scopes: template_demands::ScopeReader,
    applications: super::super::predicate_remap::translation::applications::Reader,
}
impl ProvidedOwners {
    pub(super) fn application_writer<'a>(
        &self,
        shared: &'a super::super::predicate_remap::translation::applications::Shared,
        view: &RecipeView<'_>,
        store: &Store,
        work: &mut ReservationWork,
    ) -> Result<super::super::predicate_remap::translation::applications::Writer<'a>, Error> {
        self.require_current(store, work).map_err(error)?;
        shared.writer(&self.applications, view, store, work)
    }
    pub(super) fn implementation_environment(
        &self,
        authority: &Authority<'_>,
        state: &State,
        facts: &crate::sema::model::Declarations,
        definition: crate::resolve::model::DefId,
        work: &mut ReservationWork,
    ) -> Result<
        (
            Arc<crate::types::parameters::environment::Environment>,
            usize,
        ),
        Error,
    > {
        self.require_source(authority, state, &facts.types, work)?;
        let id = self.catalog.declaration(definition).map_err(error)?;
        self.catalog
            .validate_declaration(id, definition, DefKind::SelfType)
            .map_err(error)?;
        let (provider, row, _) = self.catalog.declaration_coordinate(id).map_err(error)?;
        if provider != self.descriptor().map_err(error)?.identity()
            || !matches!(
                self.view().rows()?.header(row).map_err(error)?,
                Some(s::HeaderRef::Implementation(_))
            )
        {
            return Err(error(
                "selected impl environment changed its actual declaration",
            ));
        }
        let (all, _) =
            self.scopes
                .inputs(&self.view(), &self.catalog, &facts.generics, row, work)?;
        let environment = facts
            .generics
            .environments
            .get(&definition)
            .ok_or_else(|| error("selected impl environment omitted its actual installed Arc"))?;
        work.charge(1)?;
        Ok((Arc::clone(environment), all.len()))
    }
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
        let applications = template_demands::application_reader(state, &view, store, work)?;
        let scopes = template_demands::scope_reader(state, &view, store, work)?;
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
            applications,
            scopes,
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
        template_demands::require_application_reader(
            state,
            &self.applications,
            &view,
            store,
            work,
        )?;
        template_demands::require_scope_reader(state, &self.scopes, &view, store, work)?;
        self.require_current(store, work).map_err(error)
    }
    fn view(&self) -> RecipeView<'_> {
        RecipeView::retained_parts(&self.inventory, self.index, &self.catalog, self.root)
    }
    pub(in crate::sema::crates::typed_headers) fn descriptor(
        &self,
    ) -> Result<SourceDescriptor<'_>, &'static str> {
        let mut work = ReservationWork::default();
        self.data_identity
            .require_owner(&self.data, &mut work)
            .map_err(|_| "selected owner changed its actual sidecar allocation")?;
        self.data.require_inventory(&self.inventory, self.index)?;
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
        if !matches!(
            descriptor.profile(),
            s::Profile::Source2024Provided | s::Profile::Source2024ProvidedContracts
        ) || descriptor.identity() != provider
            || descriptor.edition() != edition
            || self.catalog.definition_at(provider, row)? != definition
        {
            return Err("selected owner changed its actual catalog/provider root");
        }
        Ok(descriptor)
    }
    fn require_current(
        &self,
        store: &Store,
        work: &mut ReservationWork,
    ) -> Result<(), &'static str> {
        work.charge(14)
            .map_err(|_| "selected owner validation work budget exceeded")?;
        self.descriptor()?;
        if !store.has_identity(&self.arena) {
            return Err("selected owner belongs to another current type arena");
        }
        let view = self.view();
        self.reader
            .require(&view, store, work)
            .map_err(|_| "selected owner lost its canonical reader")?;
        self.maps
            .require(&view, store, work)
            .map_err(|_| "selected owner lost its canonical slot maps")?;
        self.scopes
            .require(&view, store, work)
            .map_err(|_| "selected owner lost its actual scope allocation")?;
        self.applications
            .require(&view, store, work)
            .map_err(|_| "selected owner lost its actual canonical applications")
    }
    pub(in crate::sema) fn require_store(&self, store: &Store) -> Result<(), &'static str> {
        self.require_current(store, &mut ReservationWork::default())
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
        let view = self.view();
        let mut work = ReservationWork::default();
        let result = (|| {
            self.require_current(store, &mut work)?;
            let mut mapped = Borrowed::retained(
                &self.catalog,
                &view,
                &self.reader,
                &self.maps,
                store,
                &mut work,
            )
            .map_err(|_| "selected inventory lost its actual checked mapping")?;
            mapped.require_schema(store, owner, schema, members, steps)
        })();
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
    pub(in crate::sema::crates::typed_headers) fn function(
        &self,
        catalog: &Catalog,
        definition: crate::resolve::model::DefId,
        body: Option<crate::resolve::body_origin::BodyOrigin>,
        universals: u32,
    ) -> Result<bool, &'static str> {
        if !std::ptr::eq(catalog, self.catalog.as_ref()) {
            return Err("selected function changed its actual current catalog");
        }
        super::super::mapped_source::require_function(
            catalog,
            self.descriptor()?,
            definition,
            body,
            universals,
        )
    }
    pub(in crate::sema::crates::typed_headers) fn owns_original(
        &self,
        catalog: &Catalog,
        definition: crate::resolve::model::DefId,
        steps: &mut usize,
    ) -> Result<bool, &'static str> {
        *steps = steps.saturating_add(1);
        if !std::ptr::eq(catalog, self.catalog.as_ref()) {
            return Err("selected original changed its actual current catalog");
        }
        let id = catalog.declaration(definition)?;
        catalog.validate_declaration(id, definition, DefKind::Function)?;
        let (provider, _, _) = catalog.declaration_coordinate(id)?;
        Ok(provider == self.descriptor()?.identity())
    }
    pub(in crate::sema::crates::typed_headers) fn declaration_inputs<'a>(
        &'a self,
        catalog: &Catalog,
        generics: &crate::sema::generics::Generics,
        definition: crate::resolve::model::DefId,
        declared: bool,
    ) -> Result<Option<&'a [crate::sema::declaration_inputs::Input]>, &'static str> {
        if !std::ptr::eq(catalog, self.catalog.as_ref()) {
            return Err("selected inputs changed their actual catalog allocation");
        }
        let Some(id) = catalog.declaration_if_imported(definition)? else {
            return Ok(None);
        };
        let (provider, row, _) = catalog.declaration_coordinate(id)?;
        let descriptor = self.descriptor()?;
        if provider != descriptor.identity() {
            return Ok(None);
        }
        let rows = descriptor.rows();
        let raw = rows
            .declarations
            .get(row.0 as usize)
            .ok_or("selected input declaration is outside its actual table")?;
        catalog.validate_declaration(
            id,
            definition,
            super::super::mapped_source::slot_bindings::kind(raw.definition.kind),
        )?;
        let mut work = ReservationWork::default();
        let (all, prefix) = self
            .scopes
            .inputs(&self.view(), catalog, generics, row, &mut work)
            .map_err(|_| "selected inputs require their actual pending installed scope")?;
        Ok(Some(if declared { &all[..prefix] } else { all }))
    }
    fn implementation_header(&self, owner: Owner) -> Result<s::ImplHeader, &'static str> {
        self.owner(owner)?;
        let OwnerKind::ImportedImpl(id) = owner.kind else {
            return Err("selected implementation requires its actual imported impl owner");
        };
        let (_, row, _) = self.catalog.declaration_coordinate(id)?;
        let descriptor = self.descriptor()?;
        let Some(s::HeaderRef::Implementation(index)) = descriptor.rows().header(row)? else {
            return Err("selected implementation lost its actual header");
        };
        descriptor
            .rows()
            .implementations
            .get(index as usize)
            .copied()
            .ok_or("selected implementation is outside its actual table")
    }
    pub(in crate::sema) fn implementation_request(
        &self,
        store: &Store,
        owner: Owner,
    ) -> Result<crate::types::parameters::predicates::Predicate, &'static str> {
        self.require_store(store)?;
        let raw = self.implementation_header(owner)?;
        let reference = raw
            .trait_application
            .ok_or("selected inherent implementation requires its explicit producer")?;
        let mut work = ReservationWork::default();
        self.applications
            .get(&self.view(), store, reference, &mut work)
            .map_err(|_| "selected implementation changed its actual application cache")?
            .ok_or("selected implementation requires its actual pending application producer")
    }
    pub(in crate::sema) fn implementation(
        &self,
        store: &Store,
        owner: Owner,
        predicate: crate::types::parameters::predicates::Predicate,
    ) -> Result<(bool, bool), &'static str> {
        let expected = self.implementation_request(store, owner)?;
        let raw = self.implementation_header(owner)?;
        let view = self.view();
        let mut work = ReservationWork::default();
        let mut mapped = Borrowed::retained(
            &self.catalog,
            &view,
            &self.reader,
            &self.maps,
            store,
            &mut work,
        )
        .map_err(|_| "selected implementation lost its actual checked mapping")?;
        if predicate != expected || mapped.ty(raw.self_type)? != predicate.ty {
            return Err("selected implementation changed its complete actual trait/Self predicate");
        }
        Ok((raw.constant, raw.negative))
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
    pub(in crate::sema::crates::typed_headers) fn trait_header<'a>(
        &self,
        catalog: &Catalog,
        facts: &'a crate::sema::model::Declarations,
        role: Role,
        steps: &mut usize,
    ) -> Result<crate::sema::compiler_items::headers::Trait<'a>, &'static str> {
        let mut work = ReservationWork::default();
        let current = self.require_current(&facts.types, &mut work);
        *steps = steps.saturating_add(work.steps);
        current?;
        let authenticated = work.steps;
        self.role_origin(role)?;
        if !std::ptr::eq(catalog, self.catalog.as_ref()) {
            return Err("selected role header changed its actual current catalog");
        }
        let id = catalog.declaration(role.definition)?;
        if role.origin != crate::resolve::body_origin::DeclarationOrigin::Imported(id) {
            return Err("selected role header changed its actual declaration origin");
        }
        let (_, row, _) = catalog.declaration_coordinate(id)?;
        let descriptor = self.descriptor()?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Trait(index)) = rows.header(row)? else {
            return Err("selected role header lost its actual trait row");
        };
        let raw = &rows.traits[index as usize];
        let owner_id = facts
            .associated
            .owner(role.definition)
            .ok_or("selected role header requires its actual pending owner producer")?;
        let owner = *facts
            .associated
            .get(owner_id)
            .ok_or("selected role header lost its actual installed owner")?;
        let schema = facts
            .associated
            .schema(role.definition)
            .ok_or("selected role header requires its actual pending schema producer")?;
        let members = facts
            .associated
            .owner_members(owner_id)
            .ok_or("selected role header lost its actual complete member inventory")?;
        self.associated_row(&facts.types, owner, schema, members, steps)?;
        let header = facts
            .generics
            .traits
            .as_deref()
            .and_then(|contracts| contracts.header(crate::types::ItemId(role.definition.0)))
            .ok_or("selected role header requires its actual pending canonical input producer")?;
        let checked = (|| {
            let (all, prefix) = self
                .scopes
                .inputs(&self.view(), catalog, &facts.generics, row, &mut work)
                .map_err(|_| "selected role header changed its actual installed scope")?;
            let (actual, subjects) = header.inputs.imported_parts(catalog, role.definition)?;
            if !Arc::ptr_eq(all, actual)
                || prefix != raw.declared_inputs as usize
                || subjects.len() != raw.canonical_inputs as usize
                || header.self_type != schema.self_type
            {
                return Err(
                    "selected role header changed its exact all-input Arc/canonical prefix/Self",
                );
            }
            super::super::mapped_source::installed_trait_header(facts, role, raw, rows)
        })();
        *steps = steps.saturating_add(work.steps.saturating_sub(authenticated));
        checked
    }
}
