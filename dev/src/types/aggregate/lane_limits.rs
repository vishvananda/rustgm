//! Sparse actual nominal limits are fixed before any concrete layout is prepared.
use super::{Definitions, ItemId, Kind};
use crate::support::{
    hash::HashMap,
    retained::{self, Growth},
};
pub(super) type Limits = HashMap<ItemId, u64>;
impl Definitions {
    pub(crate) fn simd_limit_growth(&self) -> Growth {
        let (length, capacity) = self
            .simd_limits
            .as_deref()
            .map_or((0, 0), |rows| (rows.len(), rows.capacity()));
        let mut growth = Growth::map::<ItemId, u64>(length, capacity, 1);
        if self.simd_limits.is_none() {
            growth.bytes = growth.bytes.saturating_add(size_of::<Limits>());
        }
        growth
    }
    pub(crate) fn simd_limit_bytes(&self) -> usize {
        self.simd_limits.as_deref().map_or(0, |rows| {
            size_of::<Limits>().saturating_add(retained::map(rows))
        })
    }
    pub(crate) fn register_simd_limit(
        &mut self,
        item: ItemId,
        limit: u64,
        prepaid: usize,
    ) -> Result<(), &'static str> {
        if self.get(item).is_none_or(|row| row.kind != Kind::Struct) {
            return Err("SIMD limit requires its completed actual struct");
        }
        if self
            .simd_limits
            .as_deref()
            .is_some_and(|rows| rows.contains_key(&item))
        {
            return Err("SIMD limit is already registered");
        }
        let rows = self.simd_limits.get_or_insert_default();
        rows.try_reserve(1)
            .map_err(|_| "SIMD limit allocation failed")?;
        if size_of::<Limits>().saturating_add(retained::map(rows)) > prepaid {
            return Err("SIMD limit allocation exceeds prepaid storage");
        }
        rows.insert(item, limit);
        Ok(())
    }
    pub(crate) fn simd_lane_limit(&self, item: ItemId) -> Option<u64> {
        self.simd_limits.as_deref()?.get(&item).copied()
    }
    pub(crate) fn require_simd_limit_transport(&self) -> Result<(), &'static str> {
        if self
            .simd_limits
            .as_deref()
            .is_some_and(|rows| !rows.is_empty())
        {
            return Err("SIMD lane limits require authenticated representation metadata transport");
        }
        Ok(())
    }
}
