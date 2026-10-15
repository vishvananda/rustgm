//! Common substitution preserves each quantified block's local binder prefix.
use super::Obligations;
use crate::types::{Store, substitute::Bound};
impl Obligations {
    /// Substitute caller lifetimes/parameters without capturing quantified locals.
    /// # Errors
    /// Rejects malformed types, missing identities and exhausted substitution work.
    pub fn substitute(&mut self, store: &mut Store, fold: &mut Bound) -> Result<(), &'static str> {
        self.substitute_at(store, fold, 0)?;
        if let Some(q) = &mut self.quantified {
            for block in &mut q.blocks {
                let depth = q
                    .scopes
                    .get(block.scope.0 as usize - 1)
                    .ok_or("invalid quantified proof scope")?
                    .depth;
                block.proof.substitute_at(store, fold, depth)?;
            }
        }
        Ok(())
    }
    fn substitute_at(
        &mut self,
        store: &mut Store,
        fold: &mut Bound,
        depth: u16,
    ) -> Result<(), &'static str> {
        for pair in self.equalities.iter_mut().chain(&mut self.outlives) {
            pair.0 = fold.region(pair.0, depth)?;
            pair.1 = fold.region(pair.1, depth)?;
        }
        for (ty, region) in &mut self.types {
            *ty = fold.apply_inside(store, *ty, depth)?;
            *region = fold.region(*region, depth)?;
        }
        Ok(())
    }
}
