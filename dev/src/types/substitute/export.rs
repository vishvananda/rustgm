//! Lazy witness export preserves identities across suspensions and allocator gaps.
use super::{
    Action, Bound, RegionId, WORK_LIMIT,
    region_mapping::{Mapping, Mode},
};
use crate::support::hash::HashMap;
use std::ops::Range;

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
enum Sources {
    Intervals(Vec<ExportSource>),
    Scoped(HashMap<RegionId, bool>),
}
pub(super) struct Export {
    sources: Sources,
    pub next: u32,
    witnesses: Vec<(RegionId, RegionId)>,
}
impl Export {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        match &self.sources {
            Sources::Intervals(sources) => total.vector(sources),
            Sources::Scoped(sources) => total.map(sources),
        }
        total.vector(&self.witnesses);
        total
    }

    fn resume(&mut self, sources: &[ExportSource], first: u32) -> Result<(), &'static str> {
        if first < self.next || first >= 0x8000_0000 {
            return Err("invalid witness export allocation frontier");
        }
        let Sources::Intervals(retained) = &mut self.sources else {
            return Err("cannot append intervals to a scoped export");
        };
        let mut end = self.next;
        for source in sources {
            let interval = source.interval();
            if interval.start < end || interval.end <= interval.start || interval.end > first {
                return Err("witness export sources overlap or change earlier ownership");
            }
            end = interval.end;
        }
        retained.extend_from_slice(sources);
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
            return if matches!(self.sources, Sources::Scoped(_)) {
                Ok(region)
            } else {
                Err("witness export retains an unopened surrounding lifetime")
            };
        }
        let private = match &self.sources {
            Sources::Scoped(regions) => {
                *steps = steps.saturating_add(1);
                regions.get(&region).copied()
            }
            Sources::Intervals(sources) => {
                let mut comparisons = 1;
                let index = sources.partition_point(|source| {
                    comparisons += 1;
                    source.interval().end <= region.0
                });
                *steps = steps.saturating_add(comparisons);
                sources
                    .get(index)
                    .filter(|source| source.interval().contains(&region.0))
                    .map(|source| matches!(source, ExportSource::Private(_)))
            }
        }
        .ok_or("witness export contains a universal or unowned lifetime")?;
        if *steps > WORK_LIMIT {
            return Err("witness export work budget exhausted");
        }
        if !private {
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
            sources: Sources::Intervals(Vec::new()),
            next: 1,
            witnesses: Vec::new(),
        };
        export.resume(sources, first)?;
        fold.action = Action::Regions(Box::new(Mapping {
            mode: Mode::Export(Box::new(export)),
            replacements: HashMap::default(),
        }));
        Ok(fold)
    }

    /// A nested proof can refer to retained ancestors and still-bound caller names.
    pub(crate) fn scoped_export_regions() -> Self {
        let mut fold = Self::new(Vec::new());
        fold.action = Action::Regions(Box::new(Mapping {
            mode: Mode::Export(Box::new(Export {
                sources: Sources::Scoped(HashMap::default()),
                next: 1,
                witnesses: Vec::new(),
            })),
            replacements: HashMap::default(),
        }));
        fold
    }

    /// Authorize the exact free regions observed on refined inputs for this publication.
    /// Preserve previous exports across partial proof publication and resumption.
    pub(crate) fn prepare_scoped_export(
        &mut self,
        sources: &[(RegionId, bool)],
        first: u32,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        self.account_arguments(sources.len())?;
        let Action::Regions(mapping) = &mut self.action else {
            return Err("missing scoped lifetime export fold");
        };
        let Mode::Export(export) = &mut mapping.mode else {
            return Err("missing scoped lifetime export mapping");
        };
        let Sources::Scoped(regions) = &mut export.sources else {
            return Err("scoped lifetime export has interval sources");
        };
        if first < export.next || first >= 0x8000_0000 {
            return Err("invalid scoped witness export allocation frontier");
        }
        let growth = crate::support::retained::Growth::map::<RegionId, bool>(
            0,
            regions.capacity(),
            sources.len(),
        );
        charge(growth.work.saturating_add(sources.len()), growth.bytes)?;
        regions.clear();
        regions
            .try_reserve(sources.len())
            .map_err(|_| "scoped witness observations exceed allocation limits")?;
        regions.extend(sources.iter().copied());
        export.next = first;
        Ok(())
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
