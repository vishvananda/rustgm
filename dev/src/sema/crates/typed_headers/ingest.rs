//! Publish complete canonical formals and caller conditions, never paid evidence.
use super::{Declarations, Error, Header, Headers, error};
use crate::{
    resolve::{
        body_origin::Catalog,
        crates::Imported,
        model::{DefId, DefKind},
    },
    sema::{declaration_inputs::Input, generics::Parameter, lifetimes::FunctionRegions},
    support::crates::{CrateIdentity, generics as g},
    types::{
        ItemId, Kind,
        arguments::Argument,
        parameters::{
            clauses::{Clause, Quantified},
            environment::{Abilities, Environment},
            predicates::{Bound, Predicate},
        },
    },
};
use std::sync::Arc;
pub(super) struct Context<'a> {
    pub imported: &'a Imported,
    pub catalog: &'a Catalog,
    pub owner: &'a CrateIdentity,
    pub rows: &'a g::Rows,
    pub types: &'a super::roots::Mapped,
}
pub(super) fn function(
    context: &Context<'_>,
    facts: &mut Declarations,
    function: &g::Function,
    headers: &mut Headers,
) -> Result<(DefId, Header), Error> {
    let Context {
        imported,
        catalog,
        owner,
        rows,
        types,
    } = *context;
    prepare(context, function, headers)?;
    let canonical = &types.nodes;
    let definition = imported
        .generic_definition(owner, function.declaration)
        .map_err(error)?;
    let declaration = catalog.declaration(definition).map_err(error)?;
    let origin = catalog
        .validate_declaration(declaration, definition, DefKind::Function)
        .map_err(error)?;
    let id = imported.generic_body(owner, function.body).map_err(error)?;
    let body = catalog.origin(id).map_err(error)?;
    catalog
        .validate_function_origin(definition, body)
        .map_err(error)?;
    let formals = formals(context, facts, function, headers)?;
    let inputs = inputs(context, declaration, function, facts, headers)?;
    let clauses: Vec<_> = inputs.iter().map(|i| i.value).collect();
    let abilities: Vec<_> = formals
        .ids
        .iter()
        .map(|&id| (id, Abilities::default()))
        .collect();
    let environment = Arc::new(
        Environment::new(&facts.types, &abilities)
            .map_err(error)?
            .with_clauses(&facts.types, &clauses)
            .map_err(error)?,
    );
    let empty = Environment::new(&facts.types, &[]).map_err(error)?;
    let allocation = environment.retain_owned_from(
        &empty,
        &mut crate::types::parameters::environment::Retained::default(),
    );
    headers.charge(allocation.visits)?;
    headers.observed.include(allocation);
    headers.observed.add(2 * size_of::<usize>());
    headers.observed.vector(&inputs);
    headers.observed.vector(&formals.parameters);
    let signature = canonical[function.original.0 as usize];
    let callable = canonical[function.callable.0 as usize];
    let header = Header {
        declaration: origin,
        body,
        signature,
        callable,
        environment,
        inputs,
        declared: function.declared_clauses as usize,
        constant: function.constant,
    };
    let g::Declaration::Function { identity, .. } =
        &rows.declarations[function.declaration.0 as usize]
    else {
        return Err(error(
            "imported function changed its external declaration kind",
        ));
    };
    let symbol = headers.symbol(identity)?;
    headers.observed.add(symbol.capacity());
    publish(facts, definition, &header, formals)?;
    facts
        .crates
        .get_or_insert_default()
        .symbols
        .insert(definition, symbol);
    Ok((definition, header))
}
fn prepare(
    context: &Context<'_>,
    function: &g::Function,
    headers: &mut Headers,
) -> Result<(), Error> {
    let parameters = (function.parameters.end - function.parameters.start) as usize;
    let clauses = (function.clauses.end - function.clauses.start) as usize;
    headers.charge(clauses)?;
    let (nodes, edges) = context.types.environment_bound(context.rows, function)?;
    let scratch = parameters
        .saturating_mul(
            4 * (size_of::<Parameter>()
                + size_of::<Argument>()
                + size_of::<crate::types::parameters::ParameterId>()
                + size_of::<(crate::types::parameters::ParameterId, Abilities)>()),
        )
        .saturating_add(clauses.saturating_mul(4 * size_of::<Clause>()));
    headers.reserve(
        scratch.saturating_add(Environment::builtin_allocation_bound(
            parameters, clauses, nodes, edges,
        )),
    )
}
struct Formals {
    parameters: Vec<Parameter>,
    arguments: Vec<Argument>,
    ids: Vec<crate::types::parameters::ParameterId>,
}
fn formals(
    context: &Context<'_>,
    facts: &Declarations,
    function: &g::Function,
    headers: &mut Headers,
) -> Result<Formals, Error> {
    let Context {
        imported,
        owner,
        rows,
        types,
        ..
    } = *context;
    let mut parameters = Vec::new();
    let mut arguments = Vec::new();
    let mut ids = Vec::new();
    for &reference in function.parameters.slice(&rows.parameters).map_err(error)? {
        headers.charge(1)?;
        let binding = imported
            .generic_definition(owner, reference)
            .map_err(error)?;
        let ty = types
            .parameters
            .get(reference.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| error("imported formal parameter has no canonical root"))?;
        let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err(error("imported formal is not a rigid parameter"));
        };
        ids.push(id);
        parameters.push(Parameter {
            binding,
            anonymous: false,
            argument: Argument::Type(ty),
            default: None,
        });
        arguments.push(Argument::Type(ty));
    }
    Ok(Formals {
        parameters,
        arguments,
        ids,
    })
}
fn publish(
    facts: &mut Declarations,
    definition: DefId,
    header: &Header,
    formals: Formals,
) -> Result<(), Error> {
    let Formals {
        parameters,
        arguments,
        ids,
    } = formals;
    let callable = header.callable;
    let environment = &header.environment;
    facts
        .lifetimes
        .callables
        .insert_generic(&facts.types, ItemId(definition.0), callable, 0, &[], &ids)
        .map_err(error)?;
    let arguments = facts.types.intern_arguments(&arguments).map_err(error)?;
    let ty = facts
        .types
        .intern(Kind::FunctionItem {
            item: ItemId(definition.0),
            arguments,
            signature: callable,
        })
        .map_err(error)?;
    let slot = facts
        .definitions
        .get_mut(definition.0 as usize)
        .ok_or_else(|| error("unknown imported function slot"))?;
    if slot.replace(ty).is_some()
        || facts
            .generics
            .parameters
            .insert(definition, parameters)
            .is_some()
    {
        return Err(error("imported generic declaration was already published"));
    }
    facts.lifetimes.functions.insert(
        definition,
        FunctionRegions {
            total: 0,
            captures: 0,
            early: Vec::new(),
            late: Vec::new(),
            has_declared_late: false,
            bounds: Vec::new(),
        },
    );
    facts
        .generics
        .environments
        .insert(definition, Arc::clone(environment));
    if header.constant {
        facts.constant_functions.insert(definition);
    }
    Ok(())
}

