//! Simultaneous lifetime replacement preserves surrounding and nested binder identities.
use super::{Action, Bound, RegionId};
use std::collections::HashMap;

enum Mode {
    Open {
        first: u32,
        next: u32,
    },
    Rebase {
        source: std::ops::Range<u32>,
        first: u32,
    },
    Restore {
        retained: Retained,
    },
}
enum Retained {
    Interval(std::ops::Range<u32>),
    Stream(Vec<std::ops::Range<u32>>),
}
impl Retained {
    fn contains(&self, region: u32) -> bool {
        match self {
            Self::Interval(interval) => interval.contains(&region),
            Self::Stream(intervals) => {
                let index = intervals.partition_point(|interval| interval.end <= region);
                intervals
                    .get(index)
                    .is_some_and(|interval| interval.start <= region)
            }
        }
    }
}
pub(super) struct Mapping {
    mode: Mode,
    replacements: HashMap<RegionId, RegionId>,
}
impl Mapping {
    pub(super) fn region(
        &mut self,
        original: RegionId,
        depth: u16,
    ) -> Result<RegionId, &'static str> {
        if original == RegionId::STATIC {
            return Ok(original);
        }
        if let Mode::Rebase { source, first } = &self.mode {
            return if original.bound_parts().is_some() {
                Ok(original)
            } else if source.contains(&original.0) {
                Ok(RegionId(first + (original.0 - source.start)))
            } else {
                Err("proof template contains a foreign free lifetime")
            };
        }
        let key = match original.bound_parts() {
            Some((binder, _)) if binder < depth => return Ok(original),
            Some((binder, slot)) => RegionId::bound(binder - depth, slot)?,
            None if matches!(self.mode, Mode::Open { .. }) => {
                let Mode::Open { first, .. } = self.mode else {
                    return Err("invalid opening lifetime mapping");
                };
                if original.0 >= first {
                    return Err("existing free lifetime overlaps the fresh region interval");
                }
                self.replacements.insert(original, original);
                return Ok(original);
            }
            None => original,
        };
        let replacement = if let Some(&replacement) = self.replacements.get(&key) {
            replacement
        } else if let Mode::Open { next, .. } = &mut self.mode {
            let replacement = RegionId(*next);
            *next = next
                .checked_add(1)
                .filter(|end| *end < 0x8000_0000)
                .ok_or("opened lifetime identity space exhausted")?;
            self.replacements.insert(key, replacement);
            replacement
        } else if let Mode::Restore { retained } = &self.mode
            && key.bound_parts().is_none()
            && retained.contains(key.0)
        {
            key
        } else {
            return Err("selected type retains an unmapped lifetime");
        };
        match replacement.bound_parts() {
            Some((binder, slot)) => RegionId::bound(
                binder
                    .checked_add(depth)
                    .ok_or("lifetime binder depth overflow")?,
                slot,
            ),
            None => Ok(replacement),
        }
    }
}
impl Bound {
    /// Freshen an owned proof interval without changing any bound lifetime.
    /// Apply this before substituting arguments, whose free lifetimes belong to
    /// the caller and must not be mistaken for the template's private variables.
    /// # Errors
    /// Rejects invalid intervals and target identity overflow.
    pub fn rebase_free_regions(
        source: std::ops::Range<u32>,
        first: u32,
    ) -> Result<Self, &'static str> {
        if source.start == 0 || source.end < source.start || source.end >= 0x8000_0000 || first == 0
        {
            return Err("invalid proof lifetime interval");
        }
        first
            .checked_add(source.end - source.start)
            .filter(|end| *end < 0x8000_0000)
            .ok_or("proof lifetime interval exhausted")?;
        let mut fold = Self::new(Vec::new());
        fold.action = Action::Regions(Box::new(Mapping {
            mode: Mode::Rebase { source, first },
            replacements: HashMap::new(),
        }));
        Ok(fold)
    }
    /// Open surrounding bound lifetimes into a fresh, caller-owned free interval.
    /// Static, existing free lifetimes and nested local binders remain unchanged.
    /// Reuse this fold for all roots and assumptions in the same proof scope.
    ///
    /// # Errors
    /// Rejects a zero or bound starting identity. The caller must reserve existing
    /// free identities below `first`; folding reports allocation exhaustion.
    pub fn open_regions(first: u32) -> Result<Self, &'static str> {
        if first == 0 || first >= 0x8000_0000 {
            return Err("opening lifetimes requires a free region allocator");
        }
        let mut fold = Self::new(Vec::new());
        fold.action = Action::Regions(Box::new(Mapping {
            mode: Mode::Open { first, next: first },
            replacements: HashMap::new(),
        }));
        Ok(fold)
    }

    /// Restore selected free lifetimes using a complete, immutable mapping.
    /// This only substitutes identities; the caller must separately prove every
    /// equality used to construct the mapping. Unknown lifetimes are errors.
    #[must_use]
    pub fn restore_regions(replacements: HashMap<RegionId, RegionId>) -> Self {
        Self::restore_owned_regions(replacements, 0..0)
    }

    /// A body retaining the whole selection proof may keep its fresh existentials.
    /// All other unmapped identities remain errors, including later allocations.
    pub(crate) fn restore_owned_regions(
        replacements: HashMap<RegionId, RegionId>,
        retained: std::ops::Range<u32>,
    ) -> Self {
        let mut fold = Self::new(Vec::new());
        fold.action = Action::Regions(Box::new(Mapping {
            mode: Mode::Restore {
                retained: Retained::Interval(retained),
            },
            replacements,
        }));
        fold
    }

    /// Keep only validated owned segments; foreign gaps are never retained.
    pub(crate) fn restore_streamed_regions(
        replacements: HashMap<RegionId, RegionId>,
        retained: Vec<std::ops::Range<u32>>,
    ) -> Result<Self, &'static str> {
        if retained.len() > 1_000_000 {
            return Err("retained lifetime interval budget exceeded");
        }
        let mut end = 1;
        for interval in &retained {
            if interval.start < end || interval.end <= interval.start || interval.end >= 0x8000_0000
            {
                return Err("invalid retained lifetime intervals");
            }
            end = interval.end;
        }
        let mut fold = Self::new(Vec::new());
        fold.action = Action::Regions(Box::new(Mapping {
            mode: Mode::Restore {
                retained: Retained::Stream(retained),
            },
            replacements,
        }));
        Ok(fold)
    }

    /// The allocated interval ends here; subsequent selection must start here.
    #[must_use]
    pub fn next_region(&self) -> Option<u32> {
        match &self.action {
            Action::Regions(mapping) => match mapping.mode {
                Mode::Open { next, .. } => Some(next),
                Mode::Restore { .. } | Mode::Rebase { .. } => None,
            },
            _ => None,
        }
    }

    /// Each original surrounding lifetime and its proof-local free alias.
    /// Existing free lifetimes appear as identity pairs.
    /// Iteration order is unspecified and must not determine semantic results.
    pub fn opened_regions(&self) -> impl Iterator<Item = (RegionId, RegionId)> + '_ {
        let replacements = match &self.action {
            Action::Regions(mapping) if matches!(mapping.mode, Mode::Open { .. }) => {
                Some(&mapping.replacements)
            }
            _ => None,
        };
        replacements.into_iter().flatten().map(|(&a, &b)| (a, b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_intervals_reject_invalid_or_overlapping_free_identity_ranges() {
        for (start, end) in [(0, 2), (2, 2), (4, 3), (1, 0x8000_0000)] {
            let intervals = std::iter::once(start..end).collect();
            assert!(Bound::restore_streamed_regions(HashMap::new(), intervals).is_err());
        }
        for intervals in [vec![1..4, 3..6], vec![5..7, 1..3]] {
            assert!(Bound::restore_streamed_regions(HashMap::new(), intervals).is_err());
        }
    }

    #[test]
    fn retained_interval_boundaries_exclude_every_gap() -> Result<(), &'static str> {
        let intervals = (1..4096).step_by(3).map(|start| start..start + 2).collect();
        let mut fold = Bound::restore_streamed_regions(HashMap::new(), intervals)?;
        for id in 1..4096 {
            if id % 3 == 0 {
                assert!(fold.region(RegionId(id), 0).is_err());
            } else {
                assert_eq!(fold.region(RegionId(id), 0)?, RegionId(id));
            }
        }
        assert!(fold.region(RegionId(4096), 0).is_err());
        assert_eq!(fold.region(RegionId::STATIC, 0)?, RegionId::STATIC);
        Ok(())
    }
}
