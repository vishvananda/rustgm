//! Declared effects are readiness data, never cached constant-call permission.
use super::Catalog;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    support::hash::HashMap,
    types::parameters::{clauses::Clause, predicates::Bound},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sema) enum Effect {
    Ordinary,
    Conditional,
}
impl Catalog {
    /// Completed declaration data, never per-use const eligibility or payment.
    pub(in crate::sema) fn prepared_call_effect(
        &self,
        definition: DefId,
    ) -> Result<Option<Effect>, &'static str> {
        self.require_source_destructors_prepared()?;
        Ok(self
            .call_effects
            .as_deref()
            .and_then(|effects| effects.get(&definition))
            .copied())
    }
    pub(crate) fn check_const_call(&self, definition: DefId) -> Result<(), &'static str> {
        self.require_source_destructors_prepared()?;
        match self
            .call_effects
            .as_deref()
            .and_then(|m| m.get(&definition))
        {
            Some(Effect::Ordinary) => Ok(()),
            Some(Effect::Conditional) => {
                Err("source conditional const-call debt payment is not implemented")
            }
            None => Err("source const-call effects require completed declaration preparation"),
        }
    }
}

pub(crate) fn check_instance(
    facts: &Declarations,
    instance: crate::types::instance::Instance,
) -> Result<(), &'static str> {
    // The reached selector adapter requests F, never an executable selector
    // body. Function-root/unadapted demand must still reject its placeholder.
    super::select_calls::check_const_select(facts, instance)?;
    if let Some(crate::types::Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) {
        facts.check_const_member_execution(DefId(item.0))?;
    }
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    let Some(crate::types::Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty())
    else {
        return Err("missing selected source const function");
    };
    catalog.check_const_call(DefId(item.0))
}

/// The owning declaration calls this once, after freezing its full environment
/// and completing its actual signature. Inherited clauses remain part of it.
pub(in crate::sema) fn record(
    facts: &mut Declarations,
    definition: DefId,
) -> Result<(), &'static str> {
    if facts.source_bindings.is_none() || !facts.constant_functions.contains(&definition) {
        return Ok(());
    }
    let effect = declared_effect(facts, definition)?;
    let catalog = facts
        .source_bindings
        .as_deref_mut()
        .ok_or("source call policy disappeared")?;
    let effects = catalog
        .call_effects
        .get_or_insert_with(|| Box::new(HashMap::default()));
    if effects.contains_key(&definition) {
        return Err("source const-call effects were published twice");
    }
    if effects.len() >= 1_000_000 {
        return Err("source const-call effect inventory exhausted");
    }
    effects.insert(definition, effect);
    Ok(())
}

fn declared_effect(facts: &mut Declarations, definition: DefId) -> Result<Effect, &'static str> {
    let has_scope = facts
        .generics
        .parameters_in_scope(definition)
        .next()
        .is_some()
        || facts
            .generics
            .predicates_in_scope(definition)
            .next()
            .is_some();
    if has_scope && !facts.generics.environments.contains_key(&definition) {
        return Err("source const-call effects require a frozen declaration environment");
    }
    let mut effect = Effect::Ordinary;
    for clause in facts.generics.declared_predicates_in_scope(definition) {
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .checked_add(1)
            .ok_or("source const-call effect work exhausted")?;
        if conditional_clause(&facts.types, clause.value)? {
            effect = Effect::Conditional;
        }
    }
    Ok(effect)
}

pub(super) fn conditional_clause(
    types: &crate::types::Store,
    clause: Clause,
) -> Result<bool, &'static str> {
    let predicate = match clause {
        Clause::UnstableFeature(_) => return Ok(false),
        Clause::Predicate(value) => value,
        Clause::ForAll(id) => {
            types
                .quantified(id)
                .ok_or("source const-call clause is malformed")?
                .predicate
        }
    };
    Ok(matches!(predicate.bound, Bound::MaybeConstTrait(_)))
}

#[cfg(test)]
#[path = "../../../tests/invariants/reached_effects.rs"]
mod tests;

#[cfg(test)]
pub(super) fn declared_effect_for_test(
    facts: &mut Declarations,
    definition: DefId,
) -> Result<Effect, &'static str> {
    declared_effect(facts, definition)
}
