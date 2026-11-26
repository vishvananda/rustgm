//! Source publication reads exact item and call-signature results of original dispatch.
use super::{HashMap, Methods, NodeKey, TypeId};
use crate::support::retained;

type Key = (NodeKey, NodeKey, TypeId);
pub(super) type Targets = HashMap<Key, TypeId>;

impl Methods {
    fn retain_dispatched(
        &mut self,
        owner: NodeKey,
        source: NodeKey,
        original: TypeId,
        selected: TypeId,
        pay: retained::Payment<'_>,
    ) -> Result<(), &'static str> {
        let key = (owner, source, original);
        if let Some(previous) = self.dispatched(owner, source, original) {
            return if previous == selected {
                Ok(())
            } else {
                Err("source original has conflicting completed dispatch targets")
            };
        }
        let present = self
            .operands
            .as_deref()
            .and_then(|rows| rows.dispatched.as_deref());
        let (len, capacity) = present.map_or((0, 0), |rows| (rows.len(), rows.capacity()));
        let growth = retained::Growth::map::<Key, TypeId>(len, capacity, 1);
        pay(
            growth.work.saturating_add(1),
            growth.bytes.saturating_add(
                if present.is_none() {
                    size_of::<Targets>()
                } else {
                    0
                } + if self.operands.is_none() {
                    size_of::<super::Operands>()
                } else {
                    0
                },
            ),
        )?;
        self.operands
            .get_or_insert_default()
            .dispatched
            .get_or_insert_default()
            .insert(key, selected);
        Ok(())
    }

    pub(crate) fn dispatched(
        &self,
        owner: NodeKey,
        source: NodeKey,
        original: TypeId,
    ) -> Option<TypeId> {
        self.operands
            .as_deref()?
            .dispatched
            .as_deref()?
            .get(&(owner, source, original))
            .copied()
    }
}

impl crate::sema::model::Declarations {
    pub(crate) fn retain_source_dispatch(
        &mut self,
        owner: NodeKey,
        source: NodeKey,
        original: TypeId,
        selected: TypeId,
    ) -> Result<(), &'static str> {
        let payments = crate::sema::header_attributes::caller_payments(
            self.must_use_contracts.as_deref(),
            true,
        )
        .map_err(|_| "source dispatch retention lacks its actual budget")?;
        self.methods
            .retain_dispatched(owner, source, original, selected, &|w, b| {
                payments
                    .charge(w, b)
                    .map_err(|_| "source dispatch retention exceeds its budget")
            })
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/dispatched_targets.rs"]
mod tests;
