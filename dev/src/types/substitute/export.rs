//! Lazy witness export preserves identities across suspensions and allocator gaps.
use super::{
    Action, Bound, RegionId, WORK_LIMIT,
    region_mapping::{Mapping, Mode},
};
use std::{collections::HashMap, ops::Range};

/// Only the semantic owner can classify a lifetime's proof scope. Universal
/// aliases are omitted; they must never become an exported caller lifetime.
#[derive(Clone, Debug)]
pub enum ExportSource {
    Caller(Range<u32>),
    Private(Range<u32>),
}
impl ExportSource {
    fn interval(&self) -> &Range<u32> {
        match self {
            Self::Caller(interval) | Self::Private(interval) => interval,
        }
    }
}
pub(super) struct Export {
    sources: Vec<ExportSource>,
    pub next: u32,
    witnesses: Vec<(RegionId, RegionId)>,
}
impl Export {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.vector(&self.sources);
        total.vector(&self.witnesses);
        total
    }

    fn resume(&mut self, sources: &[ExportSource], first: u32) -> Result<(), &'static str> {
        if first < self.next || first >= 0x8000_0000 {
            return Err("invalid witness export allocation frontier");
        }
        let mut end = self.next;
        for source in sources {
            let interval = source.interval();
            if interval.start < end || interval.end <= interval.start || interval.end > first {
                return Err("witness export sources overlap or change earlier ownership");
            }
            end = interval.end;
        }
        self.sources.extend_from_slice(sources);
        self.next = first;
        Ok(())
    }

    pub fn region(
        &mut self,
        region: RegionId,
        replacements: &mut HashMap<RegionId, RegionId>,
        steps: &mut usize,
    ) -> Result<RegionId, &'static str> {
        if region.bound_parts().is_some() {
            return Err("witness export retains an unopened surrounding lifetime");
        }
        let mut comparisons = 1;
        let index = self.sources.partition_point(|source| {
            comparisons += 1;
            source.interval().end <= region.0
        });
        *steps = steps.saturating_add(comparisons);
        if *steps > WORK_LIMIT {
            return Err("witness export work budget exhausted");
        }
        let source = self
            .sources
            .get(index)
            .filter(|source| source.interval().contains(&region.0))
            .ok_or("witness export contains a universal or unowned lifetime")?;
        if matches!(source, ExportSource::Caller(_)) {
            return Ok(region);
        }
        let next = self
            .next
            .checked_add(1)
            .filter(|end| *end < 0x8000_0000)
            .ok_or("witness export lifetime identity space exhausted")?;
        let exported = RegionId(self.next);
        self.next = next;
        replacements.insert(region, exported);
        // A later caller refresh can bring the exported identity back into the
        // isolated graph. It already belongs to the older caller scope.
        replacements.insert(exported, exported);
        self.witnesses.push((region, exported));
        Ok(exported)
    }
}

impl Bound {
    /// Export only encountered private lifetimes into fresh caller identities.
    /// Sources are disjoint intervals in allocation order. Their classification
    /// and every emitted equality remain the semantic caller's proof obligation.
    /// Reuse this fold for the same query; unchanged witnesses retain identities.
    /// # Errors
    /// Rejects invalid source intervals, frontiers and exhausted work.
    pub fn export_regions(sources: &[ExportSource], first: u32) -> Result<Self, &'static str> {
        let mut fold = Self::new(Vec::new());
        fold.account_arguments(sources.len())?;
        let mut export = Export {
            sources: Vec::new(),
            next: 1,
            witnesses: Vec::new(),
        };
        export.resume(sources, first)?;
        fold.action = Action::Regions(Box::new(Mapping {
            mode: Mode::Export(Box::new(export)),
            replacements: HashMap::new(),
        }));
        Ok(fold)
    }

    /// Register newly allocated inputs and continue at the actual global frontier.
    /// Existing sources, omitted universal aliases and exported IDs cannot be
    /// reclassified. Earlier canonical substitutions therefore remain valid.
    /// # Errors
    /// Rejects another fold mode, overlapping sources and invalid frontiers.
    pub fn resume_export(
        &mut self,
        sources: &[ExportSource],
        first: u32,
    ) -> Result<(), &'static str> {
        self.account_arguments(sources.len())?;
        let Action::Regions(mapping) = &mut self.action else {
            return Err("resuming a non-export lifetime fold");
        };
        let Mode::Export(export) = &mut mapping.mode else {
            return Err("resuming a non-export lifetime mapping");
        };
        export.resume(sources, first)
    }

    /// Each encountered private lifetime and its distinct older witness, in
    /// first-use order. Keep the equality in the private lifetime's original
    /// proof scope; allocate the witness in the caller's scope before publishing.
    #[must_use]
    pub fn exported_regions(&self) -> impl ExactSizeIterator<Item = (RegionId, RegionId)> + '_ {
        let pairs = match &self.action {
            Action::Regions(mapping) => match &mapping.mode {
                Mode::Export(export) => export.witnesses.as_slice(),
                _ => &[],
            },
            _ => &[],
        };
        pairs.iter().copied()
    }
}
