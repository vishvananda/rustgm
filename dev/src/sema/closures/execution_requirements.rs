//! Checked source preconditions remain debts for every selected caller.
use super::{Bodies, ClosureId, NodeKey, RegionId};
use crate::types::outlives::Pair;
use std::{collections::HashMap, sync::Arc};
type Requirements = Option<Arc<[Pair]>>;
#[derive(Default)]
pub(super) struct Catalog {
    values: HashMap<ClosureId, Requirements>,
    payload: usize,
}
impl Catalog {
    const LIMIT: usize = 64_000_000;
    fn retain(
        &mut self,
        identity: ClosureId,
        conditions: Option<&Arc<[Pair]>>,
    ) -> Result<(), &'static str> {
        if let Some(existing) = self.values.get(&identity) {
            return if existing.as_deref() == conditions.map(AsRef::as_ref) {
                Ok(())
            } else {
                Err("checked closure execution conditions changed")
            };
        }
        let payload = self.payload.saturating_add(conditions.map_or(0, |pairs| {
            size_of_val(pairs.as_ref()).saturating_add(2 * size_of::<usize>())
        }));
        let entries = crate::support::retained::new_map::<ClosureId, Requirements>(
            self.values.len().saturating_add(1),
        );
        if payload.saturating_add(entries) > Self::LIMIT {
            return Err("closure execution conditions exceed retained storage budget");
        }
        self.values.insert(identity, conditions.cloned());
        self.payload = payload;
        Ok(())
    }
}
impl Bodies {
    pub(crate) fn retain_conditions(
        &mut self,
        identity: ClosureId,
        source: NodeKey,
        conditions: Option<&Arc<[Pair]>>,
    ) -> Result<(), &'static str> {
        let body = self
            .identity(identity)
            .ok_or("missing conditional closure source")?;
        if body.source != source || body.execution.is_none() {
            return Err("execution conditions have a different source contract");
        }
        for &(a, b) in conditions.map_or(&[][..], |pairs| pairs.as_ref()) {
            for region in [a, b] {
                if region != RegionId::STATIC
                    && !region.bound_parts().is_some_and(|(depth, slot)| {
                        depth == 0 && u32::from(slot) < body.universals
                    })
                {
                    return Err("execution condition escapes its source binder");
                }
            }
        }
        self.conditions
            .get_or_insert_with(Default::default)
            .retain(identity, conditions)
    }
    pub(crate) fn conditions(&self, identity: ClosureId) -> Option<&[Pair]> {
        Some(
            self.conditions
                .as_ref()?
                .values
                .get(&identity)?
                .as_deref()
                .unwrap_or(&[]),
        )
    }
}
#[cfg(test)]
#[path = "../../../tests/invariants/requirement_catalog.rs"]
mod tests;
