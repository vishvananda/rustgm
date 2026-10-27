//! Selected real schema data stays private until the complete header transaction.
use super::super::{
    error,
    mapped_source::checked_rows::Mapping,
    recipe_cursor::{Demand, Dependency, Port, Reserved},
};
use super::{
    header_demands::State,
    schema_mapping::{Authority, Borrowed},
};
use crate::{
    resolve::{
        bindings::consumer_policy::HeaderDataOwner, crates::ReservationWork, package::Error,
    },
    sema::{
        associated::{ImportedOwner, Owner, OwnerKind, Schema},
        model::Declarations,
    },
    support::{
        crates::{generics::source as s, owner_catalog::DeclarationRef},
        retained,
    },
};

enum Cell {
    Empty,
    Pending(Dependency),
    Prepared(ImportedOwner),
    Installed(crate::sema::associated::OwnerId),
}
pub(super) struct OwnerTransaction {
    traits: Vec<Cell>,
    implementations: Vec<Cell>,
    bytes: usize,
    data: HeaderDataOwner,
}
impl OwnerTransaction {
    fn new(
        rows: s::ValidatedView<'_>,
        data: HeaderDataOwner,
        work: &mut ReservationWork,
    ) -> Result<Box<Self>, Error> {
        let count = rows.traits.len().saturating_add(rows.implementations.len());
        work.charge(count.saturating_add(1))?;
        work.reserve(size_of::<Self>().saturating_add(count.saturating_mul(size_of::<Cell>())))?;
        let mut traits = Vec::with_capacity(rows.traits.len());
        let mut implementations = Vec::with_capacity(rows.implementations.len());
        traits.resize_with(rows.traits.len(), || Cell::Empty);
        implementations.resize_with(rows.implementations.len(), || Cell::Empty);
        let bytes = size_of::<Self>()
            .saturating_add(retained::vector(&traits))
            .saturating_add(retained::vector(&implementations));
        Ok(Box::new(Self {
            traits,
            implementations,
            bytes,
            data,
        }))
    }
    fn retained_bytes(&self) -> usize {
        self.bytes
    }
    fn cell(&mut self, header: s::HeaderRef) -> Result<&mut Cell, Error> {
        let cell = match header {
            s::HeaderRef::Trait(index) => self.traits.get_mut(index as usize),
            s::HeaderRef::Implementation(index) => self.implementations.get_mut(index as usize),
            s::HeaderRef::Function(_) => None,
        };
        cell.ok_or_else(|| error("private schema demand lacks its actual dense owner row"))
    }
}

pub(super) fn self_type(
    owners: Option<&OwnerTransaction>,
    index: u32,
    facts: &Declarations,
) -> Result<crate::types::TypeId, Error> {
    let row = owners.and_then(|owners| owners.traits.get(index as usize));
    match row {
        Some(Cell::Prepared(row)) => Ok(row.schema.self_type),
        Some(Cell::Installed(id)) => {
            let owner = facts
                .associated
                .get(*id)
                .ok_or_else(|| error("selected schema lost its installed actual owner"))?;
            facts
                .associated
                .schema(owner.self_definition)
                .map(|schema| schema.self_type)
                .ok_or_else(|| error("selected schema lost its installed actual Self"))
        }
        _ => Err(error(
            "selected trait declaration requires its actual prepared schema",
        )),
    }
}

