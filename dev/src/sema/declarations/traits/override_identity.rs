//! One canonical member may have only one override after hygiene adjustment.
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::associated::Inventory,
    support::hash::HashMap,
};

pub(super) fn overrides(
    report: &Report,
    associated: &Inventory,
    trait_item: DefId,
    members: &[DefId],
    missing: &'static str,
) -> Result<HashMap<DefId, DefId>, (DefId, &'static str)> {
    let mut result = HashMap::default();
    for &actual in members {
        let expected = associated
            .matching_member(report, trait_item, actual)
            .map_err(|error| (actual, error))?
            .ok_or((actual, missing))?;
        if result.insert(expected, actual).is_some() {
            return Err((
                actual,
                "multiple implementation members match the same trait member",
            ));
        }
    }
    Ok(result)
}
