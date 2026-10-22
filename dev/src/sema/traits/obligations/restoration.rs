//! Equality components supply result identities without granting the equalities as assumptions.
use super::Obligations;
use crate::types::{RegionId, substitute::Bound};
use std::collections::{HashMap, HashSet};

impl Obligations {
    /// Restore only components anchored in the caller, preserving every equation
    /// in this proof. `require` must still discharge them before publication.
    pub(crate) fn restoration(&self) -> Result<Bound, &'static str> {
        self.restore::<false>(false, &mut 0)
    }

    pub(crate) fn restoration_counted(&self, work: &mut usize) -> Result<Bound, &'static str> {
        self.restore::<true>(false, work)
    }

    /// Partial inference can determine a previously unknown input with a fresh
    /// lifetime. Keep it only when the body retains this selection's whole proof.
    pub(crate) fn body_restoration(&self) -> Result<Bound, &'static str> {
        self.restore::<false>(true, &mut 0)
    }

    pub(crate) fn body_restoration_counted(&self, work: &mut usize) -> Result<Bound, &'static str> {
        self.restore::<true>(true, work)
    }

    fn restore<const COUNTED: bool>(
        &self,
        retain_private: bool,
        work: &mut usize,
    ) -> Result<Bound, &'static str> {
        if self.equalities.len() > 1_000_000 {
            return Err("lifetime restoration work budget exceeded");
        }
        let mut edges: HashMap<RegionId, Vec<RegionId>> = HashMap::new();
        for &(left, right) in &self.equalities {
            if COUNTED {
                *work += 1;
            }
            edges.entry(left).or_default().push(right);
            edges.entry(right).or_default().push(left);
        }
        let mut seen = HashSet::new();
        let mut component = Vec::new();
        let mut pending = Vec::new();
        let mut replacements = HashMap::new();
        for &start in edges.keys() {
            if COUNTED {
                *work += 1;
            }
            if !seen.insert(start) {
                continue;
            }
            component.clear();
            pending.push(start);
            let mut anchor: Option<RegionId> = None;
            while let Some(region) = pending.pop() {
                if COUNTED {
                    *work += 1;
                }
                component.push(region);
                if !self.owns_region(region) {
                    if region.bound_parts().is_none() && region.0 >= self.regions.start {
                        return Err("lifetime restoration contains a foreign free identity");
                    }
                    if anchor.is_none_or(|previous| region.0 < previous.0) {
                        anchor = Some(region);
                    }
                }
                for &next in edges.get(&region).into_iter().flatten() {
                    if COUNTED {
                        *work += 1;
                    }
                    if seen.insert(next) {
                        pending.push(next);
                    }
                }
            }
            if let Some(anchor) = anchor {
                for &region in &component {
                    if COUNTED {
                        *work += 1;
                    }
                    replacements.insert(region, anchor);
                }
            }
        }
        if !retain_private {
            Ok(Bound::restore_regions(replacements))
        } else if self.has_gaps() {
            Bound::restore_streamed_regions(replacements, self.owned_regions().collect())
        } else {
            Ok(Bound::restore_owned_regions(
                replacements,
                self.regions.clone(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streamed_components_restore_caller_anchors_without_admitting_foreign_ids()
    -> Result<(), &'static str> {
        let mut proof = Obligations::new(10);
        proof.regions.end = 12;
        proof.resume_at(20)?;
        proof.regions.end = 22;
        proof.equalities.extend([
            (RegionId(10), RegionId(20)),
            (RegionId(20), RegionId(3)),
            (RegionId(11), RegionId(21)),
        ]);
        let mut anchored = proof.restoration()?;
        assert_eq!(anchored.region(RegionId(10), 0)?, RegionId(3));
        assert_eq!(anchored.region(RegionId(20), 0)?, RegionId(3));
        assert!(anchored.region(RegionId(21), 0).is_err());
        let mut retained = proof.body_restoration()?;
        assert_eq!(retained.region(RegionId(11), 0)?, RegionId(11));
        assert_eq!(retained.region(RegionId(21), 0)?, RegionId(21));
        for region in 12..20 {
            assert!(retained.region(RegionId(region), 0).is_err());
        }
        assert!(retained.region(RegionId(22), 0).is_err());
        Ok(())
    }

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
        proof.regions.end = 22;
        let mut restore = proof.body_restoration()?;
        assert_eq!(restore.region(RegionId(11), 0)?, RegionId(11));
        assert_eq!(restore.region(RegionId(20), 0)?, RegionId(20));
        for id in 13..20 {
            assert!(restore.region(RegionId(id), 0).is_err());
        }
        assert!(proof.template_interval().is_err());
        proof.equalities.push((RegionId(21), RegionId(15)));
        assert!(proof.body_restoration().is_err());
        assert!(proof.restoration().is_err());
        Ok(())
    }
}