pub(super) fn prepare(
    owners: &mut Option<Box<OwnerTransaction>>,
    recipes: &mut Option<Box<State>>,
    facts: &mut Declarations,
    authority: &Authority<'_>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<Demand<()>, Error> {
    authority.reservation.require(authority.report, work)?;
    authority.completion.columns(authority.report, work)?;
    let view = authority.reservation.retained_recipe_view(work)?;
    let rows = view.rows()?;
    let header = rows
        .header(owner)
        .map_err(error)?
        .ok_or_else(|| error("private schema demand lost its actual header"))?;
    if matches!(header, s::HeaderRef::Function(_)) {
        return Err(error(
            "private schema demand requires an actual trait or impl",
        ));
    }
    if recipes.is_none() {
        *recipes = Some(Box::new(State::new(&view, &mut facts.types, work)?));
    }
    let state = recipes
        .as_mut()
        .ok_or_else(|| error("private schema lost its persistent recipe state"))?;
    state.require_mapping(&view, &facts.types, true, work)?;
    if owners.is_none() {
        *owners = Some(OwnerTransaction::new(
            rows,
            authority.completion.retain_data(authority.report, work)?,
            work,
        )?);
    }
    let transaction = owners
        .as_mut()
        .ok_or_else(|| error("private schema transaction disappeared"))?;
    authority.completion.retained_columns(
        authority.reservation.inventory(),
        &transaction.data,
        authority.reservation.index(),
        work,
    )?;
    work.charge(1)?;
    let retained = transaction.retained_bytes();
    let cell = transaction.cell(header)?;
    match cell {
        Cell::Pending(dependency) => return Ok(Demand::Blocked(*dependency)),
        Cell::Empty => {}
        Cell::Prepared(_) | Cell::Installed(_) => {
            let mut mapped = Borrowed::new(
                authority,
                &view,
                state.memo()?,
                &state.maps,
                &facts.types,
                work,
            )?;
            cached(cell, &mut mapped, rows, header, facts)?;
            return Ok(Demand::Ready(()));
        }
    }
    let self_type = match demand(state, facts, view, header, work)? {
        Demand::Ready(ty) => ty,
        Demand::Blocked(dependency) => {
            *cell = Cell::Pending(dependency);
            return Ok(Demand::Blocked(dependency));
        }
    };
    let view = authority.reservation.recipe_view(authority.report, work)?;
    let before = work.retained;
    let row = materialize(authority, &view, state, facts, header, self_type, work)?;
    work.charge(2)?;
    let bytes = retained::vector(&row.members).saturating_add(retained::vector(&row.schema.bounds));
    if bytes > work.retained.saturating_sub(before) {
        return Err(error(
            "private schema exceeded its actual selected allocation preflight",
        ));
    }
    *cell = Cell::Prepared(row);
    transaction.bytes = retained.saturating_add(bytes);
    #[cfg(test)]
    eprintln!(
        "schema-owner layout={} cell={} retained={} peak={} work={}",
        size_of::<OwnerTransaction>(),
        size_of::<Cell>(),
        transaction.retained_bytes(),
        work.peak,
        work.steps
    );

    Ok(Demand::Ready(()))
}

fn cached(
    cell: &Cell,
    mapped: &mut Borrowed<'_, '_, '_>,
    rows: s::ValidatedView<'_>,
    header: s::HeaderRef,
    facts: &Declarations,
) -> Result<(), Error> {
    let (owner, schema) = match cell {
        Cell::Prepared(row) => (row.owner, &row.schema),
        Cell::Installed(id) => {
            let owner = facts
                .associated
                .get(*id)
                .copied()
                .ok_or_else(|| error("selected schema lost its installed owner"))?;
            let schema = facts
                .associated
                .schema(owner.self_definition)
                .ok_or_else(|| error("selected schema lost its installed schema"))?;
            (owner, schema)
        }
        _ => return Err(error("selected cached schema is not actually prepared")),
    };
    mapped.require_owner(owner).map_err(error)?;
    let (self_type, captures, bounds) = shape(rows, header)?;
    if mapped.ty(self_type).map_err(error)? != schema.self_type
        || captures != schema.captures
        || bounds.slice(&rows.region_bounds).map_err(error)?.len() != schema.bounds.len()
    {
        return Err(error(
            "private schema cache changed its actual Self/domain shape",
        ));
    }
    Ok(())
}

pub(super) fn install(
    owners: &mut Option<Box<OwnerTransaction>>,
    recipes: &mut Option<Box<State>>,
    facts: &mut Declarations,
    authority: &Authority<'_>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<Demand<()>, Error> {
    authority.reservation.require(authority.report, work)?;
    let view = authority.reservation.retained_recipe_view(work)?;
    let rows = view.rows()?;
    let header = rows
        .header(owner)
        .map_err(error)?
        .ok_or_else(|| error("selected owner lost its actual header"))?;
    let state = recipes
        .as_mut()
        .ok_or_else(|| error("selected owner lacks its actual persistent recipe state"))?;
    state.require_mapping(&view, &facts.types, false, work)?;
    let transaction = owners
        .as_mut()
        .ok_or_else(|| error("selected owner lacks its actual schema transaction"))?;
    authority.completion.retained_columns(
        authority.reservation.inventory(),
        &transaction.data,
        authority.reservation.index(),
        work,
    )?;
    let mapping = match facts.associated.imported_owner() {
        Some(crate::sema::crates::ImportedOwners::Provided(actual)) => {
            actual.require_source(authority, state, &facts.types, work)?;
            work.charge(1)?;
            std::rc::Rc::clone(actual)
        }
        Some(crate::sema::crates::ImportedOwners::Ordinary(_)) => {
            return Err(error(
                "selected owner cannot replace ordinary imported authority",
            ));
        }
        None => super::ProvidedOwners::new(authority, state, &facts.types, work)?,
    };
    publish(transaction, header, facts, mapping, &view, work)
}

