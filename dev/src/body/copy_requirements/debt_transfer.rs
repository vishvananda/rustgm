//! Typed requests leave the builtin loan list only with their complete local proof.
use super::{Arc, Body, Contract, Declarations, Kind, Predicate, Proof};
use crate::types::parameters::predicates::Bound;
#[cfg(test)]
#[path = "../../../tests/invariants/ordinary_debts.rs"]
mod tests;

pub(super) fn prepare(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
    scope: Option<&super::materialized::InstanceScope>,
) -> Result<(), &'static str> {
    let Some(pending) = contract.builtins.as_deref() else {
        return Ok(());
    };
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(pending.predicates.len());
    if pending.predicates.len() > crate::support::crates::MAX_COMPONENTS {
        return Err("body trait requirement storage budget exceeded");
    }
    if !pending.predicates.iter().any(|&(_, goal)| ordinary(goal)) {
        return Ok(());
    }
    if contract.key != body.key()
        || contract.source != body.source
        || contract.signature != body.owner.signatures().map(|(_, owned)| owned)
    {
        return Err("body trait requirements belong to another body or signature");
    }
    if let Some(instance) = body.instance {
        scope
            .ok_or("materialized trait requirements need their exact instance scope")?
            .check(facts, body, contract, pending.environment.as_ref())?;
        let valid = match (body.owner, facts.types.kind(instance.ty())) {
            (
                crate::body::ir::Owner::Function {
                    definition,
                    signature,
                    ..
                },
                Some(Kind::FunctionItem {
                    item,
                    signature: actual,
                    ..
                }),
            ) => item.0 == definition.0 && actual == signature,
            (
                crate::body::ir::Owner::Closure {
                    identity,
                    signature,
                    ..
                },
                Some(Kind::Closure {
                    identity: actual,
                    signature: actual_signature,
                    ..
                }),
            ) => identity == actual && signature == actual_signature,
            _ => false,
        };
        if !valid {
            return Err("body trait requirements belong to another callable instance");
        }
    }
    let first = crate::body::dispatch::copy_frontier(facts, contract)?;
    let proof = crate::sema::compiler_items::prove_body_requirements(
        facts,
        pending.environment.clone(),
        contract.universals,
        first,
        pending
            .predicates
            .iter()
            .filter_map(|&(_, goal)| ordinary(goal).then_some(goal)),
    )?;
    // Neither failed selection nor a successful prefix changes the pending set.
    // Keep every returned debt in the same receiving body's lifetime proof.
    contract.proofs.push(Proof::new(body.source, proof));
    let pending = contract
        .builtins
        .as_mut()
        .ok_or("body trait requirements disappeared")?;
    Arc::make_mut(pending)
        .predicates
        .retain(|&(_, goal)| !ordinary(goal));
    Ok(())
}

fn ordinary(goal: Predicate) -> bool {
    matches!(goal.bound, Bound::Trait(_))
}
