//! Install only selected genuine declaration scopes; bodies and roles remain unpaid.
use super::{SelectedHeaders, template_demands};
use crate::{
    resolve::{crates::ReservationWork, model::DefId, package::Error},
    sema::{
        crates::typed_headers::{
            error,
            recipe_cursor::Demand,
            staged_headers::contract_ports,
            unpaid_provided::{SchemaAuthority, owner_stages},
        },
        generics::Parameter,
        model::Declarations,
        traits::{Contracts, Header, inherited_inputs::HeaderInputs},
    },
    support::{
        crates::{generics::source as s, owner_catalog::DeclarationRef},
        retained::Growth,
    },
    types::{
        ItemId,
        arguments::Argument,
        parameters::{environment::Environment, predicates::Predicate},
    },
};
use std::sync::Arc;

pub(super) fn prepare(
    selected: &mut SelectedHeaders,
    facts: &mut Declarations,
    authority: &SchemaAuthority<'_>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<Demand<()>, Error> {
    if let Demand::Blocked(dependency) = selected.schema(facts, authority, owner, work)? {
        return Ok(Demand::Blocked(dependency));
    }
    let view = authority.reservation.recipe_view(authority.report, work)?;
    let rows = view.rows()?;
    let Some(s::HeaderRef::Trait(index)) = rows.header(owner).map_err(error)? else {
        return Err(error(
            "selected trait declaration changed its actual header kind",
        ));
    };
    let raw = &rows.traits[index as usize];
    if template_demands::declaration_ready(selected.recipes.as_deref(), &view, facts, owner, work)?
    {
        return Ok(Demand::Ready(()));
    }
    let members = raw.members.slice(&rows.members).map_err(error)?;
    work.charge(members.len().saturating_add(1))?;
    if raw.region_captures != 0
        || raw.region_bounds.start != raw.region_bounds.end
        || members.iter().any(|member| {
            rows.declarations[member.0 as usize].definition.kind != s::DefinitionKind::Function
        })
    {
        return Err(error(
            "selected trait region/result contracts require their explicit wider producer",
        ));
    }
    let mut declarations =
        template_demands::declarations(&mut selected.recipes, facts, owner, work, |work| {
            authority.reservation.recipe_view(authority.report, work)
        })?;
    let definition = view.definition(owner, work)?;
    if declarations.is_empty() {
        facts
            .generics
            .traits
            .as_deref()
            .and_then(|contracts| contracts.header(ItemId(definition.0)))
            .ok_or_else(|| error("installed selected trait lost its actual declaration header"))?;
        return Ok(Demand::Ready(()));
    }
    if facts
        .generics
        .traits
        .as_deref()
        .is_some_and(|contracts| contracts.header(ItemId(definition.0)).is_some())
    {
        return Err(error(
            "selected trait cannot replace an existing declaration header",
        ));
    }
    let leaf = declarations
        .last_mut()
        .filter(|leaf| leaf.definition == definition)
        .ok_or_else(|| error("selected trait lost its exact own declaration scope"))?;
    let self_type = owner_stages::self_type(selected.owners.as_deref(), index)?;
    let (header, implicit) = contract(
        &facts.types,
        authority.reservation.catalog(),
        raw,
        self_type,
        leaf,
        work,
    )?;
    preflight(facts, &declarations, work)?;
    install(facts, &mut declarations, definition, header, implicit)?;
    template_demands::mark_declarations(&mut selected.recipes, &declarations)?;
    Ok(Demand::Ready(()))
}

fn install(
    facts: &mut Declarations,
    declarations: &mut [template_demands::Declaration],
    definition: DefId,
    header: Header,
    implicit: Predicate,
) -> Result<(), Error> {
    // All state belongs to this consumed private draft. A fatal failure disposes
    // of the draft, preserving attempted work; no public header/proof escapes.
    facts
        .generics
        .traits
        .get_or_insert_default()
        .declare(ItemId(definition.0), header)
        .map_err(error)?;
    facts
        .generics
        .trait_self_bounds
        .insert(definition, implicit);
    for declaration in declarations {
        if let Some(parent) = declaration.parent {
            facts
                .generics
                .parents
                .insert(declaration.definition, parent);
        }
        for parameter in &declaration.parameters {
            let Argument::Type(ty) = parameter.argument else {
                return Err(error(
                    "selected declaration requires its actual type formal",
                ));
            };
            facts.definitions[parameter.binding.0 as usize] = Some(ty);
        }
        facts.generics.parameters.insert(
            declaration.definition,
            std::mem::take(&mut declaration.parameters),
        );
        facts
            .generics
            .environments
            .insert(declaration.definition, Arc::clone(&declaration.environment));
    }
    Ok(())
}

