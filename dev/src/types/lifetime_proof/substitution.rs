//! Common substitution preserves each quantified block's local binder prefix.
use super::{Obligations, ScopeId};
use crate::types::{Store, substitute::Bound};
impl Obligations {
    pub(crate) fn type_roots(&self) -> impl Iterator<Item = (ScopeId, crate::types::TypeId)> + '_ {
        self.type_root_positions().map(|root| (root.scope, root.ty))
    }
    pub(crate) fn map_type_roots(
        &mut self,
        mut map: impl FnMut(ScopeId, crate::types::TypeId) -> crate::types::TypeId,
    ) {
        for (ty, _) in &mut self.types {
            *ty = map(ScopeId::ROOT, *ty);
        }
        if let Some(q) = &mut self.quantified {
            for block in &mut q.blocks {
                for (ty, _) in &mut block.proof.types {
                    *ty = map(block.scope, *ty);
                }
            }
        }
    }
    /// Substitute caller lifetimes/parameters without capturing quantified locals.
    /// # Errors
    /// Rejects malformed types, missing identities and exhausted substitution work.
    pub fn substitute(&mut self, store: &mut Store, fold: &mut Bound) -> Result<(), &'static str> {
        self.substitute_inside(store, fold, 0)
    }
    /// Substitute captured arguments while retaining an existing ambient binder prefix.
    pub(crate) fn substitute_inside(
        &mut self,
        store: &mut Store,
        fold: &mut Bound,
        ambient: u16,
    ) -> Result<(), &'static str> {
        self.substitute_at(store, fold, ambient)?;
        if let Some(q) = &mut self.quantified {
            for block in &mut q.blocks {
                let depth = if block.scope == ScopeId::ROOT {
                    0
                } else {
                    q.scopes
                        .get(block.scope.0 as usize - 1)
                        .ok_or("invalid quantified proof scope")?
                        .depth
                };
                let depth = depth
                    .checked_add(ambient)
                    .ok_or("proof scope depth exhausted")?;
                block.proof.substitute_at(store, fold, depth)?;
            }
        }
        Ok(())
    }

    /// Reserve real private identities before introducing any actual caller arguments.
    pub(crate) fn rebase(&mut self, store: &mut Store, first: u32) -> Result<usize, &'static str> {
        let old = self.template_interval()?;
        let mut fold = Bound::rebase_free_regions(old.clone(), first)?;
        self.substitute(store, &mut fold)?;
        self.regions = first..first + (old.end - old.start);
        if let Some(q) = &mut self.quantified {
            for block in &mut q.blocks {
                let interval = &mut block.proof.regions;
                if interval.start < old.start
                    || interval.end > old.end
                    || interval.end < interval.start
                {
                    return Err("invalid rebased proof interval");
                }
                *interval =
                    (first + interval.start - old.start)..(first + interval.end - old.start);
            }
        }
        Ok(fold.steps)
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
