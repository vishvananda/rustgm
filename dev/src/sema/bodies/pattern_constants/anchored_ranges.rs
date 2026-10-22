//! Exact published source equalities remain obligations while aliases are transported.
use super::{Checker, PreparedScope};
use crate::{
    sema::model::Declarations,
    types::substitute::{Bound, RegionEquations},
};
use std::{collections::HashMap, num::NonZeroU32, ops::Range};
#[derive(Clone, Copy)]
pub(super) struct Ranges([NonZeroU32; 4]);
impl Ranges {
    fn new(main: Range<usize>, late: Range<usize>) -> Result<Self, &'static str> {
        let mut indices = [NonZeroU32::MIN; 4];
        for (slot, index) in indices
            .iter_mut()
            .zip([main.start, main.end, late.start, late.end])
        {
            *slot = index
                .checked_add(1)
                .and_then(|n| u32::try_from(n).ok())
                .and_then(NonZeroU32::new)
                .ok_or("pattern caller equality index space exhausted")?;
        }
        Ok(Self(indices))
    }
    fn intervals(self) -> [Range<usize>; 2] {
        [
            self.0[0].get() as usize - 1..self.0[1].get() as usize - 1,
            self.0[2].get() as usize - 1..self.0[3].get() as usize - 1,
        ]
    }
}
impl Checker<'_> {
    pub(in crate::sema::bodies) fn seal_pattern_equalities(
        &mut self,
        main: Range<usize>,
        late: Range<usize>,
    ) -> Result<(), super::Error> {
        let proof = Ranges::new(main, late).map_err(|e| self.error(self.state.body.node, e))?;
        if let Some(scope) = self
            .state
            .pending_patterns
            .as_mut()
            .and_then(|pending| pending.scopes.last_mut())
            .filter(|scope| scope.values.body == self.state.body)
        {
            if scope.proof.is_some() {
                return Err(self.error(
                    self.state.body.node,
                    "pattern caller proof was published twice",
                ));
            }
            scope.proof = Some(proof);
        }
        Ok(())
    }
}
impl PreparedScope {
    pub(in crate::sema) fn caller_publication(&self) -> Result<[Range<usize>; 2], &'static str> {
        Ok(self
            .scope
            .proof
            .ok_or("pattern caller proof is not published")?
            .intervals())
    }
    pub(in crate::sema) fn caller_region_preflight(&self) -> Result<usize, &'static str> {
        let ranges = self
            .scope
            .proof
            .ok_or("pattern caller proof is not published")?
            .intervals();
        let count = ranges.iter().try_fold(0usize, |count, range| {
            range
                .end
                .checked_sub(range.start)
                .and_then(|n| count.checked_add(n))
                .ok_or("pattern caller equality range is reversed")
        })?;
        Ok(Bound::projection_closure_preflight(
            count,
            self.scope.values.binders as usize,
        ))
    }
    pub(in crate::sema) fn caller_regions(
        &self,
        facts: &Declarations,
    ) -> Result<(Bound, usize), &'static str> {
        let mut equations = RegionEquations::default();
        let mut work = 0;
        let ranges = self
            .scope
            .proof
            .ok_or("pattern caller proof is not published")?
            .intervals();
        if ranges[0].end > ranges[1].start {
            return Err("pattern caller equality ranges overlap");
        }
        let mut slices = [&[][..]; 2];
        for (slice, range) in slices.iter_mut().zip(ranges) {
            *slice = facts
                .region_equalities
                .get(range)
                .ok_or("pattern caller equality range is stale")?;
            if slice.iter().any(|fact| fact.body != self.scope.values.body) {
                return Err("pattern caller equality range has a foreign owner");
            }
        }
        for slice in slices {
            work += slice.len();
            work += equations.extend(slice.iter().map(|fact| (fact.left, fact.right)))?;
        }
        let mut fold = Bound::close_projection_regions(1, 0, HashMap::new())?;
        fold.exchange_projection_equations(&mut equations)?;
        Ok((fold, work))
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/anchored_ranges.rs"]
mod tests;
