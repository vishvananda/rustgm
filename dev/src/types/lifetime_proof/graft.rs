//! Attach a complete proof below an ambient binder without changing private ownership.
use super::{Obligations, Range, ScopeId, map_scopes};
use std::iter::Peekable;
#[cfg(test)]
mod ownership_tests;

impl Obligations {
    pub(crate) fn graft(
        &mut self,
        proof: Self,
        root: ScopeId,
    ) -> Result<Vec<ScopeId>, &'static str> {
        proof.template_interval()?;
        self.graft_owned(proof, root)
    }

    /// Graft evidence whose envelope may interleave foreign allocations. Each
    /// foreign gap stays excluded here; every owned interval keeps its scope.
    pub(super) fn graft_owned(
        &mut self,
        mut proof: Self,
        root: ScopeId,
    ) -> Result<Vec<ScopeId>, &'static str> {
        self.scope_depth(root)?;
        if proof.regions.start != self.regions.end || proof.regions.end < proof.regions.start {
            return Err("invalid grafted proof interval");
        }
        let Some(quantified) = proof.quantified.take() else {
            self.append_at(proof, root)?;
            return Ok(vec![root]);
        };
        let mut bindings = Vec::new();
        let mut kinds = quantified.scopes.iter();
        let last =
            ScopeId(u32::try_from(quantified.scopes.len()).map_err(|_| "too many grafted scopes")?);
        map_scopes(
            &mut bindings,
            root,
            last,
            |id| {
                let binder =
                    id.0.checked_sub(1)
                        .and_then(|id| quantified.scopes.get(id as usize))
                        .ok_or("invalid grafted scope")?;
                Ok((binder.parent, binder.count))
            },
            |parent, count| {
                if kinds
                    .next()
                    .ok_or("missing grafted binder kind")?
                    .callable_depth
                    != 0
                {
                    self.callable_scope(parent, count)
                } else {
                    self.scope(parent, count)
                }
            },
        )?;
        let mut gaps = quantified.gaps.iter().cloned().peekable();
        let mut next = proof.regions.start;
        for (index, block) in quantified.blocks.into_iter().enumerate() {
            if block.proof.regions.start < next || block.proof.regions.end > proof.regions.end {
                return Err("invalid grafted block interval");
            }
            next = self.graft_gaps(next, block.proof.regions.start, &mut gaps, root)?;
            if next < block.proof.regions.start {
                let mut gap = Self::new(next);
                gap.regions.end = block.proof.regions.start;
                self.append_at(gap, root)?;
            }
            next = block.proof.regions.end;
            let scope = *bindings
                .get(block.scope.0 as usize)
                .ok_or("invalid grafted scope")?;
            let origin = quantified
                .origins
                .as_ref()
                .map_or(0, |origins| origins.get(index));
            self.append_block(block.proof, scope, origin, block.universal_aliases)?;
        }
        next = self.graft_gaps(next, proof.regions.end, &mut gaps, root)?;
        if gaps.next().is_some() {
            return Err("grafted foreign interval lies outside its proof envelope");
        }
        // Root constraints retain their ambient universe; only the remaining gap
        // is allocated here. Child intervals were allocated in their own scopes.
        proof.regions.start = next;
        self.append_at(proof, root)?;
        Ok(bindings)
    }

    /// Attach the owned prefix before each foreign gap below `end`, then exclude
    /// that gap. Returns the start of the trailing owned interval.
    fn graft_gaps(
        &mut self,
        mut next: u32,
        end: u32,
        gaps: &mut Peekable<impl Iterator<Item = Range<u32>>>,
        root: ScopeId,
    ) -> Result<u32, &'static str> {
        while let Some(gap) = gaps.next_if(|gap| gap.start < end) {
            if gap.start < next || gap.end > end {
                return Err("grafted foreign interval crosses an owned proof block");
            }
            if next < gap.start {
                let mut owned = Self::new(next);
                owned.regions.end = gap.start;
                self.append_at(owned, root)?;
            }
            self.resume_at(gap.end)?;
            next = gap.end;
        }
        Ok(next)
    }
}
