//! Initialized source cleanup requests retain normal scoped const Destruct evidence.
use crate::{
    sema::{model::Declarations, nominals::traits::State, traits::obligations::Obligations},
    support::{bindings::LangItem, constness::Constness},
    types::parameters::{environment::Environment, predicates::Predicate},
};
use std::sync::Arc;
pub(crate) fn prove_cleanup_requirements(
    facts: &mut Declarations,
    environment: Option<Arc<Environment>>,
    universals: u32,
    first: u32,
    goals: &[Predicate],
) -> Result<Obligations, &'static str> {
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("source cleanup lost its catalog")?
        .validated_language(LangItem::Destruct)?
        .ok_or("source cleanup requires actual Destruct")?;
    let mut state = State::body(facts, environment, universals, first)?;
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(state.selection_steps());
    for goal in goals {
        let Some((application, Constness::Const | Constness::Conditional)) =
            goal.bound.trait_parts()
        else {
            return Err("source cleanup requires its actual const effect");
        };
        let application = facts
            .types
            .trait_application(application)
            .ok_or("source cleanup application is missing")?;
        if application.item.0 != role.definition.0
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|a| a.len() != 0)
        {
            return Err("source cleanup requires its actual empty Destruct application");
        }
        let before = state.selection_steps();
        let result = state.prove(facts, *goal, false);
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(state.selection_steps().saturating_sub(before));
        if !result? {
            return Err(
                "active cleanup requires const Destruct in its actual defining environment",
            );
        }
    }
    Ok(state.finish())
}
