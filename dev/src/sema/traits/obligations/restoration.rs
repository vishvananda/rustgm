//! Equality components supply result identities without granting the equalities as assumptions.
use super::Obligations;
use crate::types::{RegionId, substitute::Bound};
use std::collections::{HashMap, HashSet};

impl Obligations {
    /// Restore only components anchored in the caller, preserving every equation
    /// in this proof. `require` must still discharge them before publication.
    pub(crate) fn restoration(&self) -> Result<Bound, &'static str> {
        self.restore(false)
    }

    /// Partial inference can determine a previously unknown input with a fresh
    /// lifetime. Keep it only when the body retains this selection's whole proof.
    pub(crate) fn body_restoration(&self) -> Result<Bound, &'static str> {
        self.restore(true)
    }

    fn restore(&self, retain_private: bool) -> Result<Bound, &'static str> {
        let owned = self.template_interval()?;
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
        Ok(if retain_private {
            Bound::restore_owned_regions(replacements, owned)
        } else {
            Bound::restore_regions(replacements)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_input_restoration_keeps_only_owned_existentials() -> Result<(), &'static str> {
        let mut proof = Obligations::new(10);
        proof.regions.end = 13;
        let caller = RegionId::bound(0, 1)?;
        proof
            .equalities
            .extend([(RegionId(10), caller), (RegionId(11), RegionId(12))]);
        let mut restore = proof.body_restoration()?;
        assert_eq!(restore.region(RegionId(10), 2)?, RegionId::bound(2, 1)?);
        assert_eq!(restore.region(RegionId(11), 0)?, RegionId(11));
        assert_eq!(restore.region(RegionId(12), 2)?, RegionId(12));
        assert_eq!(restore.region(RegionId::STATIC, 0)?, RegionId::STATIC);
        assert_eq!(restore.region(caller, 2)?, caller);
        for foreign in [
            RegionId(9),
            RegionId(13),
            RegionId(100),
            RegionId::bound(2, 0)?,
        ] {
            assert!(restore.region(foreign, 0).is_err());
        }
        assert!(proof.restoration()?.region(RegionId(11), 0).is_err());
        assert_eq!(
            proof.equalities,
            [(RegionId(10), caller), (RegionId(11), RegionId(12))]
        );
        proof.resume_at(20)?;
        assert!(proof.body_restoration().is_err());
        Ok(())
    }
}