fn inputs(
    context: &Context<'_>,
    declaration: crate::resolve::body_origin::ImportedDeclarationId,
    function: &g::Function,
    facts: &mut Declarations,
    headers: &mut Headers,
) -> Result<Vec<Input>, Error> {
    let Context {
        catalog,
        rows,
        types,
        ..
    } = *context;
    let clauses = function.clauses.slice(&rows.clauses).map_err(error)?;
    let mut inputs = Vec::with_capacity(clauses.len());
    for clause in clauses {
        headers.charge(1)?;
        let predicate = Predicate {
            ty: types.nodes[clause.subject.0 as usize],
            bound: match clause.bound {
                g::Bound::Sized => Bound::Sized,
                g::Bound::Copy(g::CopyIdentity::LegacyBuiltin) => Bound::Copy,
                g::Bound::Copy(g::CopyIdentity::Source { .. }) => {
                    return Err(error(
                        "source Copy metadata requires its full source trait contract",
                    ));
                }
            },
        };
        let value = if let Some(binder) = clause.quantifier {
            Clause::ForAll(
                facts
                    .types
                    .intern_quantified(Quantified {
                        bound_regions: rows.binders[binder.0 as usize].count,
                        predicate,
                    })
                    .map_err(error)?,
            )
        } else {
            Clause::Predicate(predicate)
        };
        let site = catalog
            .declaration_site(declaration, clause.site)
            .map_err(error)?;
        inputs.push(Input {
            source: catalog
                .declaration_proof_site(declaration, site)
                .map_err(error)?,
            value,
        });
    }
    Ok(inputs)
}
