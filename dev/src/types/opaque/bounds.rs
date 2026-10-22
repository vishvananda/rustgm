//! Public lifetime promises select capture slots without inspecting hidden storage.
#[cfg(test)]
#[path = "../../../tests/invariants/promised_regions.rs"]
mod tests;
use super::OpaqueId;
use crate::support::hash::HashMap;
use crate::types::{GenericArgs, Kind, RegionId, Store, TypeId, arguments::Argument};
use std::collections::hash_map::Entry;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Promise {
    Static,
    Capture(usize),
}
#[derive(Default)]
pub(in crate::types) struct Declarations {
    promises: HashMap<OpaqueId, Box<[Promise]>>,
}
impl Store {
    pub(crate) fn opaque_live_bound(
        &self,
        id: OpaqueId,
        captures: GenericArgs,
        depth: u16,
        steps: &mut usize,
    ) -> Result<Option<RegionId>, &'static str> {
        // A nested callable's local bound is not an available outer guarantee.
        if depth != 0 {
            return Ok(None);
        }
        let mut unique = None;
        let mut multiple = false;
        for region in self.opaque_outlives(id, captures, depth) {
            *steps = steps.saturating_add(1);
            if let Some(region) = region? {
                if region == RegionId::STATIC {
                    return Ok(Some(region));
                }
                multiple |= unique.is_some_and(|prior| prior != region);
                unique = Some(region);
            }
        }
        Ok(unique.filter(|_| !multiple))
    }

    pub(crate) fn declare_opaque_outlives(
        &mut self,
        application: TypeId,
        bounds: &[RegionId],
    ) -> Result<(), &'static str> {
        if bounds.is_empty() {
            return Ok(());
        }
        let Some(Kind::Opaque(id, captures)) = self.kind(application) else {
            return Err("invalid opaque lifetime declaration");
        };
        let slots: HashMap<_, _> = self
            .arguments(captures)
            .ok_or("invalid opaque lifetime captures")?
            .enumerate()
            .filter_map(|(i, a)| {
                if let Argument::Lifetime(r) = a {
                    Some((r, i))
                } else {
                    None
                }
            })
            .collect();
        let promises: Box<[_]> = bounds
            .iter()
            .map(|r| {
                if *r == RegionId::STATIC {
                    Ok(Promise::Static)
                } else {
                    slots
                        .get(r)
                        .copied()
                        .map(Promise::Capture)
                        .ok_or("opaque promise names an excluded lifetime")
                }
            })
            .collect::<Result<_, _>>()?;
        match self
            .opaque_outlives
            .get_or_insert_with(Default::default)
            .promises
            .entry(id)
        {
            Entry::Vacant(entry) => {
                entry.insert(promises);
                Ok(())
            }
            Entry::Occupied(entry) if *entry.get() == promises => Ok(()),
            Entry::Occupied(_) => Err("conflicting opaque lifetime promises"),
        }
    }

    pub(crate) fn opaque_outlives(
        &self,
        id: OpaqueId,
        captures: GenericArgs,
        depth: u16,
    ) -> impl Iterator<Item = Result<Option<RegionId>, &'static str>> + '_ {
        self.opaque_outlives
            .as_ref()
            .and_then(|d| d.promises.get(&id))
            .into_iter()
            .flatten()
            .map(move |promise| {
                let region = match promise {
                    Promise::Static => RegionId::STATIC,
                    Promise::Capture(index) => {
                        let Some(Argument::Lifetime(region)) =
                            self.arguments(captures).and_then(|mut a| a.nth(*index))
                        else {
                            return Err("opaque lifetime argument kind mismatch");
                        };
                        region
                    }
                };
                super::super::outlives::free(region, depth)
            })
    }
}
