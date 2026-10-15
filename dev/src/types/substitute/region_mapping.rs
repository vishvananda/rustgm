//! Simultaneous lifetime replacement preserves surrounding and nested binder identities.
use super::{Action, Bound, RegionId};
use std::collections::HashMap;

enum Mode {
    Open { first: u32, next: u32 },
    Restore,
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
        let mut fold = Self::new(Vec::new());
        fold.action = Action::Regions(Box::new(Mapping {
            mode: Mode::Restore,
            replacements,
        }));
        fold
    }

    /// The allocated interval ends here; subsequent selection must start here.
    #[must_use]
    pub fn next_region(&self) -> Option<u32> {
        match &self.action {
            Action::Regions(mapping) => match mapping.mode {
                Mode::Open { next, .. } => Some(next),
                Mode::Restore => None,
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
