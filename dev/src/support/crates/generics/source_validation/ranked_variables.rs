//! Dense inverse order is unpaid data; it allocates no canonical region identity.
use super::Work;
use crate::support::crates::{MAX_BYTES, generics::source::BodyUnit};

/// One immutable raw unit's zero-based allocation ordinals. This is not a
/// catalog/arena/body binding, defining environment or semantic proof domain.
pub struct AllocationOrdinals<'rows> {
    unit: &'rows BodyUnit,
    ordinals: Vec<u32>,
}

pub(super) fn build<'rows>(
    unit: &'rows BodyUnit,
    existing_peak: usize,
    extra_bytes: usize,
    work: &mut Work,
) -> Result<AllocationOrdinals<'rows>, &'static str> {
    // Only NumberingUnits calls this producer after its complete permutation
    // and alias relation checks. Reuse that immutable seal rather than running
    // a second independently maintained structural validation algorithm.
    let count = unit.allocation_order.len();
    work.charge(count.saturating_add(1))?;
    let growth = AllocationOrdinals::allocation_bound(count);
    if existing_peak
        .saturating_add(extra_bytes)
        .saturating_add(growth)
        > MAX_BYTES
    {
        return Err("source allocation ordinals exceed combined storage budget");
    }
    let mut ordinals = vec![0; count];
    for (ordinal, &slot) in unit.allocation_order.iter().enumerate() {
        let target = ordinals
            .get_mut(slot as usize)
            .ok_or("checked source allocation slot disappeared")?;
        *target = u32::try_from(ordinal).map_err(|_| "source allocation ordinal overflow")?;
    }
    Ok(AllocationOrdinals { unit, ordinals })
}
impl AllocationOrdinals<'_> {
    pub(crate) const fn allocation_bound(count: usize) -> usize {
        size_of::<Self>().saturating_add(count.saturating_mul(size_of::<u32>()))
    }

    /// # Errors
    /// Rejects an unknown raw slot and charges unsuccessful queries too.
    pub fn ordinal(&self, slot: u32, work: &mut Work) -> Result<u32, &'static str> {
        work.charge(1)?;
        self.ordinals
            .get(slot as usize)
            .copied()
            .ok_or("source allocation ordinal has an unknown raw variable slot")
    }
    /// Return the declared universal's allocation ordinal, or retain the raw
    /// unpaid absence claim. Actual required alias presence is checked later by
    /// the authenticated defining environment producer, never by this helper.
    /// # Errors
    /// Rejects an unknown universal, including queries against absent ledgers.
    pub fn alias(&self, universal: u32, work: &mut Work) -> Result<Option<u32>, &'static str> {
        work.charge(1)?;
        if universal >= self.unit.universals {
            return Err("source alias ordinal has an unknown declaration universal");
        }
        let Some(aliases) = &self.unit.aliases else {
            return Ok(None);
        };
        let slot = aliases
            .get(universal as usize)
            .ok_or("checked source universal alias disappeared")?;
        self.ordinal(*slot, work).map(Some)
    }
    #[must_use]
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(crate::support::retained::vector(&self.ordinals))
    }
}
