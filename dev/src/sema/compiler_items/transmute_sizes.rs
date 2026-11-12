//! Size equality uses the shared ABI readiness walk and never supplies call proof.
use super::DefId;
use crate::{
    sema::model::Declarations,
    types::{
        TypeId,
        parameters::{clauses::Clause, environment::Environment},
    },
};
const LIMIT: usize = crate::support::crates::MAX_COMPONENTS;
const CLOSED: &str = "constant storage layout requires a closed normalized type";
fn charge(work: &mut usize, count: usize) -> Result<(), &'static str> {
    *work = work.saturating_add(count);
    if *work > LIMIT {
        return Err("source transmute layout work budget exceeded");
    }
    Ok(())
}
pub(in crate::sema) fn require_equal(
    facts: &mut Declarations,
    environment: Option<&Environment>,
    source: TypeId,
    destination: TypeId,
) -> Result<(), String> {
    let mut work = 0;
    let mut paid = 0;
    let result = (|| {
        charge(&mut work, 1)?;
        pay(facts, work, &mut paid)?;
        if facts.types.kind(source).is_none() || facts.types.kind(destination).is_none() {
            return Err("source transmute has an invalid storage type".into());
        }
        for ty in [source, destination] {
            premise_free(facts, environment, ty, &mut work, &mut paid)?;
        }
        // In particular, generic identity and region-erased identity do not
        // require pretending that the parameter has a concrete byte size.
        let relation = crate::abi::storage_relation::compare(
            &mut facts.types,
            &mut facts.aggregates,
            &mut facts.layouts,
            [source, destination],
            &mut work,
            LIMIT,
        );
        pay(facts, work, &mut paid)?;
        if relation? == crate::abi::storage_relation::StorageRelation::Equal {
            return Ok(());
        }
        let source = size(facts, source, &mut work, &mut paid)?;
        let destination = size(facts, destination, &mut work, &mut paid)?;
        if source != destination {
            return Err("source transmute requires equal source and destination sizes".into());
        }
        Ok(())
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    pay(facts, work, &mut paid)?;
    result
}
/// A generic body's parameterized nominal with projected fields, as in
/// `Atomic<*mut T>`, takes its premise-free normal form before the relation or
/// readiness reads it. rustc prefers a body premise over an implementation, so
/// a premise of a trait with associated types may keep that projection rigid.
/// Such a body keeps the closed boundary, even after another body published
/// the same form.
fn premise_free(
    facts: &mut Declarations,
    environment: Option<&Environment>,
    ty: TypeId,
    work: &mut usize,
    paid: &mut usize,
) -> Result<(), String> {
    if facts.types.contains_parameters(ty) != Some(true)
        || !facts.aggregates.has_projected_templates()
    {
        return Ok(());
    }
    let normalized = crate::sema::stored_fields::prepare_premise_free(facts, ty, work);
    charge(work, 0)?;
    pay(facts, *work, paid)?;
    // A premise that keeps the projection rigid decides first, whatever the
    // premise-free attempt found.
    if selects_projections(facts, environment, work)? {
        let reads = facts
            .aggregates
            .reads_parameterized_projections(&facts.types, ty, work, LIMIT);
        pay(facts, *work, paid)?;
        if reads? {
            return Err(CLOSED.into());
        }
    }
    normalized
}
/// Whether a body premise could select a projection: an elaborated trait
/// premise whose trait declares an associated type, or any quantified premise.
/// An unreadable environment counts as one.
fn selects_projections(
    facts: &Declarations,
    environment: Option<&Environment>,
    work: &mut usize,
) -> Result<bool, &'static str> {
    let Some(environment) = environment else {
        return Ok(false);
    };
    let Ok(clauses) = environment.trait_clauses() else {
        return Ok(true);
    };
    for clause in clauses {
        charge(work, 1)?;
        let item = match clause {
            Clause::Predicate(predicate) => match predicate.bound.trait_parts() {
                Some((application, _)) => {
                    facts.types.trait_application(application).map(|a| a.item)
                }
                None => continue,
            },
            Clause::ForAll(_) => return Ok(true),
            Clause::UnstableFeature(_) => continue,
        };
        let Some(item) = item else {
            return Ok(true);
        };
        let declared = facts
            .associated
            .owner(DefId(item.0))
            .and_then(|owner| facts.associated.owner_members(owner));
        let Some(members) = declared else {
            return Ok(true);
        };
        charge(work, members.len())?;
        if members
            .iter()
            .any(|member| facts.associated.type_contract(*member).is_some())
        {
            return Ok(true);
        }
    }
    Ok(false)
}
/// Pay each completed child before another child starts. State initialization is
/// prepaid; failed work remains charged to the same sticky producer ledger.
fn pay(facts: &Declarations, work: usize, paid: &mut usize) -> Result<(), &'static str> {
    let delta = work.saturating_sub(*paid);
    *paid = work;
    crate::sema::header_attributes::charge_source_contract_query(facts, delta)
        .map_err(|_| "source transmute query work exhausted")
}
fn size(
    facts: &mut Declarations,
    ty: TypeId,
    work: &mut usize,
    paid: &mut usize,
) -> Result<u64, String> {
    charge(work, 1)?;
    pay(facts, *work, paid)?;
    if let Some(layout) = facts.layouts.get(ty) {
        return Ok(layout.size);
    }
    let before = facts.aggregates.instantiation_steps();
    let prepared = facts
        .aggregates
        .prepare_storage_fields(&mut facts.types, ty);
    charge(
        work,
        facts
            .aggregates
            .instantiation_steps()
            .saturating_sub(before),
    )?;
    pay(facts, *work, paid)?;
    prepared?;
    let ready = facts
        .layouts
        .storage_readiness(&facts.types, &facts.aggregates, ty, work, LIMIT);
    pay(facts, *work, paid)?;
    ready?;
    charge(work, facts.layouts.pending_state_work(&facts.types, ty))?;
    pay(facts, *work, paid)?;
    let before = facts.layouts.steps;
    let demanded = facts.layouts.demand_bounded(
        &facts.types,
        &facts.aggregates,
        ty,
        LIMIT.saturating_sub(*work),
    );
    charge(work, facts.layouts.steps.saturating_sub(before))?;
    pay(facts, *work, paid)?;
    demanded.map_err(|error| error.message)?;
    Ok(facts
        .layouts
        .get(ty)
        .map(|layout| layout.size)
        .ok_or("source transmute lost its prepared layout")?)
}

#[cfg(test)]
#[path = "../../../tests/invariants/bitcast_allowance.rs"]
mod tests;
