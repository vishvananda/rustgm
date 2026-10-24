//! Environment provenance and Copy queries remain separate from required goals.
use super::{Context, Declarations, DefId, Sites, Stage, s};
use crate::{
    sema::associated::OwnerKind,
    types::{parameters::environment::Environment, parameters::predicates::Bound},
};
use std::sync::Arc;

fn validate_environment(
    facts: &Declarations,
    definition: DefId,
    environment: &Arc<Environment>,
) -> Result<(), &'static str> {
    if facts
        .generics
        .body_environment(definition)
        .is_none_or(|actual| !Arc::ptr_eq(actual, environment))
    {
        return Err("source original environment is not its actual retained declaration input");
    }
    Ok(())
}

fn translated_environment(
    stage: &mut Stage,
    facts: &Declarations,
    definition: DefId,
    environment: Option<&Arc<Environment>>,
) -> Result<s::EnvironmentOrigin, &'static str> {
    let Some(environment) = environment else {
        return Ok(s::EnvironmentOrigin::Absent);
    };
    if facts.generics.body_environment(definition).is_none() {
        return Ok(s::EnvironmentOrigin::Contract);
    }
    validate_environment(facts, definition, environment)?;
    let member = stage.declaration(definition)?;
    if let Some(owner) = facts
        .associated
        .owner(definition)
        .and_then(|owner| facts.associated.get(owner))
        && let OwnerKind::Trait {
            definition: trait_,
            constant: true,
            ..
        } = owner.kind
    {
        let self_type = facts
            .associated
            .schema(trait_)
            .ok_or("source const default has no actual Self schema")?
            .self_type;
        let mut found = false;
        for clause in environment.trait_clauses() {
            stage.charge(1)?;
            let predicate = clause.ordinary().map_err(
                |_| "source const default local premise must retain its actual unquantified owner",
            )?;
            if predicate.ty == self_type
                && let Bound::MaybeConstTrait(application) = predicate.bound
            {
                let actual = facts
                    .types
                    .trait_application(application)
                    .ok_or("source local Self application is unknown")?;
                if actual.item.0 == trait_.0
                    && facts
                        .types
                        .arguments(actual.arguments)
                        .is_some_and(|arguments| arguments.len() == 0)
                {
                    found = true;
                }
            }
        }
        if found {
            return Ok(s::EnvironmentOrigin::ConstTraitDefault {
                member,
                trait_: stage.address(trait_)?,
            });
        }
    }
    Ok(s::EnvironmentOrigin::Declaration(member))
}

pub(super) fn body_environment(
    stage: &mut Stage,
    facts: &Declarations,
    definition: DefId,
) -> Result<s::EnvironmentOrigin, &'static str> {
    match facts.generics.body_environment(definition) {
        Some(environment) => translated_environment(stage, facts, definition, Some(environment)),
        None => Ok(s::EnvironmentOrigin::Contract),
    }
}

pub(super) fn emit(
    stage: &mut Stage,
    facts: &Declarations,
    definition: DefId,
    builtins: &crate::body::regions::BuiltinRequirements,
    context: Context,
    sites: &mut Sites,
) -> Result<s::BuiltinInputs, &'static str> {
    let environment =
        translated_environment(stage, facts, definition, builtins.environment.as_ref())?;
    let mut rows = s::BuiltinInputs {
        environment,
        predicates: Vec::new(),
        source_copy: None,
    };
    for &(site, predicate) in &builtins.predicates {
        stage.reserve(4 * size_of::<s::LocatedRequirement>())?;
        rows.predicates.push(s::LocatedRequirement {
            site: sites.input(stage, site)?,
            value: stage.requirement(facts, predicate, context)?,
        });
    }
    if let Some(copy) = &builtins.source_copy {
        let actual = facts
            .source_bindings
            .as_deref()
            .ok_or("source Copy query has no actual source catalog")?
            .validated_language(crate::support::bindings::LangItem::Copy)?
            .map(|declaration| declaration.definition);
        if copy.declaration != actual {
            return Err("source Copy query belongs to another actual declaration");
        }
        stage.reserve(size_of::<s::CopyQueries>())?;
        let declaration = copy
            .declaration
            .map(|definition| stage.address(definition))
            .transpose()?;
        let mut queries = s::CopyQueries {
            declaration,
            goals: Vec::new(),
        };
        for &predicate in &copy.goals {
            stage.reserve(4 * size_of::<s::Requirement>())?;
            queries
                .goals
                .push(stage.requirement(facts, predicate, context)?);
        }
        rows.source_copy = Some(queries);
    }
    Ok(rows)
}
