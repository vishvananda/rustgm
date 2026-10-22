//! Alpha transport uses exact owned segments before external arguments are introduced.
use super::{Obligations, Range};
use crate::support::hash::HashMap;
use crate::types::{RegionId, Store, substitute::Bound};

pub(crate) struct OwnedTransport {
    ranges: Vec<(Range<u32>, Range<u32>)>,
    pub(crate) fold: Bound,
    pub(crate) steps: usize,
    first: u32,
    end: u32,
}
impl OwnedTransport {
    #[cfg(test)]
    pub(crate) fn new(
        owned: &[Range<u32>],
        retained: &[RegionId],
        universals: u32,
        first: u32,
    ) -> Result<Self, &'static str> {
        Self::new_budgeted(owned, retained, universals, first, 64 * 1024 * 1024)
    }
    pub(crate) fn new_budgeted(
        owned: &[Range<u32>],
        retained: &[RegionId],
        universals: u32,
        first: u32,
        bytes: usize,
    ) -> Result<Self, &'static str> {
        let mut previous = 1;
        let mut count = 0_usize;
        for interval in owned {
            if interval.start < previous
                || interval.end <= interval.start
                || interval.end >= 0x8000_0000
            {
                return Err("invalid caller-owned lifetime segments");
            }
            previous = interval.end;
            count = count.saturating_add((interval.end - interval.start) as usize);
        }
        let entries = count
            .saturating_add(retained.len())
            .saturating_add(universals as usize);
        let extra = size_of::<Self>()
            .saturating_sub(size_of::<Bound>())
            .saturating_add(
                owned
                    .len()
                    .saturating_mul(size_of::<(Range<u32>, Range<u32>)>()),
            );
        if first == 0
            || universals > 65_536
            || entries > 1_000_000
            || crate::support::retained::new_map::<RegionId, RegionId>(entries)
                .saturating_add(extra)
                .saturating_add(size_of::<Bound>())
                .saturating_add(Bound::restore_owner_overhead())
                > bytes.min(64 * 1024 * 1024)
        {
            return Err("caller lifetime transport storage budget exceeded");
        }
        let end = first
            .checked_add(u32::try_from(count).map_err(|_| "too many caller lifetimes")?)
            .filter(|&end| end < 0x8000_0000)
            .ok_or("caller lifetime transport identity space exhausted")?;
        let mut replacements = HashMap::with_capacity_and_hasher(
            entries,
            crate::support::hash::BuildFxHasher::default(),
        );
        let mut ranges = Vec::with_capacity(owned.len());
        let mut next = first;
        for interval in owned {
            let target = next..next + interval.end - interval.start;
            replacements.extend(
                interval
                    .clone()
                    .zip(target.clone())
                    .map(|(a, b)| (RegionId(a), RegionId(b))),
            );
            next = target.end;
            ranges.push((interval.clone(), target));
        }
        for &region in retained {
            let index = owned.partition_point(|interval| interval.end <= region.0);
            if region == RegionId::STATIC
                || region.bound_parts().is_some()
                || owned
                    .get(index)
                    .is_some_and(|interval| interval.contains(&region.0))
            {
                return Err("retained caller lifetime overlaps an owned identity");
            }
            replacements.insert(region, region);
        }
        for slot in 0..universals {
            let region =
                RegionId::bound(0, u16::try_from(slot).map_err(|_| "too many caller slots")?)?;
            replacements.insert(region, region);
        }
        Ok(Self {
            ranges,
            fold: Bound::restore_budgeted_regions(
                replacements,
                1_000_000,
                bytes.saturating_sub(extra),
            )?,
            steps: owned.len().saturating_add(entries),
            first,
            end,
        })
    }
    pub(crate) fn validate_fixed(&self) -> Result<(), &'static str> {
        self.fold.validate_restore_owner()
    }
    pub(crate) fn matches(&self, owned: &[Range<u32>]) -> bool {
        self.ranges.iter().map(|(source, _)| source).eq(owned)
    }
    pub(crate) fn interval_range(&self) -> Range<u32> {
        self.first..self.end
    }
    pub(crate) fn retained_bytes(&self) -> (usize, usize) {
        let (bytes, visits) = self.fold.retained_allocation();
        (
            bytes
                .saturating_add(size_of::<Self>())
                .saturating_add(crate::support::retained::vector(&self.ranges)),
            visits,
        )
    }
    pub(crate) fn tighten_bytes(&mut self, bytes: usize) -> Result<(), &'static str> {
        let extra = size_of::<Self>()
            .saturating_sub(size_of::<Bound>())
            .saturating_add(crate::support::retained::vector(&self.ranges));
        self.fold.tighten_restore_bytes(bytes.saturating_sub(extra))
    }
    /// Disjointness is necessary, but does not establish ownership of actuals.
    pub(crate) fn check_actuals(&mut self, actuals: &[RegionId]) -> Result<(), &'static str> {
        for region in actuals {
            self.steps = self.steps.saturating_add(1);
            if region.bound_parts().is_none() && region.0 >= self.first {
                return Err("actual caller lifetime overlaps the fresh transport namespace");
            }
        }
        Ok(())
    }
    fn position(&mut self, original: u32) -> u32 {
        let mut left = 0;
        let mut right = self.ranges.len();
        while left < right {
            self.steps = self.steps.saturating_add(1);
            let middle = left + (right - left) / 2;
            if self.ranges[middle].0.end <= original {
                left = middle + 1;
            } else {
                right = middle;
            }
        }
        self.ranges.get(left).map_or(self.end, |(source, target)| {
            target.start + original.saturating_sub(source.start)
        })
    }
    fn interval(&mut self, original: Range<u32>) -> Result<Range<u32>, &'static str> {
        if original.end < original.start {
            return Err("caller proof lifetime interval is reversed");
        }
        let start = self.position(original.start);
        let end = self.position(original.end);
        if end.saturating_sub(start) != original.end - original.start {
            return Err("caller proof claims a foreign lifetime gap");
        }
        Ok(start..end)
    }
    pub(crate) fn proof(
        &mut self,
        store: &mut Store,
        proof: &mut Obligations,
    ) -> Result<(), &'static str> {
        self.allocations(proof)?;
        if let Some(quantified) = &mut proof.quantified {
            for block in &mut quantified.blocks {
                if block.proof.quantified.is_some() {
                    return Err("caller proof contains an unflattened nested graph");
                }
                self.allocations(&mut block.proof)?;
            }
        }
        proof.substitute(store, &mut self.fold)?;
        Ok(())
    }
    fn allocations(&mut self, proof: &mut Obligations) -> Result<(), &'static str> {
        let original = proof.regions.clone();
        let count = proof
            .quantified
            .as_ref()
            .map_or(1, |q| q.gaps.len().saturating_add(1));
        let (retained, visits) = self.fold.retained_allocation();
        self.steps = self.steps.saturating_add(visits);
        if count > 1_000_000
            || retained.saturating_add(count.saturating_mul(size_of::<Range<u32>>()))
                > 64 * 1024 * 1024
        {
            return Err("caller proof lifetime segment budget exceeded");
        }
        let mut owned = Vec::with_capacity(count);
        for interval in proof.owned_regions() {
            owned.push(self.interval(interval)?);
        }
        proof.regions = self.position(original.start)..self.position(original.end);
        let mut next = proof.regions.start;
        if let Some(quantified) = &mut proof.quantified {
            quantified.gaps.clear();
            for interval in owned {
                if next < interval.start {
                    quantified.gaps.push(next..interval.start);
                }
                next = interval.end;
            }
            if next < proof.regions.end {
                quantified.gaps.push(next..proof.regions.end);
            }
        }
        if proof.regions.start < self.first || proof.regions.end > self.end {
            return Err("caller proof escaped its transported namespace");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/caller_namespaces.rs"]
mod tests;
