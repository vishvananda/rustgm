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
    pub symbol: String,
}
struct Active {
    owner: DefId,
    parameters: usize,
}

pub(super) fn prepare(
    context: &mapped_source::Context<'_>,
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
    for scope in scopes {
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
            if !parameters.is_empty() {
                callables
                    .insert_scoped(&facts.types, &owners, header.callable, 0, &[], &parameters)
                    .map_err(error)?;
            } else {
                // A completed empty scope still owns a callable registration;
                // its absence must not stand in for a checked closed signature.
                callables
                    .insert_monomorphic(&facts.types, ItemId(scope.definition.0), header.callable)
                    .map_err(error)?;
            }
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

fn symbol(
    context: &mapped_source::Context<'_>,
    definition: DefId,
    budget: &mut Headers,
) -> Result<String, Error> {
    let id = context.catalog.declaration(definition).map_err(error)?;
    let (owner, row, _) = context.catalog.declaration_coordinate(id).map_err(error)?;
    let rows = context.descriptor.rows();
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
    let identity = ExternalDef::new(owner.clone(), path, ExternalKind::Function).map_err(error)?;
    if context.imported.definition(&identity) != Some(definition) {
        return Err(error(
            "source callable changed its actual reserved external identity",
        ));
    }
    let symbol = budget.symbol(&identity)?;
    budget.observed.add(symbol.capacity());
    Ok(symbol)
}
