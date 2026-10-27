//! Install real selected callable declarations while their defining bodies stay pending.
use super::{SelectedHeaders, declarations, template_demands};
use crate::{
    resolve::{
        body_origin::DeclarationOrigin, crates::ReservationWork, model::DefId, package::Error,
    },
    sema::{
        crates::{
            ImportedOwners,
            typed_headers::{
                Header, Headers, error, mapped_source,
                recipe_cursor::{Demand, MemoLookup, Port, Reserved},
                staged_headers,
                unpaid_provided::SchemaAuthority,
            },
        },
        lifetimes::FunctionRegions,
        model::Declarations,
    },
    support::{crates::owner_catalog::DeclarationRef, retained::Growth},
    types::{GenericArgs, ItemId, Kind, TypeId},
};
use std::{rc::Rc, sync::Arc};

pub(super) fn ready(
    selected: &SelectedHeaders,
    facts: &Declarations,
    authority: &SchemaAuthority<'_>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<bool, Error> {
    let view = authority.reservation.recipe_view(authority.report, work)?;
    let definition = view.definition(owner, work)?;
    let Some(headers) = facts
        .crates
        .as_deref()
        .and_then(|linkage| linkage.originals.as_deref())
    else {
        return Ok(false);
    };
    let Some(header) = headers.entries.get(&definition) else {
        return Ok(false);
    };
    let state = selected
        .recipes
        .as_deref()
        .ok_or_else(|| error("selected method lost its actual signature producer"))?;
    let Some(ImportedOwners::Provided(mapping)) = facts.associated.imported_owner() else {
        return Err(error("selected method lost its actual indexed owner"));
    };
    mapping.require_source(authority, state, &facts.types, work)?;
    let actual_owner = facts
        .associated
        .imported_owner()
        .ok_or_else(|| error("selected method owner disappeared"))?;
    if headers
        .source
        .as_ref()
        .is_none_or(|source| !source.same_owner(actual_owner))
    {
        return Err(error(
            "selected method header changed its actual owner allocation",
        ));
    }
    let pair = state
        .ready_pair(&view, &facts.types, owner, work)?
        .ok_or_else(|| error("selected method lost its actual prepared signature pair"))?;
    template_demands::registered_pair(state, &view, facts, owner, pair, work)?;
    if !template_demands::declaration_ready(Some(state), &view, facts, owner, work)? {
        return Err(error("selected method lost its actual installed scope"));
    }
    let rows = view.rows()?;
    let raw = rows
        .functions
        .get(pair.function_row as usize)
        .ok_or_else(|| error("selected method lost its actual function row"))?;
    let body = pair
        .body
        .map(|body| authority.reservation.catalog().origin(body))
        .transpose()
        .map_err(error)?;
    if header.declaration != DeclarationOrigin::Imported(pair.declaration)
        || header.signature != pair.original
        || header.callable != pair.callable
        || header.body != body
        || header.constant != is_constant(raw.constness)?
    {
        return Err(error(
            "installed method changed its actual sealed declaration/signature/body presence",
        ));
    }
    Ok(true)
}

pub(super) fn prepare(
    selected: &mut SelectedHeaders,
    facts: &mut Declarations,
    authority: &SchemaAuthority<'_>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<Demand<()>, Error> {
    let view = authority.reservation.recipe_view(authority.report, work)?;
    let rows = view.rows()?;
    let state = selected
        .recipes
        .as_mut()
        .ok_or_else(|| error("selected method omitted its actual template producer"))?;
    let pair = match state.pair(
        authority.reservation.retained_recipe_view(work)?,
        &mut facts.types,
        owner,
        work,
    )? {
        Demand::Ready(pair) => pair,
        Demand::Blocked(dependency) => return Ok(Demand::Blocked(dependency)),
    };
    let raw = rows
        .functions
        .get(pair.function_row as usize)
        .filter(|raw| raw.declaration == owner)
        .ok_or_else(|| error("selected method lost its actual function row"))?;
    let definition = view.definition(owner, work)?;
    state.require_mapping(&view, &facts.types, false, work)?;
    if template_demands::declaration_ready(Some(state), &view, facts, owner, work)? {
        let header = facts
            .crates
            .as_deref()
            .and_then(|linkage| linkage.originals.as_deref())
            .and_then(|headers| headers.entries.get(&definition))
            .ok_or_else(|| error("installed method omitted its actual declaration header"))?;
        if header.declaration != DeclarationOrigin::Imported(pair.declaration)
            || header.signature != pair.original
            || header.callable != pair.callable
            || header.constant != is_constant(raw.constness)?
        {
            return Err(error(
                "installed method changed its sealed declaration/signature",
            ));
        }
        return Ok(Demand::Ready(()));
    }
    let mut scopes =
        template_demands::declarations(&mut selected.recipes, facts, owner, work, |work| {
            authority.reservation.recipe_view(authority.report, work)
        })?;
    let leaf = scopes
        .last()
        .filter(|scope| scope.definition == definition)
        .ok_or_else(|| error("selected method lost its actual own declaration scope"))?;
    let state = selected
        .recipes
        .as_deref()
        .ok_or_else(|| error("selected method lost its actual recipe owner"))?;
    let callable = callable(facts, authority, state, pair, leaf, work)?;
    let imported = authority
        .report
        .resolution
        .crates
        .as_deref()
        .ok_or_else(|| error("selected method lost its actual imported names"))?;
    let context = mapped_source::Context {
        imported,
        catalog: authority.reservation.catalog(),
        descriptor: rows.descriptor(),
        provider_index: authority.reservation.index(),
    };
    let symbol = selected_symbol(&context, raw, definition, work)?;
    declarations::preflight_scopes(facts, &scopes, work)?;
    publish(facts, definition, callable, symbol, &mut scopes, work)?;
    template_demands::mark_declarations(&mut selected.recipes, &scopes)?;
    Ok(Demand::Ready(()))
}

