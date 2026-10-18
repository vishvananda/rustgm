//! Interleaved body queries retain one scope map without claiming foreign allocations.
use super::{Obligations, Range};

impl Obligations {
    /// Continue a body proof after other queries have reserved their identities.
    /// This records only the new gap, never copies an earlier proof or scope prefix.
    pub(crate) fn resume_at(&mut self, first: u32) -> Result<(), &'static str> {
        if first < self.regions.end || self.regions.end < self.regions.start {
            return Err("streamed proof allocation moved backwards");
        }
        if first > self.regions.end {
            self.exclude(self.regions.end..first);
            self.regions.end = first;
        }
        Ok(())
    }

    /// Membership is logarithmic in interleaved foreign intervals.
    pub(crate) fn owns_region(&self, region: crate::types::RegionId) -> bool {
        if !self.regions.contains(&region.0) {
            return false;
        }
        let Some(quantified) = &self.quantified else {
            return true;
        };
        let index = quantified.gaps.partition_point(|gap| gap.end <= region.0);
        quantified
            .gaps
            .get(index)
            .is_none_or(|gap| gap.start > region.0)
    }

    pub(crate) fn has_gaps(&self) -> bool {
        self.quantified.as_ref().is_some_and(|q| !q.gaps.is_empty())
    }

    /// Fresh declaration templates are contiguous; streamed body IDs already
    /// belong to their caller and must never enter template rebasing.
    pub(crate) fn template_interval(&self) -> Result<Range<u32>, &'static str> {
        if self.has_gaps() {
            return Err("streamed body proof is not a fresh declaration template");
        }
        Ok(self.regions.clone())
    }

    /// Actual owned identity intervals; the envelope may also contain foreign allocations.
    pub fn owned_regions(&self) -> impl Iterator<Item = Range<u32>> + '_ {
        let mut next = self.regions.start;
        self.quantified
            .iter()
            .flat_map(|q| &q.gaps)
            .cloned()
            .chain(std::iter::once(self.regions.end..self.regions.end))
            .filter_map(move |gap| {
                let owned = next..gap.start;
                next = gap.end;
                (!owned.is_empty()).then_some(owned)
            })
    }

    pub(super) fn exclude(&mut self, gap: Range<u32>) {
        self.quantified
            .get_or_insert_with(Default::default)
            .exclude(gap);
    }
}

impl super::quantified::Quantified {
    pub(super) fn exclude(&mut self, gap: Range<u32>) {
        let gaps = &mut self.gaps;
        if let Some(previous) = gaps.last_mut()
            && previous.end == gap.start
        {
            previous.end = gap.end;
        } else {
            gaps.push(gap);
        }
    }
}

#[cfg(test)]
mod interleaved_tests;
