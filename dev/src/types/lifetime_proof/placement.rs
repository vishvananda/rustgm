//! Selected proof graphs assign reserved intervals to scopes in allocation order.
#[cfg(test)]
#[path = "../../../tests/invariants/proof_placement.rs"]
mod tests;
use super::{Obligations, ScopeId, quantified::Block};

impl Obligations {
    /// Attach flat evidence inside this proof's already reserved envelope.
    /// Allocation blocks arrive once, in order; constraint-only blocks may follow
    /// at the envelope's end. No new allocation or foreign gap is claimed here.
    pub(crate) fn place_at(&mut self, proof: Self, scope: ScopeId) -> Result<(), &'static str> {
        if scope == ScopeId::ROOT
            || proof.has_quantified()
            || proof.regions.start < self.regions.start
            || proof.regions.end > self.regions.end
            || proof.regions.end < proof.regions.start
        {
            return Err("invalid reserved scoped evidence");
        }
        self.scope_depth(scope)?;
        let q = self
            .quantified
            .as_mut()
            .ok_or("missing reserved proof scope")?;
        if q.blocks
            .last()
            .is_some_and(|previous| previous.proof.regions.end > proof.regions.start)
        {
            return Err("reserved scoped intervals overlap or move backwards");
        }
        let gap = q.gaps.partition_point(|gap| gap.end <= proof.regions.start);
        if !proof.regions.is_empty()
            && q.gaps
                .get(gap)
                .is_some_and(|gap| gap.start < proof.regions.end)
        {
            return Err("reserved scoped evidence claims a foreign interval");
        }
        q.blocks.push(Block {
            scope,
            proof,
            universal_aliases: false,
        });
        Ok(())
    }
}
