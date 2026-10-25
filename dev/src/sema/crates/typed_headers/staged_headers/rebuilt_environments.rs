//! Original environments retain exact declaration-versus-contract provenance.
use super::{Declarations, Error, Headers, Scope, error, mapped_source, s};
use crate::types::{
    Kind,
    arguments::Argument,
    parameters::{clauses::Clause, environment::Abilities},
};
use std::sync::Arc;

pub(super) fn complete(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    facts: &Declarations,
    scopes: &mut [Scope],
    budget: &mut Headers,
) -> Result<(), Error> {
    graph.require_store(&facts.types).map_err(error)?;
    let rows = context.descriptor.rows();
    for scope in scopes {
        budget.charge(1)?;
        let Some(callable) = &scope.function else {
            continue;
        };
        let Some(body) = callable.header.body else {
            continue;
        };
        let declaration = context
            .catalog
            .declaration(scope.definition)
            .map_err(error)?;
        let (owner, row, _) = context
            .catalog
            .declaration_coordinate(declaration)
            .map_err(error)?;
        if owner != context.descriptor.identity() {
            return Err(error(
                "source original environment belongs to a foreign artifact",
            ));
        }
        let Some(s::HeaderRef::Function(index)) = rows.header(row).map_err(error)? else {
            return Err(error(
                "source original environment lacks its actual function header",
            ));
        };
        let s::FunctionBody::Original(reference) = rows.functions[index as usize].body else {
            return Err(error(
                "source required method cannot acquire an original environment",
            ));
        };
        let id = context.catalog.body_at(owner, reference).map_err(error)?;
        if context.catalog.origin(id).map_err(error)? != body {
            return Err(error(
                "source original environment changed its actual body mapping",
            ));
        }
        let inputs = rows
            .body_inputs
            .get(reference.0 as usize)
            .filter(|inputs| inputs.body == reference)
            .ok_or_else(|| error("source original environment omitted its actual body inputs"))?;
        let environment = match inputs.environment {
            s::EnvironmentOrigin::Declaration(actual) if actual == row => {
                Arc::clone(&scope.environment)
            }
            s::EnvironmentOrigin::Contract => from_contract(graph, facts, scope, inputs, budget)?,
            _ => {
                return Err(error(
                    "source original environment origin requires its wider explicit profile",
                ));
            }
        };
        scope
            .function
            .as_mut()
            .ok_or_else(|| error("source original environment lost its actual callable"))?
            .header
            .environment = environment;
    }
    Ok(())
}

fn from_contract(
    graph: &mapped_source::Graph,
    facts: &Declarations,
    scope: &Scope,
    inputs: &s::BodyInputs,
    budget: &mut Headers,
) -> Result<Arc<crate::types::parameters::environment::Environment>, Error> {
    if scope.parent.is_some() || inputs.parameters.len() != scope.parameters.len() {
        return Err(error(
            "contract environment changed its complete original parameter domain",
        ));
    }
    budget.reserve(
        inputs
            .parameters
            .len()
            .saturating_mul(size_of::<(crate::types::parameters::ParameterId, Abilities)>())
            .saturating_add(
                inputs
                    .type_assumptions
                    .len()
                    .saturating_mul(size_of::<Clause>()),
            ),
    )?;
    let mut parameters = Vec::with_capacity(inputs.parameters.len());
    for (&slot, actual) in inputs.parameters.iter().zip(&scope.parameters) {
        budget.charge(1)?;
        let argument = graph.slot(slot)?;
        let Argument::Type(ty) = argument else {
            return Err(error(
                "contract environment contains a non-type original parameter",
            ));
        };
        let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err(error(
                "contract environment lost its actual rigid parameter identity",
            ));
        };
        if argument != actual.argument || actual.anonymous || actual.default.is_some() {
            return Err(error(
                "contract environment changed its canonical original slot binding",
            ));
        }
        parameters.push((id, Abilities::default()));
    }
    let mut clauses = Vec::with_capacity(inputs.type_assumptions.len());
    for input in &inputs.type_assumptions {
        budget.charge(1)?;
        clauses.push(Clause::Predicate(
            graph.requirement(input.value).map_err(error)?,
        ));
    }
    // The existing estimator rejects non-leaf/region/qualified forms before
    // constructing an environment; unsupported inputs never borrow scope assumptions.
    super::environments::prepare(facts, None, &parameters, &clauses, 0, 0, budget)
}
