//! Scoped body opening consumes only the current producer's exact environment.
use super::{Arc, Body, Contract, Declarations, Environment};
use crate::{body::ir::Owner, sema::model::BodyRegionOrigin};

pub(super) fn check(
    facts: &mut Declarations,
    body: &Body,
    contract: &Contract,
    environment: Option<&Arc<Environment>>,
) -> Result<(), &'static str> {
    facts.body_work.canonical_steps += 1;
    let actual = body
        .source
        .source()
        .and_then(|source| facts.generics.builtin_requirements.get(&source))
        .and_then(|requirements| requirements.environment.as_ref());
    if environment.is_none_or(|environment| environment.body_aliases().is_none())
        && actual.is_none_or(|environment| environment.body_aliases().is_none())
    {
        return Ok(());
    }
    let environment = environment.ok_or("body alias opening lost its environment")?;
    if body.instance.is_some() || actual.is_none_or(|actual| !Arc::ptr_eq(actual, environment)) {
        return Err("body alias opening belongs to another original or environment");
    }
    let source = body
        .source
        .source()
        .ok_or("body alias opening has no source owner")?;
    let aliases = environment
        .body_aliases()
        .ok_or("body alias opening lost the actual producer's slot mapping")?;
    if let Owner::Function {
        definition,
        signature,
        execution,
    } = body.owner
    {
        facts.body_work.canonical_steps += 1;
        let header = facts
            .function_header(definition)?
            .ok_or("body alias owner is not prepared")?;
        if header.body != Some(body.source)
            || signature != header.signature
            || execution.is_some()
            || header.universals != contract.universals
            || header
                .environment
                .is_none_or(|actual| !Arc::ptr_eq(actual, environment))
        {
            return Err("body alias opening changed its actual function header");
        }
    }
    facts.body_work.canonical_steps += aliases.len();
    if aliases.len() > 1_000_000 || aliases.len() != contract.universals as usize {
        return Err("body alias opening has an incomplete declaration slot vector");
    }
    for &alias in aliases.ordered() {
        if facts.body_regions.get(source, alias).is_none_or(|region| {
            region.body != source || !matches!(region.origin, BodyRegionOrigin::UniversalAlias)
        }) {
            return Err("body alias opening contains a foreign or unprepared body region");
        }
    }
    Ok(())
}
