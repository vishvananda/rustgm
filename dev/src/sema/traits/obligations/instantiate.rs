//! Proof-private existentials are freshened before caller arguments enter the template.
use super::Obligations;
use crate::types::{RegionId, Store, substitute::Bound};

impl Obligations {
    pub(crate) fn instantiate_at(
        &self,
        store: &mut Store,
        parameters: &mut Bound,
        first: u32,
        depth: u16,
    ) -> Result<(Self, usize), &'static str> {
        if self.has_quantified() {
            return Err("alias direct proof already contains quantifiers");
        }
        let mut fresh = Bound::rebase_free_regions(self.regions.clone(), first)?;
        let mut result = Self::new(first);
        result.regions.end = first + (self.regions.end - self.regions.start);
        let mut region = |id| parameters.region(fresh.region(id, 0)?, depth);
        for &(a, b) in &self.equalities {
            result.equalities.push((region(a)?, region(b)?));
        }
        for &(a, b) in &self.outlives {
            result.outlives.push((region(a)?, region(b)?));
        }
        for &(ty, shorter) in &self.types {
            let ty = fresh.apply(store, ty)?;
            let ty = parameters.apply_inside(store, ty, depth)?;
            let shorter: RegionId = parameters.region(fresh.region(shorter, 0)?, depth)?;
            result.types.push((ty, shorter));
        }
        Ok((result, fresh.steps))
    }
}
