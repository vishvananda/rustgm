//! Initialized source cleanup requests retain normal scoped const Destruct evidence.
use crate::{
    sema::{
        model::Declarations,
        nominals::traits::{Decision, State},
        traits::obligations::Obligations,
    },
    support::{bindings::LangItem, constness::Constness},
    types::{
        destruction::Eligibility,
        parameters::{environment::Environment, predicates::Predicate},
    },
};
use std::sync::Arc;
pub(crate) fn prove_cleanup_requirements(
    facts: &mut Declarations,
    environment: Option<Arc<Environment>>,
    universals: u32,
    first: u32,
    goals: &[(Eligibility, Predicate)],
) -> Result<Obligations, &'static str> {
    let (decisions, proof) =
        decide_cleanup_requirements(facts, environment, universals, first, goals)?;
    for (&decision, &(purpose, _)) in decisions.iter().zip(goals) {
        if decision == Decision::Absent {
            return Err(if purpose == Eligibility::OwnDrop {
                "active cleanup requires const Drop in its actual defining environment"
            } else {
                "active cleanup requires const Destruct in its actual defining environment"
            });
        }
    }
    Ok(proof)
}
/// A completed negative is a qualifier answer only, never positive cleanup proof.
pub(crate) fn decide_cleanup_requirements(
    facts: &mut Declarations,
    environment: Option<Arc<Environment>>,
    universals: u32,
    first: u32,
    goals: &[(Eligibility, Predicate)],
) -> Result<(Vec<Decision>, Obligations), &'static str> {
    if goals.len().saturating_mul(size_of::<Decision>()) > 64 * 1024 * 1024 {
        return Err("const qualification decisions exceed storage budget");
    }
    let mut decisions = Vec::new();
    decisions
        .try_reserve_exact(goals.len())
        .map_err(|_| "const qualification decision allocation failed")?;
    let mut state = State::body(facts, environment, universals, first)?;
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(state.selection_steps());
    for &(purpose, goal) in goals {
        validate_goal(facts, purpose, goal)?;
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(8);
        let before = state.selection_steps();
        let result = state.decide(facts, goal);
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(state.selection_steps().saturating_sub(before));
        decisions.push(result?);
    }
    Ok((decisions, state.finish()))
}

fn validate_goal(
    facts: &Declarations,
    purpose: Eligibility,
    goal: Predicate,
) -> Result<(), &'static str> {
    let Some((application, Constness::Const | Constness::Conditional)) = goal.bound.trait_parts()
    else {
        return Err("source cleanup requires its actual const effect");
    };
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source cleanup lost its catalog")?;
    let role = if purpose == Eligibility::OwnDrop {
        LangItem::Drop
    } else {
        LangItem::Destruct
    };
    let declaration = catalog
        .validated_language(role)?
        .ok_or("source cleanup requires its actual language declaration")?;
    let application = facts
        .types
        .trait_application(application)
        .ok_or("source cleanup application is missing")?;
    if application.item.0 != declaration.definition.0
        || facts
            .types
            .arguments(application.arguments)
            .is_none_or(|a| a.len() != 0)
    {
        return Err("source cleanup requires its actual empty language application");
    }
    if purpose == Eligibility::OwnDrop {
        catalog.require_source_destructors_prepared()?;
        let Some(crate::types::Kind::Adt(item, _)) = facts.types.kind(goal.ty) else {
            return Err("own Drop eligibility requires an actual nominal application");
        };
        if facts.destructors.item(item).is_none() {
            return Err("own Drop eligibility requires its actual registered implementation");
        }
    }
    Ok(())
}
