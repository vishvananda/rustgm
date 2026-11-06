//! Scoped value qualification retains genuine evidence before refined flow.
mod copies;
use super::{Body, Contract, Declarations, Purpose, Work, proofs};
use crate::{
    sema::compiler_items::CleanupDecision as Decision,
    types::{TypeId, arena::Identity, destruction::Qualif},
};

/// Only this authenticated source-definition query constructs answers.
/// They filter eligibility; they grant neither access nor whole Destruct proof.
pub(crate) struct Qualification<'body> {
    body: &'body Body,
    types: Identity,
    definitions: Identity,
    state: std::sync::Arc<super::State>,
    values: Vec<(TypeId, Qualif)>,
}
impl<'body> Qualification<'body> {
    fn published(
        facts: &mut Declarations,
        body: &'body Body,
        contract: &Contract,
        values: Vec<(TypeId, Qualif)>,
    ) -> Result<Self, &'static str> {
        let requirements = contract
            .builtins
            .as_deref()
            .ok_or("const qualification lost its published body policy")?;
        let state = requirements
            .defining_owner()
            .ok_or("const qualification lost its published state")?;
        state.checked(
            body,
            requirements.environment.as_ref(),
            &mut Work::new(1_000_000),
        )?;
        Ok(Self {
            body,
            types: facts.types.arena_identity(),
            definitions: facts.aggregates.arena_identity(),
            state: std::sync::Arc::clone(state),
            values,
        })
    }
    pub(crate) fn values(
        &self,
        body: &Body,
        types: &crate::types::Store,
        definitions: &crate::types::aggregate::Definitions,
        requirements: Option<&crate::body::regions::BuiltinRequirements>,
    ) -> Result<&[(TypeId, Qualif)], &'static str> {
        if !std::ptr::eq(self.body, body)
            || !types.has_identity(&self.types)
            || !definitions.has_identity(&self.definitions)
        {
            return Err("const qualification belongs to another body or arena");
        }
        let requirements = requirements
            .ok_or("const qualification lost its actual region-checked source owner")?;
        let state = requirements
            .defining()
            .ok_or("const qualification lost its defining proof")?;
        if !std::ptr::eq(state, self.state.as_ref()) {
            return Err("const qualification requires its refreshed actual defining state");
        }
        state.checked(
            body,
            requirements.environment.as_ref(),
            &mut Work::new(1_000_000),
        )?;
        Ok(&self.values)
    }
}

pub(crate) fn qualify<'body>(
    facts: &mut Declarations,
    body: &'body Body,
    contract: &mut Contract,
    questions: &[TypeId],
) -> Result<Qualification<'body>, String> {
    let bytes = questions
        .len()
        .checked_mul(
            size_of::<(TypeId, Qualif)>()
                + size_of::<(TypeId, Purpose)>()
                + size_of::<usize>()
                + size_of::<crate::types::parameters::predicates::Predicate>()
                + size_of::<(TypeId, bool)>(),
        )
        .filter(|&n| n <= 64 * 1024 * 1024)
        .ok_or("const qualification requests exceed storage budget")?;
    let mut work = Work::new(1_000_000);
    work.charge(questions.len())?;
    let mut prepared = proofs::prepare(facts, body, contract, &mut work)?;
    facts
        .source_bindings
        .as_deref()
        .ok_or("const qualification requires its actual source catalog")?
        .require_source_destructors_prepared()?;
    let environment = std::sync::Arc::clone(&prepared.environment);
    let mut needed = copies::query(
        facts,
        body,
        contract,
        &prepared,
        questions,
        (&mut work, 64 * 1024 * 1024 - bytes),
    )?;
    let mut values = Vec::new();
    let mut selected = Vec::new();
    let mut indices = Vec::new();
    values
        .try_reserve_exact(questions.len())
        .map_err(|_| "const qualification allocation failed")?;
    selected
        .try_reserve_exact(questions.len())
        .map_err(|_| "const qualification goal allocation failed")?;
    indices
        .try_reserve_exact(questions.len())
        .map_err(|_| "const qualification index allocation failed")?;
    for (index, &ty) in questions.iter().enumerate() {
        let before = needed.steps;
        let copied = contract
            .builtins
            .as_deref()
            .ok_or("const qualification lost its source body policy")?
            .copy_qualification(body, ty)?
            == Some(true);
        let drop = !copied
            && needed.demand_scoped_bounded(
                &facts.types,
                &facts.aggregates,
                ty,
                64 * 1024 * 1024 - bytes,
            )?;
        work.charge(needed.steps.saturating_sub(before))?;
        values.push((
            ty,
            if drop {
                Qualif::MayQualify
            } else {
                Qualif::NoDrop
            },
        ));
        if drop {
            selected.push((ty, Purpose::Whole));
            indices.push(index);
        }
    }
    if !selected.is_empty() {
        let first = crate::body::dispatch::copy_frontier(facts, contract)?;
        // Value qualification in a const function uses its actual conditional
        // caller law. This is not the owner's const execution effect.
        let qualifier_effect = crate::support::constness::Constness::Conditional;
        let goals = proofs::goals(facts, &selected, prepared.role, qualifier_effect, &mut work)?;
        let (decisions, proof) = crate::sema::compiler_items::decide_cleanup_requirements(
            facts,
            Some(std::sync::Arc::clone(&environment)),
            contract.universals,
            first,
            &goals,
        )?;
        let effect = prepared.effect();
        for (&decision, &index) in decisions.iter().zip(&indices) {
            if decision == Decision::Proven {
                let (ty, bad) = &mut values[index];
                *bad = Qualif::Destruct;
                if effect == qualifier_effect {
                    prepared.claims.select(
                        &mut facts.types,
                        *ty,
                        Purpose::Whole,
                        effect,
                        &mut work,
                    )?;
                }
            }
        }
        proofs::retain_proof(body, contract, &prepared, proof, &mut work)?;
    }
    proofs::publish(facts, contract, prepared, work)?;
    Ok(Qualification::published(facts, body, contract, values)?)
}

#[cfg(test)]
#[path = "../../../tests/invariants/ctqual.rs"]
mod tests;
