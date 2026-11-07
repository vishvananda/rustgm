//! Prepare real callable templates and stable symbols without publishing definitions.
use super::{Declarations, Error, Headers, Scope, error, mapped_source};
use crate::{
    resolve::model::DefId,
    support::crates::{ExternalDef, ExternalKind, generics::source as s},
    types::{GenericArgs, ItemId, Kind, TypeId, callable::Templates, parameters::ParameterId},
};

pub(super) struct Callable {
    pub definition: DefId,
    pub ty: TypeId,
    pub symbol: Option<String>,
}
struct Active {
    owner: DefId,
    parameters: usize,
}

pub(super) fn prepare(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    facts: &mut Declarations,
    scopes: &[Scope],
    budget: &mut Headers,
) -> Result<(Templates, Vec<Callable>), Error> {
    let rows = context.descriptor.rows();
    budget.reserve(
        scopes
            .len()
            .saturating_mul(size_of::<Active>() + size_of::<ItemId>())
            .saturating_add(rows.slots.len().saturating_mul(size_of::<ParameterId>()))
            .saturating_add(rows.functions.len().saturating_mul(size_of::<Callable>())),
    )?;
    let mut active = Vec::<Active>::with_capacity(scopes.len());
    let mut owners = Vec::with_capacity(scopes.len());
    let mut parameters = Vec::with_capacity(rows.slots.len());
    let mut callables = Templates::default();
    let mut result = Vec::with_capacity(rows.functions.len());
    for (raw, scope) in rows.scopes.iter().zip(scopes) {
        budget.charge(1)?;
        while active.last().map(|a| a.owner) != scope.parent {
            budget.charge(1)?;
            let closed = active
                .pop()
                .ok_or_else(|| error("source callable owner path is missing or closed"))?;
            parameters.truncate(closed.parameters);
            owners.pop();
        }
        let start = parameters.len();
        owners.push(ItemId(scope.definition.0));
        for parameter in &scope.parameters {
            budget.charge(1)?;
            // A nominal's leading lifetimes own no callable parameter identity,
            // exactly as `Generics::parameter_ids` omits them.
            if matches!(
                parameter.argument,
                crate::types::arguments::Argument::Lifetime(_)
            ) {
                continue;
            }
            parameters.push(
                crate::sema::generics::parameter_id(&facts.types, parameter.argument)
                    .map_err(error)?,
            );
        }
        if let Some(callable) = &scope.function {
            let header = &callable.header;
            // These are the exact scoped generic IDs required by the existing
            // callable API, not a separately flattened proof environment.
            budget.charge(owners.len().saturating_add(parameters.len()))?;
            budget.reserve(
                parameters
                    .len()
                    .saturating_mul(size_of::<ParameterId>())
                    .saturating_add(facts.types.node_allocation_bound(1))
                    .saturating_add(owners.len().saturating_mul(4 * size_of::<ItemId>()))
                    .saturating_add(crate::support::retained::new_map::<
                        ItemId,
                        crate::types::callable::Template,
                    >(1)),
            )?;
            registration(
                &mut facts.types,
                &mut callables,
                (&owners, &parameters),
                callable,
                budget,
            )?;
            let ty = facts
                .types
                .intern(Kind::FunctionItem {
                    item: ItemId(scope.definition.0),
                    arguments: GenericArgs::EMPTY,
                    signature: header.callable,
                })
                .map_err(error)?;
            let symbol = symbol(context, scope.definition, budget)?;
            result.push(Callable {
                definition: scope.definition,
                ty,
                symbol,
            });
        }
        constructors(
            context,
            graph,
            facts,
            &mut callables,
            (raw.owner, scope.definition, &parameters[start..]),
            budget,
        )?;
        active.push(Active {
            owner: scope.definition,
            parameters: start,
        });
    }
    budget.charge(1)?;
    let allocation = callables.allocation();
    budget.charge(allocation.visits)?;
    budget.observed.include(allocation);
    Ok((callables, result))
}

