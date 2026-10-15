//! Attach a complete proof below an ambient binder without changing private ownership.
use super::{Obligations, ScopeId, map_scopes};
#[cfg(test)]
mod ownership_tests;

impl Obligations {
    pub(crate) fn graft(
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
            |parent, count| self.scope(parent, count),
        )?;
        let mut next = proof.regions.start;
        for block in quantified.blocks {
            if block.proof.regions.start < next || block.proof.regions.end > proof.regions.end {
                return Err("invalid grafted block interval");
            }
            if next < block.proof.regions.start {
                let mut gap = Self::new(next);
                gap.regions.end = block.proof.regions.start;
                self.append_at(gap, root)?;
            }
            next = block.proof.regions.end;
            let scope = *bindings
                .get(block.scope.0 as usize)
                .ok_or("invalid grafted scope")?;
            self.append_at(block.proof, scope)?;
        }
        // Root constraints retain their ambient universe; only the remaining gap
        // is allocated here. Child intervals were allocated in their own scopes.
        proof.regions.start = next;
        self.append_at(proof, root)?;
        Ok(bindings)
    }
}