fn contract(
    store: &crate::types::Store,
    catalog: &crate::resolve::body_origin::Catalog,
    raw: &s::TraitHeader,
    self_type: crate::types::TypeId,
    leaf: &mut template_demands::Declaration,
    work: &mut ReservationWork,
) -> Result<(Header, Predicate), Error> {
    let implicit = contract_ports::implicit_self(
        store,
        leaf.definition,
        &leaf.parameters,
        &leaf.inputs,
        raw,
        self_type,
        work,
    )?;
    work.charge(leaf.parameters.len().saturating_add(1))?;
    work.reserve(leaf.parameters.len().saturating_mul(size_of::<Parameter>()))?;
    let parameters = leaf.parameters.clone();
    let inputs = HeaderInputs::imported(
        leaf.trait_inputs
            .take()
            .ok_or_else(|| error("selected trait lost its move-only actual input producer"))?,
    );
    inputs
        .require_catalog(catalog, leaf.definition)
        .map_err(error)?;
    let header = Header {
        self_type,
        parameters,
        inputs,
        results: Vec::new(),
        regions: Vec::new(),
    };
    work.charge(4)?;
    let declaration = catalog.declaration(leaf.definition).map_err(error)?;
    catalog
        .validate_declaration(
            declaration,
            leaf.definition,
            crate::resolve::model::DefKind::Trait,
        )
        .map_err(error)?;
    let (provider, _, _) = catalog.declaration_coordinate(declaration).map_err(error)?;
    let self_definition = catalog
        .definition_at(provider, raw.self_definition)
        .map_err(error)?;
    let actual = catalog.declaration(self_definition).map_err(error)?;
    catalog
        .validate_declaration(
            actual,
            self_definition,
            crate::resolve::model::DefKind::SelfType,
        )
        .map_err(error)?;
    crate::sema::compiler_items::headers::require_self(store, header.self_type, self_definition)
        .map_err(error)?;
    Ok((header, implicit))
}

fn preflight(
    facts: &Declarations,
    declarations: &[template_demands::Declaration],
    work: &mut ReservationWork,
) -> Result<(), Error> {
    work.charge(declarations.len().saturating_mul(3).saturating_add(1))?;
    let parents = declarations
        .iter()
        .filter(|row| row.parent.is_some())
        .count();
    let mut growth = Growth::map::<DefId, Vec<Parameter>>(
        facts.generics.parameters.len(),
        facts.generics.parameters.capacity(),
        declarations.len(),
    );
    growth.include(Growth::map::<DefId, Arc<Environment>>(
        facts.generics.environments.len(),
        facts.generics.environments.capacity(),
        declarations.len(),
    ));
    growth.include(Growth::map::<DefId, DefId>(
        facts.generics.parents.len(),
        facts.generics.parents.capacity(),
        parents,
    ));
    growth.include(Growth::map::<DefId, Predicate>(
        facts.generics.trait_self_bounds.len(),
        facts.generics.trait_self_bounds.capacity(),
        1,
    ));
    if let Some(contracts) = facts.generics.traits.as_deref() {
        growth.include(contracts.declaration_growth());
    } else {
        growth.bytes = growth.bytes.saturating_add(size_of::<Contracts>());
        growth.include(Contracts::default().declaration_growth());
    }
    work.charge(growth.work)?;
    work.reserve(growth.bytes)?;
    for row in declarations {
        work.charge(row.parameters.len().saturating_add(2))?;
        if facts.generics.parameters.contains_key(&row.definition)
            || facts.generics.environments.contains_key(&row.definition)
            || facts.generics.parents.contains_key(&row.definition)
            || facts.generics.predicates.contains_key(&row.definition)
        {
            return Err(error(
                "selected declaration cannot replace actual existing scope facts",
            ));
        }
        for parameter in &row.parameters {
            if !matches!(parameter.argument, Argument::Type(_))
                || facts
                    .definitions
                    .get(parameter.binding.0 as usize)
                    .is_none_or(Option::is_some)
            {
                return Err(error(
                    "selected declaration has a missing or previously installed formal cell",
                ));
            }
        }
    }
    Ok(())
}
