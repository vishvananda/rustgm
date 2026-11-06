//! A genuine scoped Copy selection filters destruction, never whole Destruct proof.
use super::super::{Body, Contract, Declarations, Prepared, TypeId, Work, proofs};
use crate::types::{
    GenericArgs, ItemId,
    parameters::predicates::{Bound, Predicate},
    traits::Application,
};
pub(super) fn query(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
    prepared: &Prepared,
    questions: &[TypeId],
    (work, limit): (&mut Work, usize),
) -> Result<crate::types::destruction::NeedsDrop, String> {
    let mut needed = crate::types::destruction::NeedsDrop::scoped_bounded(
        facts.destructors.implementations().iter().map(|d| d.item),
        std::sync::Arc::clone(&prepared.environment),
        limit,
    )?;
    let Some(declaration) = crate::sema::compiler_items::copy_declaration(facts)? else {
        return Ok(needed);
    };
    work.charge(questions.len())?;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(declaration.0),
        arguments: GenericArgs::EMPTY,
    })?;
    let mut goals = Vec::new();
    goals
        .try_reserve_exact(questions.len())
        .map_err(|_| "const qualifier Copy goal allocation failed")?;
    for &ty in questions {
        facts
            .types
            .kind(ty)
            .ok_or("const qualifier Copy subject is invalid")?;
        goals.push(Predicate {
            ty,
            bound: Bound::Trait(application),
        });
    }
    let first = crate::body::dispatch::copy_frontier(facts, contract)?;
    let decisions = crate::sema::compiler_items::copy_decisions(
        facts,
        Some(std::sync::Arc::clone(&prepared.environment)),
        contract.universals,
        first,
        &goals,
    )?;
    if decisions.choices.len() != questions.len()
        || decisions
            .choices
            .iter()
            .zip(questions)
            .any(|((actual, _), expected)| actual != expected)
    {
        return Err("const qualifier Copy selection changed its actual request frontier".into());
    }
    proofs::retain_proof(body, contract, prepared, decisions.proof, work)?;
    needed.seed_scoped_copy(
        &facts.types,
        &prepared.environment,
        &decisions.choices,
        limit,
    )?;
    Ok(needed)
}
