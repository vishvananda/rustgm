//! Compiler-created body aliases retain ordered declaration slots, not proof equalities.
use crate::{
    support::{hash::HashMap, retained::Growth},
    types::RegionId,
};

pub(crate) struct BodyAliases {
    ordered: Vec<RegionId>,
    slots: HashMap<RegionId, u16>,
}
impl BodyAliases {
    pub(crate) fn new(ordered: Vec<RegionId>) -> Result<Self, &'static str> {
        let mut slots = HashMap::default();
        for (slot, &alias) in ordered.iter().enumerate() {
            let slot = u16::try_from(slot).map_err(|_| "too many body alias slots")?;
            if alias == RegionId::STATIC
                || alias.bound_parts().is_some()
                || slots.insert(alias, slot).is_some()
            {
                return Err("invalid or repeated body universal alias");
            }
        }
        Ok(Self { ordered, slots })
    }

    /// Cost only: selected clone, validation and geometric table coexistence.
    pub(crate) fn construction_bound(count: usize) -> Result<Growth, &'static str> {
        if count == 0 {
            return Ok(Growth::default());
        }
        // The producer indexes elements, so 65536 slots are representable.
        u16::try_from(count - 1).map_err(|_| "too many body alias slots")?;
        let values = Growth::vector::<RegionId>(0, 0, count);
        let map = Growth::map::<RegionId, u16>(0, 0, count);
        let control = count.saturating_mul(4).saturating_mul(2).saturating_add(32);
        Ok(Growth {
            bytes: values
                .bytes
                .saturating_add(map.bytes.saturating_mul(2))
                .saturating_add(control.saturating_mul(2))
                .saturating_add(size_of::<Self>() + 2 * size_of::<usize>()),
            work: map
                .work
                .saturating_mul(4)
                .saturating_add(count.saturating_mul(4))
                .saturating_add(16),
        })
    }
    /// Conservative control/alignment allowance beyond logical entry slots.
    pub(crate) fn control_allocation(&self) -> usize {
        let capacity = self.slots.capacity();
        if capacity == 0 {
            0
        } else {
            capacity.saturating_mul(2).saturating_add(32)
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.ordered.len()
    }

    pub(crate) fn slot(&self, alias: RegionId) -> Option<u16> {
        self.slots.get(&alias).copied()
    }

    pub(crate) fn ordered(&self) -> &[RegionId] {
        &self.ordered
    }

    pub(crate) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.vector(&self.ordered);
        total.map(&self.slots);
        total
    }
}
