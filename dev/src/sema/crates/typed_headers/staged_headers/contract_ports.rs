//! Move the authenticated canonical prefix into ordinary trait contracts.
use super::{Error, Headers, Inputs, Scope, error, mapped_source};
use crate::{
    resolve::model::DefId,
    sema::traits::{Contracts, Header, inherited_inputs::HeaderInputs},
    types::{
        ItemId, Store,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};

pub(super) fn prepare(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    store: &Store,
    scopes: &mut [Scope],
    budget: &mut Headers,
) -> Result<(Contracts, Vec<(DefId, Predicate)>), Error> {
    let rows = context.descriptor.rows();
    budget.reserve(
        size_of::<Contracts>()
            .saturating_add(
                rows.traits
                    .len()
                    .saturating_mul(size_of::<(DefId, Predicate)>()),
            )
            .saturating_add(crate::support::retained::new_map::<DefId, Predicate>(
                rows.traits.len(),
            )),
    )?;
    let mut contracts = Contracts::default();
    let mut self_bounds = Vec::with_capacity(rows.traits.len());
    for (raw, scope) in rows.scopes.iter().zip(scopes) {
        let Inputs::Trait(_) = &scope.inputs else {
            continue;
        };
        budget.charge(1)?;
        let Some(super::s::HeaderRef::Trait(index)) = rows.header(raw.owner).map_err(error)? else {
            return Err(error(
                "source canonical trait contract lost its actual header kind",
            ));
        };
        let header = &rows.traits[index as usize];
        let self_type = graph.ty(header.self_type)?;
        self_bounds.push((
            scope.definition,
            implicit_self(store, scope, header, self_type, budget)?,
        ));
        let bounds = header
            .region_bounds
            .slice(&rows.region_bounds)
            .map_err(error)?;
        budget.reserve(
            crate::support::retained::new_map::<ItemId, Header>(1)
                .saturating_add(size_of::<ItemId>())
                .saturating_add(
                    scope
                        .parameters
                        .len()
                        .saturating_mul(size_of::<crate::sema::generics::Parameter>()),
                )
                .saturating_add(
                    bounds
                        .len()
                        .saturating_mul(size_of::<crate::types::outlives::Pair>()),
                ),
        )?;
        let regions = bounds
            .iter()
            .map(|bound| {
                budget.charge(1)?;
                Ok((graph.region(bound.longer)?, graph.region(bound.shorter)?))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let parameters = scope.parameters.clone();
        budget.observed.vector(&parameters);
        budget.observed.vector(&regions);
        let Inputs::Trait(inputs) = std::mem::replace(&mut scope.inputs, Inputs::SelfType) else {
            return Err(error(
                "source trait input ownership changed during canonical staging",
            ));
        };
        let inputs = HeaderInputs::imported(inputs);
        budget.charge(1)?;
        inputs
            .require_catalog(context.catalog, scope.definition)
            .map_err(error)?;
        contracts
            .declare(
                ItemId(scope.definition.0),
                Header {
                    self_type,
                    parameters,
                    inputs,
                    results: Vec::new(),
                    regions,
                },
            )
            .map_err(error)?;
    }
    budget.observed.add(size_of::<Contracts>());
    let allocation = contracts.header_table_allocation();
    budget.charge(allocation.visits)?;
    budget.observed.include(allocation);
    budget.charge(1)?;
    Ok((contracts, self_bounds))
}

fn implicit_self(
    store: &Store,
    scope: &Scope,
    raw: &super::s::TraitHeader,
    self_type: crate::types::TypeId,
    budget: &mut Headers,
) -> Result<Predicate, Error> {
    budget.charge(1)?;
    if raw.declared_inputs <= raw.canonical_inputs {
        return Err(error(
            "source trait omitted its actual declared implicit Self input",
        ));
    }
    let input = scope
        .inputs
        .all()
        .get(raw.canonical_inputs as usize)
        .ok_or_else(|| {
            error("source implicit Self input is outside its full declaration assumptions")
        })?;
    let Clause::Predicate(predicate) = input.value else {
        return Err(error(
            "source implicit Self input changed its actual clause kind",
        ));
    };
    let Bound::Trait(application) = predicate.bound else {
        return Err(error(
            "source implicit Self input changed its actual ordinary effect",
        ));
    };
    let application = store
        .trait_application(application)
        .ok_or_else(|| error("source implicit Self input has no full actual application"))?;
    let arguments = store
        .arguments(application.arguments)
        .ok_or_else(|| error("source implicit Self input lost its actual formal arguments"))?;
    budget.charge(arguments.len())?;
    if predicate.ty != self_type
        || application.item != ItemId(scope.definition.0)
        || !arguments.eq(scope.parameters.iter().map(|parameter| parameter.argument))
    {
        return Err(error(
            "source implicit Self input changed its exact trait/formal application",
        ));
    }
    Ok(predicate)
}
