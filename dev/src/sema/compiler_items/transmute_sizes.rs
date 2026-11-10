//! Size equality uses the shared ABI readiness walk and never supplies call proof.
use crate::{sema::model::Declarations, types::TypeId};
const LIMIT: usize = crate::support::crates::MAX_COMPONENTS;
fn charge(work: &mut usize, count: usize) -> Result<(), &'static str> {
    *work = work.saturating_add(count);
    if *work > LIMIT {
        return Err("source transmute layout work budget exceeded");
    }
    Ok(())
}
pub(in crate::sema) fn require_equal(
    facts: &mut Declarations,
    source: TypeId,
    destination: TypeId,
) -> Result<(), &'static str> {
    let mut work = 0;
    let mut paid = 0;
    let result = (|| {
        charge(&mut work, 1)?;
        pay(facts, work, &mut paid)?;
        if facts.types.kind(source).is_none() || facts.types.kind(destination).is_none() {
            return Err("source transmute has an invalid storage type");
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
            return Err("source transmute requires equal source and destination sizes");
        }
        Ok(())
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    pay(facts, work, &mut paid)?;
    result
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
) -> Result<u64, &'static str> {
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
    facts
        .layouts
        .get(ty)
        .map(|layout| layout.size)
        .ok_or("source transmute lost its prepared layout")
}

#[cfg(test)]
#[path = "../../../tests/invariants/bitcast_allowance.rs"]
mod tests;
