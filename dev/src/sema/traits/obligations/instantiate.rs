//! Proof-private existentials are freshened before caller arguments enter the template.
use super::Obligations;
use crate::types::{RegionId, Store, substitute::Bound};

impl Obligations {
    pub(crate) fn substitute(
        &mut self,
        store: &mut Store,
        fold: &mut Bound,
    ) -> Result<(), &'static str> {
        for pair in self.equalities.iter_mut().chain(&mut self.outlives) {
            pair.0 = fold.region(pair.0, 0)?;
            pair.1 = fold.region(pair.1, 0)?;
        }
        for (ty, region) in &mut self.types {
            *ty = fold.apply(store, *ty)?;
            *region = fold.region(*region, 0)?;
        }
        Ok(())
    }
    pub(crate) fn instantiate(
        &self,
        store: &mut Store,
        parameters: &mut Bound,
        first: u32,
    ) -> Result<Self, &'static str> {
        let mut fresh = Bound::rebase_free_regions(self.regions.clone(), first)?;
        let mut result = Self::new(first);
        result.regions.end = first + (self.regions.end - self.regions.start);
        let mut region = |id| parameters.region(fresh.region(id, 0)?, 0);
        for &(a, b) in &self.equalities {
            result.equalities.push((region(a)?, region(b)?));
        }
        for &(a, b) in &self.outlives {
            result.outlives.push((region(a)?, region(b)?));
        }
        for &(ty, shorter) in &self.types {
            let ty = fresh.apply(store, ty)?;
            let ty = parameters.apply(store, ty)?;
            let shorter: RegionId = parameters.region(fresh.region(shorter, 0)?, 0)?;
            result.types.push((ty, shorter));
        }
        Ok(result)
    }
}