/// Version thirteen: each tuple variant of an enum scope is a callable
/// constructor over its enum's own formals, as local declarations register it.
fn constructors(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    facts: &mut Declarations,
    callables: &mut Templates,
    owner: (
        crate::support::crates::owner_catalog::DeclarationRef,
        DefId,
        &[ParameterId],
    ),
    budget: &mut Headers,
) -> Result<(), Error> {
    let (row, enumeration, parameters) = owner;
    let rows = context.descriptor.rows();
    if rows.enums().headers.is_empty() {
        return Ok(());
    }
    budget.charge(2 + rows.enums().headers.len().checked_ilog2().unwrap_or(0) as usize)?;
    let Ok(position) = rows
        .enums()
        .headers
        .binary_search_by_key(&row.0, |header| header.declaration.0)
    else {
        return Ok(());
    };
    let header = &rows.enums().headers[position];
    let Some(s::HeaderRef::Nominal(index)) = rows.header(row).map_err(error)? else {
        return Err(error(
            "source enum constructor lost its actual nominal header",
        ));
    };
    let result = graph.ty(rows.nominals[index as usize].self_type)?;
    let regions = super::definition_cells::nominal_regions(
        rows,
        rows.nominals[index as usize].self_type,
        budget,
    )?;
    for variant in header
        .variants
        .slice(&rows.enums().variants)
        .map_err(error)?
    {
        budget.charge(1)?;
        if variant.shape != s::VariantShape::Tuple {
            continue;
        }
        let payload = variant.fields.slice(&rows.enums().fields).map_err(error)?;
        budget.reserve(
            parameters
                .len()
                .saturating_mul(size_of::<ParameterId>())
                .saturating_add(facts.types.node_allocation_bound(payload.len() + 2))
                .saturating_add(callables.registration_allocation_bound(parameters.len(), 1)),
        )?;
        let signature = super::definition_cells::constructor_signature(
            graph,
            &mut facts.types,
            (payload, regions),
            result,
            budget,
        )?;
        let item = ItemId(context.definition(variant.declaration, budget)?.0);
        if parameters.is_empty() && regions == 0 {
            callables.insert_monomorphic(&facts.types, item, signature)
        } else {
            // As a local constructor template does, each late enum lifetime
            // is relocated to the template's own frame before registration.
            budget.charge(regions as usize)?;
            let mapping = (0..regions)
                .map(|index| {
                    u16::try_from(index)
                        .map_err(|_| "too many constructor parameters")
                        .and_then(|index| crate::types::RegionId::bound(1, index))
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(error)?;
            let template = crate::types::substitute::Bound::new(mapping)
                .signature(&mut facts.types, signature, 0)
                .map_err(error)?;
            callables.insert_constructor(
                &facts.types,
                item,
                ItemId(enumeration.0),
                template,
                regions,
                parameters,
            )
        }
        .map_err(error)?;
    }
    Ok(())
}

/// A callable without captured slots registers its public signature. Version
/// thirteen: an inherent method's captured slots become its template's leading
/// early parameters and its own late slots its public late frame, exactly as a
/// local method template relocates them. That template is rebuilt here from
/// the method's original signature and region scheme.
fn registration(
    types: &mut crate::types::Store,
    callables: &mut Templates,
    formals: (&[ItemId], &[ParameterId]),
    callable: &super::formal_headers::Callable,
    budget: &mut Headers,
) -> Result<(), Error> {
    let (owners, parameters) = formals;
    let (signature, regions) = (callable.header.signature, &callable.regions);
    if regions.captures == 0 {
        return register(
            types,
            callables,
            owners,
            parameters,
            callable.header.callable,
        );
    }
    if !regions.bounds.is_empty() {
        return Err(error(
            "source captured method region bounds require their wider callable profile",
        ));
    }
    let Some(Kind::Function {
        parameters: inputs, ..
    }) = types.kind(signature)
    else {
        return Err(error("source captured method lost its original signature"));
    };
    let inputs = types.list(inputs).map_or(0, <[TypeId]>::len);
    budget.reserve(
        (regions.total as usize)
            .saturating_mul(size_of::<crate::types::RegionId>())
            .saturating_add(types.node_allocation_bound(inputs.saturating_add(2)))
            .saturating_add(callables.registration_allocation_bound(parameters.len(), 1)),
    )?;
    let mut work = 0;
    let mapping = regions.callable_mapping(&mut work).map_err(error)?;
    budget.charge(work)?;
    let late = u32::try_from(regions.late.len())
        .map_err(|_| error("source captured method has too many late slots"))?;
    let template = crate::types::substitute::Bound::new(mapping)
        .signature(types, signature, late)
        .map_err(error)?;
    callables
        .insert_scoped(types, owners, template, regions.captures, &[], parameters)
        .map_err(error)
}

/// Both private preparation owners supply their already authenticated complete
/// ancestry/formals and prepay their own actual allocation envelope.
pub(in crate::sema::crates::typed_headers) fn register(
    types: &crate::types::Store,
    callables: &mut Templates,
    owners: &[ItemId],
    parameters: &[ParameterId],
    signature: TypeId,
) -> Result<(), Error> {
    if parameters.is_empty() {
        let item = *owners
            .last()
            .ok_or_else(|| error("missing actual callable owner"))?;
        callables
            .insert_monomorphic(types, item, signature)
            .map_err(error)
    } else {
        callables
            .insert_scoped(types, owners, signature, 0, &[], parameters)
            .map_err(error)
    }
}

fn symbol(
    context: &mapped_source::Context<'_>,
    definition: DefId,
    budget: &mut Headers,
) -> Result<Option<String>, Error> {
    let id = context.catalog.declaration(definition).map_err(error)?;
    let (_, declaration, _) = context.catalog.declaration_coordinate(id).map_err(error)?;
    budget.charge(
        2 + context
            .descriptor
            .rows()
            .exact_callables
            .len()
            .checked_ilog2()
            .unwrap_or(0) as usize,
    )?;
    if context
        .descriptor
        .rows()
        .exact_callable(declaration)
        .is_some()
    {
        return Ok(None);
    }
    let symbol = symbol_from(context, definition, budget)?;
    budget.observed.add(symbol.capacity());
    Ok(Some(symbol))
}
pub(in crate::sema::crates::typed_headers) fn symbol_from(
    context: &mapped_source::Context<'_>,
    definition: DefId,
    budget: &mut impl super::super::recipe_cursor::Budget,
) -> Result<String, Error> {
    let id = context.catalog.declaration(definition).map_err(error)?;
    let (owner, row, _) = context.catalog.declaration_coordinate(id).map_err(error)?;
    let rows = context.descriptor.rows();
    if context.descriptor.profile().feature_predicates()
        && let Some(s::HeaderRef::Function(index)) = rows.header(row).map_err(error)?
        && rows.functions[index as usize].parent.is_some_and(|parent| {
            matches!(
                rows.header(parent),
                Ok(Some(s::HeaderRef::Implementation(_)))
            )
        })
    {
        let s::FunctionBody::Original(reference) = rows.functions[index as usize].body else {
            return Err(error("private source method omitted its actual original"));
        };
        let body = context.catalog.body_at(owner, reference).map_err(error)?;
        context
            .catalog
            .validate_function_origin(definition, context.catalog.origin(body).map_err(error)?)
            .map_err(error)?;
        let growth = 4 * (owner.name().len() + owner.disambiguator().len() + 64);
        budget.charge(growth)?;
        budget.reserve(growth)?;
        return Ok(owner.private_original_symbol(reference, row));
    }
    let path = if context.descriptor.profile().module_contracts() {
        let root = context
            .descriptor
            .module_contracts()
            .map_err(error)?
            .rows()
            .namespace
            .root;
        let path = s::callable_path(&rows.declarations, root, row, |n| budget.charge(n), error)?;
        budget.reserve(path.storage_bound())?;
        path.materialize().map_err(error)?
    } else {
        legacy_callable_path(&rows, row, budget)?
    };
    let identity = ExternalDef::new(owner.clone(), path, ExternalKind::Function).map_err(error)?;
    if context.imported.definition(&identity) != Some(definition) {
        return Err(error(
            "source callable changed its actual reserved external identity",
        ));
    }
    super::super::symbol(&identity, budget)
}

fn legacy_callable_path(
    rows: &s::Columns,
    row: crate::support::crates::owner_catalog::DeclarationRef,
    budget: &mut impl super::super::recipe_cursor::Budget,
) -> Result<Vec<String>, Error> {
    let raw = &rows.declarations[row.0 as usize];
    let parent = raw
        .definition
        .parent
        .ok_or_else(|| error("source callable lacks its actual lexical owner"))?;
    let parent = &rows.declarations[parent.0 as usize];
    let name = raw
        .name
        .as_ref()
        .ok_or_else(|| error("source callable has no stable declared name"))?;
    let parent_name = if parent.definition.kind == s::DefinitionKind::Trait {
        Some(
            parent
                .name
                .as_ref()
                .ok_or_else(|| error("source method has no stable trait owner name"))?,
        )
    } else if parent.definition.kind == s::DefinitionKind::Module {
        None
    } else {
        return Err(error(
            "source callable requires its wider actual lexical owner profile",
        ));
    };
    budget.reserve(
        name.len()
            .saturating_add(parent_name.map_or(0, String::len))
            .saturating_add(2 * size_of::<String>()),
    )?;
    let mut path = Vec::with_capacity(2);
    if let Some(name) = parent_name {
        path.push(name.clone());
    }
    path.push(name.clone());
    budget.charge(
        path.iter()
            .map(String::len)
            .sum::<usize>()
            .saturating_add(1),
    )?;
    Ok(path)
}