fn publish(
    transaction: &mut OwnerTransaction,
    header: s::HeaderRef,
    facts: &mut Declarations,
    mapping: std::rc::Rc<super::ProvidedOwners>,
    view: &crate::resolve::crates::unpaid_provided::RecipeView<'_>,
    work: &mut ReservationWork,
) -> Result<Demand<()>, Error> {
    match transaction.cell(header)? {
        Cell::Installed(id) => {
            let actual = facts
                .associated
                .get(*id)
                .copied()
                .ok_or_else(|| error("selected owner lost its installed identity"))?;
            let expected = source_owner(view, header, work)?;
            if actual != expected {
                return Err(error("selected owner changed its installed actual row"));
            }
            return Ok(Demand::Ready(()));
        }
        Cell::Pending(dependency) => return Ok(Demand::Blocked(*dependency)),
        Cell::Empty => return Err(error("selected owner requires its actual prepared schema")),
        Cell::Prepared(_) => {}
    }
    let cell = transaction.cell(header)?;
    let Cell::Prepared(row) = cell else {
        return Err(error("selected owner lost its actual prepared row"));
    };
    work.charge(
        row.members
            .len()
            .saturating_add(row.schema.bounds.len())
            .saturating_add(1),
    )?;
    let growth = facts
        .associated
        .selected_allocation_bound(row.members.len());
    work.charge(growth.work)?;
    work.reserve(growth.bytes)?;
    let Cell::Prepared(row) = std::mem::replace(cell, Cell::Empty) else {
        return Err(error(
            "selected owner changed during exclusive installation",
        ));
    };
    let moved = retained::vector(&row.members).saturating_add(retained::vector(&row.schema.bounds));
    let mut steps = work.steps;
    let result = facts.associated.append_selected(
        &facts.types,
        crate::sema::crates::ImportedOwners::Provided(mapping),
        row,
        &mut steps,
    );
    work.steps = steps;
    let id = result.map_err(error)?;
    *cell = Cell::Installed(id);
    transaction.bytes = transaction.bytes.saturating_sub(moved);
    Ok(Demand::Ready(()))
}

fn shape(
    rows: s::ValidatedView<'_>,
    header: s::HeaderRef,
) -> Result<
    (
        crate::support::crates::generics::TypeRef,
        u32,
        crate::support::crates::generics::Range,
    ),
    Error,
