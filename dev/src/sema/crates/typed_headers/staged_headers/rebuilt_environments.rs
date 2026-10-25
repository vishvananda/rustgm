//! Original environments retain exact declaration-versus-contract provenance.
use super::{Declarations, Error, Headers, Scope, error, mapped_source, s};
use crate::types::{
    Kind,
    arguments::Argument,
    parameters::{clauses::Clause, environment::Abilities},
};
use crate::{
    sema::crates::typed_headers::recipe_cursor::Budget,
    support::crates::{
        generics::BinderRef,
        owner_catalog::{BodyRef, DeclarationRef},
    },
    types::parameters::ParameterId,
};
use std::sync::Arc;

/// A positive zero-content Contract instruction, bound to its actual raw body.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::sema::crates::typed_headers) struct EmptyContract {
    body: BodyRef,
    declaration: DeclarationRef,
    frame: BinderRef,
}
impl EmptyContract {
    pub(in crate::sema::crates::typed_headers) fn check(
        rows: s::ValidatedView<'_>,
        function: u32,
        header: s::FunctionHeader,
        scope: (usize, s::ParameterScope),
        budget: &mut impl Budget,
    ) -> Result<Option<Self>, Error> {
        budget.charge(14)?;
        if rows.functions.get(function as usize) != Some(&header)
            || rows.scopes.get(scope.0) != Some(&scope.1)
            || scope.1.owner != header.scope
        {
            return Err(error(
                "empty Contract changed its actual function or scope row",
            ));
        }
        let s::FunctionBody::Original(body) = header.body else {
            return Err(error("empty Contract requires its actual original body"));
        };
        let original = rows
            .originals
            .get(body.0 as usize)
            .ok_or_else(|| error("empty Contract lost its actual original"))?;
        let input = rows
            .body_inputs
            .get(body.0 as usize)
            .ok_or_else(|| error("empty Contract lost its actual raw inputs"))?;
        let unit = rows
            .body_units
            .get(body.0 as usize)
            .ok_or_else(|| error("empty Contract lost its actual numbering domain"))?;
        if original.body != body
            || original.declaration != header.declaration
            || input.body != body
            || input.frame != header.original.binder
            || unit.body != body
            || unit.universals != header.regions.total
        {
            return Err(error(
                "empty Contract changed its complete original identity",
            ));
        }
        for (signature, kind) in [
            (header.original, s::BinderKind::OriginalFunction),
            (header.callable, s::BinderKind::PublicFunction),
        ] {
            let binder = rows
                .binders
                .get(signature.binder.0 as usize)
                .ok_or_else(|| error("empty Contract lost its actual signature binder"))?;
            if binder.owner != header.declaration || binder.kind != kind {
                return Err(error("empty Contract changed its actual signature owner"));
            }
            if binder.parent.is_some() || binder.count != 0 {
                return Ok(None);
            }
        }
        if input.environment != s::EnvironmentOrigin::Contract
            || header.parent.is_some()
            || scope.1.parent.is_some()
            || scope.1.captures != 0
            || !scope
                .1
                .declared
                .slice(&rows.slots)
                .map_err(error)?
                .is_empty()
            || !header
                .inputs
                .slice(&rows.clauses)
                .map_err(error)?
                .is_empty()
            || header.regions.total != 0
            || header.regions.captures != 0
            || header.regions.early.start != header.regions.early.end
            || header.regions.late.start != header.regions.late.end
            || header.regions.bounds.start != header.regions.bounds.end
            || !input.parameters.is_empty()
            || !input.assumptions.is_empty()
            || !input.type_assumptions.is_empty()
        {
            return Ok(None);
        }
        Ok(Some(Self {
            body,
            declaration: header.declaration,
            frame: input.frame,
        }))
    }
}

/// Construction parts carry no body, caller, well-formedness or proof permission.
pub(in crate::sema::crates::typed_headers) struct ContractParts {
    parameters: Vec<(ParameterId, Abilities)>,
    clauses: Vec<Clause>,
}
impl ContractParts {
    pub(in crate::sema::crates::typed_headers) fn private_empty(_actual: EmptyContract) -> Self {
        Self {
            parameters: Vec::new(),
            clauses: Vec::new(),
        }
    }
    pub(in crate::sema::crates::typed_headers) fn build(
        self,
        facts: &Declarations,
        budget: &mut impl super::environments::Accounting,
    ) -> Result<Arc<crate::types::parameters::environment::Environment>, Error> {
        super::environments::prepare(facts, None, &self.parameters, &self.clauses, 0, 0, budget)
    }
}

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
    ContractParts::ordinary(graph, facts, scope, inputs, budget)?.build(facts, budget)
}
impl ContractParts {
    fn ordinary(
        graph: &mapped_source::Graph,
        facts: &Declarations,
        scope: &Scope,
        inputs: &s::BodyInputs,
        budget: &mut Headers,
    ) -> Result<Self, Error> {
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
        Ok(Self {
            parameters,
            clauses,
        })
    }
}
