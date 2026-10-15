//! Equality components supply result identities without granting the equalities as assumptions.
use super::Obligations;
use crate::types::{RegionId, substitute::Bound};
use std::collections::{HashMap, HashSet};

impl Obligations {
    /// Restore only components anchored in the caller, preserving every equation
    /// in this proof. `require` must still discharge them before publication.
    pub(crate) fn restoration(&self) -> Result<Bound, &'static str> {
        if self.equalities.len() > 1_000_000 {
            return Err("lifetime restoration work budget exceeded");
        }
        let mut edges: HashMap<RegionId, Vec<RegionId>> = HashMap::new();
        for &(left, right) in &self.equalities {
            edges.entry(left).or_default().push(right);
            edges.entry(right).or_default().push(left);
        }
        let mut seen = HashSet::new();
        let mut component = Vec::new();
        let mut pending = Vec::new();
        let mut replacements = HashMap::new();
        for &start in edges.keys() {
            if !seen.insert(start) {
                continue;
            }
            component.clear();
            pending.push(start);
            let mut anchor: Option<RegionId> = None;
            while let Some(region) = pending.pop() {
                component.push(region);
                if !self.regions.contains(&region.0) {
                    if region.bound_parts().is_none() && region.0 >= self.regions.end {
                        return Err("lifetime restoration contains a foreign free identity");
                    }
                    if anchor.is_none_or(|previous| region.0 < previous.0) {
                        anchor = Some(region);
                    }
                }
                for &next in edges.get(&region).into_iter().flatten() {
                    if seen.insert(next) {
                        pending.push(next);
                    }
                }
            }
            if let Some(anchor) = anchor {
                for &region in &component {
                    replacements.insert(region, anchor);
                }
            }
        }
        Ok(Bound::restore_regions(replacements))
    }
}
