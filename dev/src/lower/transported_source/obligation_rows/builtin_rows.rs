//! Environment provenance and Copy queries remain separate from required goals.
use super::{Context, Declarations, DefId, Sites, Stage, s};
use crate::{sema::associated::OwnerKind, types::parameters::environment::Environment};
use std::sync::Arc;

fn validate_environment(
    facts: &Declarations,
    definition: DefId,
    body_source: Option<crate::resolve::unit::NodeKey>,
    environment: &Arc<Environment>,
) -> Result<(), &'static str> {
    if let Some(source) = body_source
        && facts
            .generics
            .builtin_requirements
            .get(&source)
            .and_then(|requirements| requirements.environment.as_ref())
            .is_some_and(|actual| Arc::ptr_eq(actual, environment))
    {
        return Ok(());
    }
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
    body_source: Option<crate::resolve::unit::NodeKey>,
    environment: Option<&Arc<Environment>>,
) -> Result<s::EnvironmentOrigin, &'static str> {
    let Some(environment) = environment else {
        return Ok(s::EnvironmentOrigin::Absent);
    };
    if facts.generics.body_environment(definition).is_none() {
        return Ok(s::EnvironmentOrigin::Contract);
    }
    validate_environment(facts, definition, body_source, environment)?;
    // A completed body cache does not turn a contract-derived scope into a
    // declaration environment. Feature premises still require explicit rows.
    if !facts.generics.environments.contains_key(&definition)
        && !environment.has_feature_assumptions()
    {
        return Ok(s::EnvironmentOrigin::Contract);
    }
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
        let defining = facts.defining_self_clause(trait_)?;
        for clause in environment.trait_clauses()? {
            stage.charge(1)?;
            if clause == defining {
                return Ok(s::EnvironmentOrigin::ConstTraitDefault {
                    member,
                    trait_: stage.address(trait_)?,
                });
            }
        }
    }
    Ok(s::EnvironmentOrigin::Declaration(member))
}

// Without a source-retained builtin environment, anonymous Copy preparation
// constructs its environment from this body's complete contract, not the
// enclosing declaration. Import must replay that same construction.
fn anonymous_contract(facts: &Declarations, cfg: &super::Body) -> bool {
    matches!(cfg.owner, super::ir::Owner::AnonymousConstant)
        && cfg
            .source
            .source()
            .and_then(|source| facts.generics.builtin_requirements.get(&source))
            .and_then(|input| input.environment.as_ref())
            .is_none()
}

pub(super) fn body_environment(
    stage: &mut Stage,
    facts: &Declarations,
    definition: DefId,
    cfg: &super::Body,
) -> Result<s::EnvironmentOrigin, &'static str> {
    if anonymous_contract(facts, cfg) {
        return Ok(s::EnvironmentOrigin::Contract);
    }
    match facts.generics.body_environment(definition) {
        Some(environment) => {
            translated_environment(stage, facts, definition, None, Some(environment))
        }
        None => Ok(s::EnvironmentOrigin::Contract),
    }
}

pub(super) fn emit(
    stage: &mut Stage,
    facts: &Declarations,
    cfg: &super::Body,
    definition: DefId,
    builtins: &crate::body::regions::BuiltinRequirements,
    context: Context,
    sites: &mut Sites,
) -> Result<s::BuiltinInputs, &'static str> {
    let body_source = matches!(
        cfg.owner,
        super::ir::Owner::Closure { .. } | super::ir::Owner::AnonymousConstant
    )
    .then(|| cfg.source.source())
    .flatten();
    let environment = if anonymous_contract(facts, cfg) {
        if builtins
            .environment
            .as_ref()
            .is_some_and(|env| env.has_feature_assumptions())
        {
            return Err("source anonymous contract cannot fabricate defining feature premises");
        }
        builtins
            .environment
            .as_ref()
            .map_or(s::EnvironmentOrigin::Absent, |_| {
                s::EnvironmentOrigin::Contract
            })
    } else {
        translated_environment(
            stage,
            facts,
            definition,
            body_source,
            builtins.environment.as_ref(),
        )?
    };
    let mut rows = s::BuiltinInputs {
        environment,
        predicates: Vec::new(),
        source_copy: None,
    };
    for &(site, predicate) in &builtins.predicates {
        stage.reserve(4 * size_of::<s::LocatedRequirement>())?;
        rows.predicates.push(s::LocatedRequirement {
            site: sites.input(stage, site)?,
            binder: None,
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
