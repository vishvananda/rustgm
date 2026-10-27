//! Source copying is selected in one immutable caller; no physical fallback applies.
use crate::{
    sema::{model::Declarations, nominals::traits::State, traits::obligations::Obligations},
    support::bindings::LangItem,
    types::{
        TypeId,
        parameters::{environment::Environment, predicates::Predicate},
    },
};
use std::sync::Arc;

pub(crate) struct Decisions {
    pub choices: Vec<(TypeId, bool)>,
    pub proof: Obligations,
}

/// Identity only for a positively empty set of reads; decisions still use the
/// completed declaration path below. Pending is never converted to absence.
pub(crate) fn reserved_copy_declaration(
    facts: &mut Declarations,
) -> Result<Option<crate::resolve::model::DefId>, &'static str> {
    facts.body_work.canonical_steps += 1;
    facts
        .source_bindings
        .as_deref()
        .ok_or("source Copy has no source policy")?
        .role(LangItem::Copy)
        .map(|role| role.map(|role| role.definition))
}

/// Resolve actual declaration data only; every Copy decision still needs its
/// own caller-scoped selection and complete lifetime proof below.
pub(crate) fn copy_declaration(
    facts: &mut Declarations,
) -> Result<Option<crate::resolve::model::DefId>, &'static str> {
    facts.body_work.canonical_steps += 1;
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("source Copy has no source policy")?
        .validated_role(LangItem::Copy)?;
    if let Some(role) = role
        && matches!(
            role.origin,
            crate::resolve::body_origin::DeclarationOrigin::Imported(_)
        )
    {
        facts.associated.require_imported_role(role)?;
    }
    Ok(role.map(|role| role.definition))
}

pub(crate) fn copy_decisions(
    facts: &mut Declarations,
    environment: Option<Arc<Environment>>,
    universals: u32,
    first: u32,
    goals: &[Predicate],
) -> Result<Decisions, &'static str> {
    decisions(
        facts,
        environment,
        universals,
        first,
        goals,
        |state, facts, goal| state.copy_decision(facts, goal),
    )
}
pub(crate) fn copy_decisions_prepared(
    facts: &mut Declarations,
    environment: Option<Arc<Environment>>,
    universals: u32,
    first: u32,
    goals: &[Predicate],
    preparation: &mut dyn crate::sema::dispatch::Preparation,
) -> Result<Decisions, crate::sema::dispatch::Failure> {
    decisions(
        facts,
        environment,
        universals,
        first,
        goals,
        |state, facts, goal| state.prove_prepared(facts, goal, preparation),
    )
}
fn decisions<E: From<&'static str>>(
    facts: &mut Declarations,
    environment: Option<Arc<Environment>>,
    universals: u32,
    first: u32,
    goals: &[Predicate],
    mut select: impl FnMut(&mut State, &mut Declarations, Predicate) -> Result<bool, E>,
) -> Result<Decisions, E> {
    let declaration = copy_declaration(facts)?.ok_or("source Copy language item is missing")?;
    let mut state = State::body(facts, environment, universals, first)?;
    facts.body_work.canonical_steps += state.selection_steps();
    let mut decisions = Vec::with_capacity(goals.len());
    for &goal in goals {
        if !matches!(goal.bound, crate::types::parameters::predicates::Bound::Trait(id)
            if facts.types.trait_application(id).is_some_and(|a| a.item.0 == declaration.0
                && facts.types.arguments(a.arguments).is_some_and(|args| args.len() == 0)))
        {
            return Err("source Copy requirement has a different declaration or arguments".into());
        }
        let before = state.selection_steps();
        let selected = select(&mut state, facts, goal);
        facts.body_work.canonical_steps += state.selection_steps().saturating_sub(before);
        decisions.push((goal.ty, selected?));
    }
    Ok(Decisions {
        choices: decisions,
        proof: state.finish(),
    })
}

/// Discharge ordinary typed requests in one caller scope. The returned lifetime
/// and formation debts still require the receiving body's ordinary region proof.
pub(crate) fn prove_body_requirements(
    facts: &mut Declarations,
    environment: Option<Arc<Environment>>,
    universals: u32,
    first: u32,
    goals: impl Iterator<Item = Predicate>,
) -> Result<Obligations, &'static str> {
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
    if goals.size_hint().0 > crate::support::crates::MAX_COMPONENTS {
        return Err("body trait requirement work budget exceeded");
    }
    let mut state = State::body(facts, environment, universals, first)?;
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(state.selection_steps());
    for (index, goal) in goals.enumerate() {
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
        if index >= crate::support::crates::MAX_COMPONENTS {
            return Err("body trait requirement work budget exceeded");
        }
        if !matches!(
            goal.bound,
            crate::types::parameters::predicates::Bound::Trait(_)
        ) {
            return Err("body trait requirement has an unsupported effect or kind");
        }
        let before = state.selection_steps();
        let selected = state.prove(facts, goal, false);
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(state.selection_steps().saturating_sub(before));
        if !selected? {
            return Err("body trait requirement is not proved");
        }
    }
    Ok(state.finish())
}