> {
    Ok(match header {
        s::HeaderRef::Trait(index) => {
            let raw = &rows.traits[index as usize];
            (raw.self_type, raw.region_captures, raw.region_bounds)
        }
        s::HeaderRef::Implementation(index) => {
            let raw = &rows.implementations[index as usize];
            (raw.self_type, raw.region_captures, raw.region_bounds)
        }
        s::HeaderRef::Function(_) => {
            return Err(error("private schema changed its actual header kind"));
        }
    })
}
fn demand(
    state: &mut State,
    facts: &mut Declarations,
    view: crate::resolve::crates::unpaid_provided::RecipeView<'_>,
    header: s::HeaderRef,
    work: &mut ReservationWork,
) -> Result<Demand<crate::types::TypeId>, Error> {
    let rows = view.rows()?;
    let (self_type, _, bounds) = shape(rows, header)?;
    let bounds = bounds.slice(&rows.region_bounds).map_err(error)?;
    work.charge(bounds.len().saturating_add(1))?;
    let mut port = Reserved::new(view, &state.maps, &facts.types, work)?;
    let self_type = match state
        .cursor
        .demand(&mut port, &mut facts.types, self_type, work)?
    {
        Demand::Ready(ty) => ty,
        Demand::Blocked(dependency) => return Ok(Demand::Blocked(dependency)),
    };
    for bound in bounds {
        for region in [bound.longer, bound.shorter] {
            if let Demand::Blocked(dependency) = port.region(region, work)? {
                return Ok(Demand::Blocked(dependency));
            }
        }
    }
    Ok(Demand::Ready(self_type))
}
fn materialize(
    authority: &Authority<'_>,
    view: &crate::resolve::crates::unpaid_provided::RecipeView<'_>,
    state: &State,
    facts: &Declarations,
    header: s::HeaderRef,
    self_type: crate::types::TypeId,
    work: &mut ReservationWork,
) -> Result<ImportedOwner, Error> {
    let rows = view.rows()?;
    let (_, captures, bounds) = shape(rows, header)?;
    let bounds = bounds.slice(&rows.region_bounds).map_err(error)?;
    let actual = source_owner(view, header, work)?;
    let members = members(view, header, work)?;
    work.charge(bounds.len().saturating_add(1))?;
    work.reserve(
        bounds
            .len()
            .saturating_mul(size_of::<crate::types::outlives::Pair>()),
    )?;
    let mut mapped = Borrowed::new(
        authority,
        view,
        state.memo()?,
        &state.maps,
        &facts.types,
        work,
    )?;
    let mut regional = Vec::with_capacity(bounds.len());
    for bound in bounds {
        regional.push((
            mapped.region(bound.longer).map_err(error)?,
            mapped.region(bound.shorter).map_err(error)?,
        ));
    }
    let row = ImportedOwner {
        owner: actual,
        schema: Schema {
            self_type,
            captures,
            bounds: regional,
        },
        members,
    };
    let mut steps = 0usize;
    mapped
        .require_schema(
            &facts.types,
            row.owner,
            &row.schema,
            &row.members,
            &mut steps,
        )
        .map_err(error)?;
    Ok(row)
}

fn source_owner(
    view: &crate::resolve::crates::unpaid_provided::RecipeView<'_>,
    header: s::HeaderRef,
    work: &mut ReservationWork,
) -> Result<Owner, Error> {
    let rows = view.rows()?;
    Ok(match header {
        s::HeaderRef::Trait(index) => {
            let raw = &rows.traits[index as usize];
            let flags = raw.flags.bits();
            Owner {
                source: None,
                self_definition: view.definition(raw.self_definition, work)?,
                kind: OwnerKind::Trait {
                    definition: view.definition(raw.declaration, work)?,
                    auto_: flags & s::TraitFlags::AUTO != 0,
                    unsafe_: flags & s::TraitFlags::UNSAFE != 0,
                    constant: flags & s::TraitFlags::CONST != 0,
                },
            }
        }
        s::HeaderRef::Implementation(index) => {
            let raw = &rows.implementations[index as usize];
            Owner {
                source: None,
                self_definition: view.definition(raw.self_definition, work)?,
                kind: OwnerKind::ImportedImpl(view.declaration(raw.self_definition, work)?),
            }
        }
        s::HeaderRef::Function(_) => {
            return Err(error("private associated owner cannot be a function"));
        }
    })
}
fn members(
    view: &crate::resolve::crates::unpaid_provided::RecipeView<'_>,
    header: s::HeaderRef,
    work: &mut ReservationWork,
) -> Result<Vec<crate::resolve::model::DefId>, Error> {
    let rows = view.rows()?;
    let s::HeaderRef::Trait(index) = header else {
        let s::HeaderRef::Implementation(index) = header else {
            return Err(error("private associated members lost their actual owner"));
        };
        let links = rows.implementations[index as usize]
            .members
            .slice(&rows.member_links)
            .map_err(error)?;
        work.charge(links.len().saturating_mul(6).saturating_add(1))?;
        work.reserve(
            links
                .len()
                .saturating_mul(size_of::<crate::resolve::model::DefId>()),
        )?;
        let mut members = Vec::with_capacity(links.len());
        for link in links {
            if let Some(implementation) = link.implementation {
                members.push(view.definition(implementation, work)?);
            }
        }
        return Ok(members);
    };
    let members = rows.traits[index as usize]
        .members
        .slice(&rows.members)
        .map_err(error)?;
    work.charge(members.len().saturating_mul(4).saturating_add(1))?;
    work.reserve(
        members
            .len()
            .saturating_mul(size_of::<crate::resolve::model::DefId>()),
    )?;
    let mut actual = Vec::with_capacity(members.len());
    for &row in members {
        actual.push(view.definition(row, work)?);
    }
    Ok(actual)
}
