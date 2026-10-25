//! Zero-domain body environments retain actual Declaration or Contract construction.
use super::{
    Allocation, Arc, Argument, DeclarationRef, Declarations, Demand, Dependency, Environment,
    Error, Observed, Owner, Port, Prepared, Progress, Ready, RecipeView, ReservationWork, Scope,
    Session, State, error, numbered, numbering_records, ports, s,
};
use crate::{
    sema::crates::typed_headers::staged_headers::rebuilt_environments::{
        ContractParts, EmptyContract,
    },
    sema::function_context::assumption_environment as defining,
    support::crates::{generics as g, owner_catalog::BodyRef},
    types::parameters::ParameterId,
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Origin {
    Declaration,
    Numbered,
    Contract(EmptyContract),
}
#[derive(Clone, Copy)]
pub(super) struct BodyDemand {
    pub expected: Ready,
    pub origin: Origin,
}

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
fn body_origin(
    rows: s::ValidatedView<'_>,
    expected: Ready,
    work: &mut ReservationWork,
) -> Result<(BodyRef, Option<Origin>), Error> {
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
    let origin = if !input.assumptions.is_empty() || !input.type_assumptions.is_empty() {
        None
    } else if unit.universals != 0 {
        if input.environment == s::EnvironmentOrigin::Declaration(expected.header.declaration)
            && numbered::signature(rows, expected, work)?
        {
            Some(Origin::Numbered)
        } else {
            None
        }
    } else {
        match input.environment {
            s::EnvironmentOrigin::Declaration(owner) if owner == expected.header.declaration => {
                Some(Origin::Declaration)
            }
            s::EnvironmentOrigin::Contract => EmptyContract::check(
                rows,
                expected.pair.function_row,
                expected.header,
                expected.scope,
                work,
            )?
            .map(Origin::Contract),
            _ => None,
        }
    };
    Ok((body, origin))
}

fn body_environment(
    facts: &Declarations,
    scope: &Scope,
    demand: &BodyDemand,
    work: &mut ReservationWork,
) -> Result<(Arc<Environment>, usize), Error> {
    match demand.origin {
        Origin::Declaration | Origin::Numbered => Ok((Arc::clone(&scope.environment), 0)),
        Origin::Contract(actual) => {
            scope.require_empty(demand.expected)?;
            let mut accounting = Observed {
                work,
                allocation: Allocation::default(),
            };
            let environment = ContractParts::private_empty(actual).build(facts, &mut accounting)?;
            if Arc::ptr_eq(&environment, &scope.environment) || !environment.is_neutral() {
                return Err(error(
                    "empty Contract substituted its declaration environment",
                ));
            }
            Ok((environment, accounting.allocation.bytes))
        }
    }
}

pub(super) struct Finished {
    pub environment: Arc<Environment>,
    pub bytes: usize,
    pub recording: Option<numbering_records::Key>,
}
pub(super) fn finish(
    facts: &mut Declarations,
    prepared: &Prepared,
    rows: s::ValidatedView<'_>,
    port: &mut ports::Inputs<'_, '_>,
    owner: &Owner,
    demand: &BodyDemand,
    work: &mut ReservationWork,
) -> Result<Finished, Error> {
    let BodyDemand { expected, origin } = *demand;
    let scope = owner
        .scopes
        .get(expected.scope.0)
        .and_then(Option::as_deref)
        .ok_or_else(|| error("private original lost its actual prepared scope"))?;
    let (body, actual) = body_origin(rows, expected, work)?;
    if actual != Some(origin)
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
    let (body_environment, bytes) = body_environment(facts, scope, demand, work)?;
    work.reserve(scope.parameters.len().saturating_mul(size_of::<Argument>()))?;
    let mut arguments = Vec::with_capacity(scope.parameters.len());
    for &slot in &input.parameters {
        let parameter = owner.original_parameter(rows, prepared, scope, slot, work)?;
        let argument = port.port.slot(&mut facts.types, slot, work)?;
        if crate::sema::generics::parameter_id(&facts.types, argument).map_err(error)? != parameter
        {
            return Err(error(
                "private defining environment changed its complete actual original parameter order",
            ));
        }
        arguments.push(argument);
    }
    let kernel = defining::Prepared::begin(
        defining::Context {
            types: &mut facts.types,
            requirements: &mut facts.lifetimes.nominal_requirements,
            variances: &facts.lifetimes.variances,
            callables: &prepared.templates,
        },
        defining::Scope {
            signature: Some(expected.pair.original),
            universals: if origin == Origin::Numbered {
                expected.header.regions.total
            } else {
                0
            },
            parameters: arguments.into_iter(),
            bounds: &[],
            clauses: scope.clauses.iter().copied(),
        },
        Some(Arc::clone(&body_environment)),
        &[],
    )
    .map_err(Error::unlocated)?;
    if origin == Origin::Numbered {
        let completed = owner.finish_numbered(port, facts, prepared, demand, kernel, work)?;
        work.retained = before.saturating_add(completed.bytes);
        return Ok(Finished {
            environment: completed.environment,
            bytes: completed.bytes,
            recording: Some(completed.key),
        });
    }
    if kernel.needs_aliases() {
        return Err(error(
            "private zero-domain defining environment unexpectedly requires aliases",
        ));
    }
    let completed = kernel.finish(0, Vec::new()).map_err(Error::unlocated)?;
    let environment = completed.environment.ok_or_else(|| {
        error("private defining environment lost its actual declaration assumptions")
    })?;
    if completed.changed || !Arc::ptr_eq(&environment, &body_environment) {
        return Err(error(
            "private zero-domain defining environment changed its actual declaration construction",
        ));
    }
    if bytes > work.retained.saturating_sub(before) {
        return Err(error(
            "empty Contract exceeded its actual environment prepayment",
        ));
    }
    work.retained = before.saturating_add(bytes);
    Ok(Finished {
        environment,
        bytes,
        recording: None,
    })
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
        let numbered = expected.header.regions.total != 0
            && self
                .environments
                .as_deref()
                .map(|owner| owner.numbered_hint(expected, work))
                .transpose()?
                .unwrap_or(false);
        let origin = if numbered {
            Some(Origin::Numbered)
        } else {
            body_origin(rows, expected, work)?.1
        };
        let Some(origin) = origin else {
            return Ok(Demand::Blocked(Dependency::OwnedBodyDomain(
                expected.pair.body.ok_or_else(|| {
                    error("private defining demand lacks its actual original body identity")
                })?,
            )));
        };
        if origin == Origin::Numbered {
            self.record_selected(&view, &facts.types, &expected, work)?;
        }
        let mut environments = if let Some(owner) = self.environments.take() {
            owner
        } else {
            Box::new(Owner::new(rows, work)?)
        };
        let status = environments.complete(
            view,
            state,
            facts,
            self,
            &BodyDemand { expected, origin },
            work,
        )?;
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

impl Owner {
    fn original_parameter(
        &self,
        rows: s::ValidatedView<'_>,
        prepared: &Prepared,
        leaf: &Scope,
        reference: s::SlotRef,
        work: &mut ReservationWork,
    ) -> Result<ParameterId, Error> {
        work.charge(12)?;
        let slot = rows
            .slots
            .get(reference.0 as usize)
            .ok_or_else(|| error("private original lost its actual formal row"))?;
        let (index, raw) = prepared.scope(rows, slot.owner)?;
        let declaring = self
            .scopes
            .get(index)
            .and_then(Option::as_deref)
            .filter(|actual| actual.raw == raw)
            .ok_or_else(|| error("private original lost its actual declaring scope"))?;
        let inherited = if let Some(parent) = raw.parent {
            let (index, parent_raw) = prepared.scope(rows, parent)?;
            self.scopes
                .get(index)
                .and_then(Option::as_deref)
                .filter(|actual| actual.raw == parent_raw)
                .ok_or_else(|| error("private original lost its actual parent scope"))?
                .parameters
                .len()
        } else {
            0
        };
        let own = declaring
            .parameters
            .len()
            .checked_sub(inherited)
            .ok_or_else(|| error("private original changed its complete formal ancestry"))?;
        if !matches!(slot.kind, s::ParameterKind::Type { .. })
            || reference.0 < raw.declared.start
            || reference.0 >= raw.declared.end
            || slot.ordinal != reference.0 - raw.declared.start
            || slot.index != slot.ordinal
            || own != raw.declared.slice(&rows.slots).map_err(error)?.len()
            || slot.index as usize >= own
        {
            return Err(error(
                "private original requires its exact actual type formal scope",
            ));
        }
        let position = inherited
            .checked_add(slot.index as usize)
            .ok_or_else(|| error("private original formal ordinal overflow"))?;
        let parameter = declaring
            .parameters
            .get(position)
            .map(|(parameter, _)| *parameter)
            .ok_or_else(|| error("private original omitted its actual declared formal"))?;
        if leaf
            .parameters
            .get(position)
            .map(|(parameter, _)| *parameter)
            != Some(parameter)
        {
            return Err(error(
                "private original changed its exact inherited formal identity",
            ));
        }
        Ok(parameter)
    }
}