// An actual impl member has a canonical catalog owner, not a public extern path.
fn selected_symbol(
    context: &mapped_source::Context<'_>,
    raw: &crate::support::crates::generics::source::FunctionHeader,
    definition: DefId,
    work: &mut ReservationWork,
) -> Result<Option<String>, Error> {
    use crate::support::crates::generics::source as s;
    work.charge(4)?;
    let rows = context.descriptor.rows();
    if let Some(parent) = raw.parent
        && let Some(s::HeaderRef::Implementation(index)) = rows.header(parent).map_err(error)?
    {
        let implementation = rows
            .implementations
            .get(index as usize)
            .ok_or_else(|| error("selected method lost its actual impl header"))?;
        if implementation.self_definition != parent
            || rows.declarations[raw.declaration.0 as usize]
                .definition
                .parent
                != Some(parent)
            || rows.declarations[parent.0 as usize].definition.kind != s::DefinitionKind::SelfType
        {
            return Err(error(
                "selected method changed its actual impl lexical owner",
            ));
        }
        let owner = context
            .catalog
            .definition_at(context.descriptor.identity(), parent)
            .map_err(error)?;
        let declaration = context.catalog.declaration(owner).map_err(error)?;
        context
            .catalog
            .validate_declaration(declaration, owner, crate::resolve::model::DefKind::SelfType)
            .map_err(error)?;
        return Ok(None);
    }
    staged_headers::template_ports::symbol_from(context, definition, work).map(Some)
}

fn callable(
    facts: &Declarations,
    authority: &SchemaAuthority<'_>,
    state: &super::super::header_demands::State,
    pair: super::super::header_demands::SignaturePair,
    leaf: &template_demands::Declaration,
    work: &mut ReservationWork,
) -> Result<staged_headers::Callable, Error> {
    let view = authority.reservation.retained_recipe_view(work)?;
    let rows = view.rows()?;
    let raw = rows
        .functions
        .get(pair.function_row as usize)
        .ok_or_else(|| error("selected method lost its raw signature owner"))?;
    let regions = {
        let mut port = Reserved::new(
            authority.reservation.retained_recipe_view(work)?,
            &state.maps,
            &facts.types,
            work,
        )?;
        staged_headers::scheme_ports::from_rows(rows, raw, work, |reference, work| {
            match port.region(reference, work)? {
                Demand::Ready(region) => Ok(region),
                Demand::Blocked(_) => Err(error(
                    "selected callable scheme requires its actual pending region producer",
                )),
            }
        })?
    };
    let receiver = staged_headers::scheme_ports::receiver_from(
        &facts.types,
        raw,
        pair.original,
        work,
        |reference, work| match state.memo()?.get(&view, &facts.types, reference, work)? {
            MemoLookup::Ready(ty) => Ok(ty),
            MemoLookup::Pending { .. } => Err(error(
                "selected receiver requires its actual pending type producer",
            )),
        },
    )?;
    let declared = raw.declared_inputs as usize;
    if declared > leaf.inputs.len() {
        return Err(error(
            "selected method truncated its actual declaration input prefix",
        ));
    }
    let header = Header {
        declaration: DeclarationOrigin::Imported(pair.declaration),
        body: pair
            .body
            .map(|body| authority.reservation.catalog().origin(body))
            .transpose()
            .map_err(error)?,
        signature: pair.original,
        callable: pair.callable,
        environment: None,
        inputs: Arc::clone(&leaf.inputs),
        declared,
        universals: raw.regions.total,
        constant: is_constant(raw.constness)?,
    };
    Ok(staged_headers::Callable {
        header,
        regions,
        receiver,
    })
}

