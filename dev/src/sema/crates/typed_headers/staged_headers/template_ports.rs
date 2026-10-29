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
            register(
                &facts.types,
                &mut callables,
                &owners,
                &parameters,
                header.callable,
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
