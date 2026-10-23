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

pub(crate) fn copy_decisions(
    facts: &mut Declarations,
    environment: Option<Arc<Environment>>,
    universals: u32,
    first: u32,
    goals: &[Predicate],
) -> Result<Decisions, &'static str> {
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("source Copy has no source policy")?
        .validated_language(LangItem::Copy)?
        .ok_or("source Copy language item is missing")?;
    let mut state = State::body(facts, environment, universals, first)?;
    let mut decisions = Vec::with_capacity(goals.len());
    for &goal in goals {
        if !matches!(goal.bound, crate::types::parameters::predicates::Bound::Trait(id)
            if facts.types.trait_application(id).is_some_and(|a| a.item.0 == declaration.definition.0
                && facts.types.arguments(a.arguments).is_some_and(|args| args.len() == 0)))
        {
            return Err("source Copy requirement has a different declaration or arguments");
        }
        decisions.push((goal.ty, state.copy_decision(facts, goal)?));
    }
    Ok(Decisions {
        choices: decisions,
        proof: state.finish(),
    })
}
