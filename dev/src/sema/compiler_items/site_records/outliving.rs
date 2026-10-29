//! One cold source Location demand indexes its actual declared lifetime bounds.
use crate::{
    resolve::{bindings::Declaration, model::DefId, package::Error},
    sema::model::Declarations,
    support::{hash::HashMap, retained::Growth},
    types::{ItemId, outlives::Pair},
};

pub(super) fn complete(facts: &mut Declarations, claimed: Declaration) -> Result<(), Error> {
    let root = ItemId(claimed.definition.0);
    let payments =
        crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())?;
    if facts.lifetimes.nominal_requirements.get(root).is_some() {
        return Ok(());
    }
    // This unique cold role hook pays one existing declaration-bounds walk.
    // The inference kernel itself consumes indexed inputs, never body assumptions
    // or a repeated prefix scan for each reached nominal.
    let mut explicit: HashMap<ItemId, Vec<Pair>> = HashMap::default();
    for &(item, pair) in &facts.lifetimes.nominal_bounds {
        payments.charge(1, 0).map_err(Error::unlocated)?;
        if !explicit.contains_key(&item) {
            let growth = Growth::map::<ItemId, Vec<Pair>>(explicit.len(), explicit.capacity(), 1);
            payments
                .charge(growth.work, growth.bytes)
                .map_err(Error::unlocated)?;
            explicit
                .try_reserve(1)
                .map_err(|_| Error::unlocated("source lifetime index allocation failed"))?;
        }
        let values = explicit.entry(item).or_default();
        let growth = Growth::vector::<Pair>(values.len(), values.capacity(), 1);
        payments
            .charge(growth.work, growth.bytes)
            .map_err(Error::unlocated)?;
        values
            .try_reserve(1)
            .map_err(|_| Error::unlocated("source lifetime bounds allocation failed"))?;
        values.push(pair);
    }
    let predicates = &facts.generics.predicates;
    facts
        .lifetimes
        .nominal_requirements
        .append_reached(
            &mut facts.types,
            &facts.aggregates,
            root,
            &explicit,
            |item| {
                predicates
                    .get(&DefId(item.0))
                    .into_iter()
                    .flatten()
                    .map(|predicate| predicate.value)
            },
            &|work, bytes| payments.charge(work, bytes),
        )
        .map_err(Error::unlocated)
}