fn publish(
    facts: &mut Declarations,
    definition: DefId,
    callable: staged_headers::Callable,
    symbol: Option<String>,
    scopes: &mut [template_demands::Declaration],
    work: &mut ReservationWork,
) -> Result<(), Error> {
    let staged_headers::Callable {
        header,
        regions,
        receiver,
    } = callable;
    work.charge(4)?;
    if facts
        .definitions
        .get(definition.0 as usize)
        .is_none_or(Option::is_some)
        || facts.lifetimes.functions.contains_key(&definition)
        || facts.methods.receivers.contains_key(&definition)
        || facts
            .crates
            .as_deref()
            .is_some_and(|linkage| linkage.symbols.contains_key(&definition))
    {
        return Err(error(
            "selected method cannot replace existing semantic declaration facts",
        ));
    }
    let mapping = match facts.associated.imported_owner() {
        Some(ImportedOwners::Provided(owner)) => Rc::clone(owner),
        _ => {
            return Err(error(
                "selected method requires its actual installed owner mapping",
            ));
        }
    };
    Headers::ensure_selected_owner(facts, work)?;
    let growth = publication_growth(facts, symbol.is_some(), receiver.is_some())?;
    let linkage = facts
        .crates
        .as_deref_mut()
        .ok_or_else(|| error("selected method lost its current linkage"))?;
    if let Some(headers) = linkage.originals.as_deref()
        && (headers.source.as_ref().is_none_or(|actual| {
            !actual.same_owner(&ImportedOwners::Provided(Rc::clone(&mapping)))
        }) || headers.entries.contains_key(&definition))
    {
        return Err(error(
            "selected method changed its actual header owner/index",
        ));
    }
    work.charge(growth.work)?;
    work.reserve(
        growth
            .bytes
            .saturating_add(facts.types.node_allocation_bound(1)),
    )?;
    let ty = facts
        .types
        .intern(Kind::FunctionItem {
            item: ItemId(definition.0),
            arguments: GenericArgs::EMPTY,
            signature: header.callable,
        })
        .map_err(error)?;
    let headers = linkage
        .originals
        .as_deref_mut()
        .ok_or_else(|| error("selected method lost its actual prepaid header owner"))?;
    headers.entries.insert(definition, header);
    if let Some(symbol) = symbol {
        linkage.symbols.insert(definition, symbol);
    }
    facts.lifetimes.functions.insert(definition, regions);
    if let Some(receiver) = receiver {
        facts.methods.receivers.insert(definition, receiver);
    }
    facts.definitions[definition.0 as usize] = Some(ty);
    declarations::install_scopes(facts, scopes)
}

fn publication_growth(facts: &Declarations, symbol: bool, receiver: bool) -> Result<Growth, Error> {
    let linkage = facts
        .crates
        .as_deref()
        .ok_or_else(|| error("selected method lost its current linkage"))?;
    let mut growth = Growth::map::<DefId, FunctionRegions>(
        facts.lifetimes.functions.len(),
        facts.lifetimes.functions.capacity(),
        1,
    );
    if symbol {
        growth.include(Growth::map::<DefId, String>(
            linkage.symbols.len(),
            linkage.symbols.capacity(),
            1,
        ));
    }
    if receiver {
        growth.include(Growth::map::<DefId, (TypeId, u32)>(
            facts.methods.receivers.len(),
            facts.methods.receivers.capacity(),
            1,
        ));
    }
    let headers = linkage
        .originals
        .as_deref()
        .ok_or_else(|| error("selected method growth lacks its actual header owner"))?;
    growth.include(Growth::map::<DefId, Header>(
        headers.entries.len(),
        headers.entries.capacity(),
        1,
    ));
    Ok(growth)
}

fn is_constant(value: crate::support::constness::Constness) -> Result<bool, Error> {
    match value {
        crate::support::constness::Constness::Ordinary => Ok(false),
        crate::support::constness::Constness::Const => Ok(true),
        crate::support::constness::Constness::Conditional => Err(error(
            "selected callable eligibility requires its actual ordinary or const declaration",
        )),
    }
}
