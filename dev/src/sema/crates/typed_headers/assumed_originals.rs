//! Only actual zero-domain Declaration originals can acquire private readiness.
use super::{
    Arc, Argument, Budget, DeclarationRef, Declarations, Demand, Dependency, Environment, Error,
    Owner, Port, Prepared, Progress, Ready, RecipeView, ReservationWork, Scope, Session, State,
    error, ports, s,
};
use crate::{
    sema::function_context::assumption_environment as defining,
    support::crates::{generics as g, owner_catalog::BodyRef},
    types::callable::Templates,
};

pub(super) fn input_range(
    rows: s::ValidatedView<'_>,
    scope: s::ParameterScope,
) -> Result<(g::Range, Option<usize>), Error> {
    match rows.header(scope.owner).map_err(error)? {
        Some(s::HeaderRef::Function(index)) => Ok((rows.functions[index as usize].inputs, None)),
        Some(s::HeaderRef::Trait(index))
            if scope.parent == Some(rows.traits[index as usize].self_definition) =>
        {
            let header = rows.traits[index as usize];
            Ok((header.inputs, Some(header.canonical_inputs as usize)))
        }
        Some(s::HeaderRef::Implementation(index)) if scope.parent.is_none() => {
            Ok((rows.implementations[index as usize].inputs, None))
        }
        None if scope.parent.is_none() => {
            let actual = rows
                .declarations
                .get(scope.owner.0 as usize)
                .ok_or_else(|| error("private Self environment lost its actual declaration"))?;
            let parent = actual
                .definition
                .parent
                .ok_or_else(|| error("private Self environment lacks its actual trait"))?;
            let Some(s::HeaderRef::Trait(index)) = rows.header(parent).map_err(error)? else {
                return Err(error(
                    "private Self environment lacks its actual trait header",
                ));
            };
            if actual.definition.kind != s::DefinitionKind::SelfType
                || rows.traits[index as usize].self_definition != scope.owner
            {
                return Err(error(
                    "private Self environment changed its actual schema owner",
                ));
            }
            // The actual Self scope has no own clauses; its trait's complete
            // canonical inputs remain on the following real trait scope.
            Ok((g::Range { start: 0, end: 0 }, None))
        }
        _ => Err(error(
            "private environment requires its later actual scope profile",
        )),
    }
}
pub(super) fn leaf_path(
    rows: s::ValidatedView<'_>,
    path: &[usize],
    work: &mut ReservationWork,
) -> Result<bool, Error> {
    for &index in path {
        work.charge(1)?;
        let (range, _) = input_range(rows, rows.scopes[index])?;
        let clauses = range.slice(&rows.clauses).map_err(error)?;
        work.charge(clauses.len())?;
        for clause in clauses {
            let s::Requirement::Trait {
                application,
                effect: crate::support::constness::Constness::Ordinary,
            } = clause.requirement
            else {
                return Ok(false);
            };
            if clause.binder.is_some() {
                return Ok(false);
            }
            let application = rows
                .trait_applications
                .get(application.0 as usize)
                .ok_or_else(|| {
                    error("private environment has an unknown actual trait application")
                })?;
            let arguments = application
                .arguments
                .slice(&rows.arguments)
                .map_err(error)?;
            work.charge(arguments.len().saturating_add(1))?;
            let leaf = |reference: g::TypeRef| {
                matches!(
                    rows.types.get(reference.0 as usize),
                    Some(s::Type::Scalar(_) | s::Type::Parameter(_))
                )
            };
            if !leaf(application.subject)
                || !arguments
                    .iter()
                    .all(|argument| matches!(argument, s::Argument::Type(ty) if leaf(*ty)))
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
fn zero_body(
    rows: s::ValidatedView<'_>,
    expected: Ready,
    work: &mut ReservationWork,
) -> Result<(BodyRef, bool), Error> {
    work.charge(6)?;
    let s::FunctionBody::Original(body) = expected.header.body else {
        return Err(error(
            "private defining environment requires an actual original body",
        ));
    };
    let original = rows
        .originals
        .get(body.0 as usize)
        .ok_or_else(|| error("private defining environment lost its actual original"))?;
    let input = rows
        .body_inputs
        .get(body.0 as usize)
        .ok_or_else(|| error("private defining environment lost its raw body inputs"))?;
    let unit = rows.body_units.get(body.0 as usize).ok_or_else(|| {
        error("private defining environment lost its complete actual numbering unit")
    })?;
    if original.body != body
        || original.declaration != expected.header.declaration
        || input.body != body
        || input.frame != expected.header.original.binder
        || unit.body != body
        || unit.universals != expected.header.regions.total
        || input.parameters.len() != expected.formals
    {
        return Err(error(
            "private defining environment changed its actual header/body/domain identity",
        ));
    }
    Ok((
        body,
        unit.universals == 0
            && expected.header.regions.total == 0
            && input.environment == s::EnvironmentOrigin::Declaration(expected.header.declaration)
            && input.assumptions.is_empty()
            && input.type_assumptions.is_empty(),
    ))
}

pub(super) fn finish(
    facts: &mut Declarations,
    templates: &Templates,
    rows: s::ValidatedView<'_>,
    port: &mut ports::Inputs<'_, '_>,
    scope: &Scope,
    expected: Ready,
    work: &mut ReservationWork,
) -> Result<Arc<Environment>, Error> {
    let (body, zero) = zero_body(rows, expected, work)?;
    if !zero
        || scope.parameters.len() != expected.formals
        || scope.environment.has_region_builtins()
    {
        return Err(error(
            "private defining environment requires its later regional producer",
        ));
    }
    let input = &rows.body_inputs[body.0 as usize];
    work.charge(input.parameters.len().saturating_add(1))?;
    let before = work.retained;
    work.reserve(scope.parameters.len().saturating_mul(size_of::<Argument>()))?;
    let mut arguments = Vec::with_capacity(scope.parameters.len());
    for (&slot, (parameter, _)) in input.parameters.iter().zip(&scope.parameters) {
        let argument = port.port.slot(&mut facts.types, slot, work)?;
        if crate::sema::generics::parameter_id(&facts.types, argument).map_err(error)? != *parameter
        {
            return Err(error(
                "private defining environment changed its complete actual original parameter order",
            ));
        }
        arguments.push(argument);
    }
    let prepared = defining::Prepared::begin(
        defining::Context {
            types: &mut facts.types,
            requirements: &mut facts.lifetimes.nominal_requirements,
            variances: &facts.lifetimes.variances,
            callables: templates,
        },
        defining::Scope {
            signature: Some(expected.pair.original),
            universals: 0,
            parameters: arguments.into_iter(),
            bounds: &[],
            clauses: scope.clauses.iter().copied(),
        },
        Some(Arc::clone(&scope.environment)),
        &[],
    )
    .map_err(Error::unlocated)?;
    if prepared.needs_aliases() {
        return Err(error(
            "private zero-domain defining environment unexpectedly requires aliases",
        ));
    }
    let completed = prepared.finish(0, Vec::new()).map_err(Error::unlocated)?;
    let environment = completed.environment.ok_or_else(|| {
        error("private defining environment lost its actual declaration assumptions")
    })?;
    if completed.changed || !Arc::ptr_eq(&environment, &scope.environment) {
        return Err(error(
            "private zero-domain defining environment changed its actual declaration construction",
        ));
    }
    work.retained = before;
    Ok(environment)
}
impl Prepared {
    fn defining(
        &mut self,
        view: RecipeView<'_>,
        state: &mut State,
        facts: &mut Declarations,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<Demand<()>, Error> {
        let rows = view.rows()?;
        work.charge(1)?;
        let Some(s::HeaderRef::Function(index)) = rows.header(owner).map_err(error)? else {
            return Err(error("private defining demand lacks its actual function"));
        };
        let expected = self
            .ready
            .get(index as usize)
            .copied()
            .flatten()
            .ok_or_else(|| {
                error("private defining demand lacks its actual prepared named callable")
            })?;
        self.require(&view, &facts.types, owner, expected.pair, work)?;
        let (_, zero) = zero_body(rows, expected, work)?;
        if !zero {
            return Ok(Demand::Blocked(Dependency::OwnedBodyDomain(
                expected.pair.body.ok_or_else(|| {
                    error("private defining demand lacks its actual original body identity")
                })?,
            )));
        }
        let mut environments = if let Some(owner) = self.environments.take() {
            owner
        } else {
            Box::new(Owner::new(rows, work)?)
        };
        let status = environments.complete(view, state, facts, self, expected, work)?;
        self.environments = Some(environments);
        Ok(status)
    }
}
impl Session {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private defining environments await actual nonzero body domains/original proof; public Provided and roles stay closed"
        )
    )]
    pub(in crate::sema::crates::typed_headers::unpaid_provided) fn defining_environment(
        self: Box<Self>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<Progress, Error> {
        let (mut session, status) = self.callable_template(owner, work)?;
        if status != Demand::Ready(()) {
            return Ok((session, status));
        }
        session.require_owner(work)?;
        let view = session
            .inputs
            .recipe_view(&session.inventory, session.provider_index, work)?;
        let state = session
            .recipes
            .as_mut()
            .ok_or_else(|| error("private defining demand lost its actual recipe owner"))?;
        let mut prepared = state
            .callables
            .take()
            .ok_or_else(|| error("private defining demand lost its actual template owner"))?;
        let status = prepared.defining(view, state, &mut session.facts, owner, work)?;
        state.callables = Some(prepared);
        Ok((session, status))
    }
}
