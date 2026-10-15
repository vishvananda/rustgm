//! Scope indexes are local to one evidence object; extension remaps them once.
use super::{Obligations, Range};
use crate::types::{
    RegionId,
    outlives::Queries,
    relations::{Constraints, ProofScope},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScopeId(pub(crate) u32);
impl ScopeId {
    pub const ROOT: Self = Self(0);
}
#[derive(Clone)]
pub(super) struct Binder {
    pub parent: ScopeId,
    pub count: u32,
    pub depth: u16,
}
#[derive(Clone)]
pub(super) struct Block {
    pub scope: ScopeId,
    pub proof: Obligations,
}
#[derive(Clone, Default)]
pub(super) struct Quantified {
    pub scopes: Vec<Binder>,
    pub blocks: Vec<Block>,
}
impl Quantified {
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
    pub fn extend(&mut self, mut other: Self) -> Result<(), &'static str> {
        let offset = u32::try_from(self.scopes.len()).map_err(|_| "too many proof scopes")?;
        if self.scopes.len().saturating_add(other.scopes.len()) > 1_000_000 {
            return Err("quantified proof scope budget exhausted");
        }
        for scope in &mut other.scopes {
            if scope.parent != ScopeId::ROOT {
                scope.parent.0 += offset;
            }
        }
        for block in &mut other.blocks {
            block.scope.0 += offset;
        }
        self.scopes.extend(other.scopes);
        self.blocks.extend(other.blocks);
        Ok(())
    }
    pub fn require(
        &self,
        regions: Range<u32>,
        constraints: &mut Constraints<'_>,
        queries: &mut Queries<'_>,
        source: u32,
    ) -> Result<(), String> {
        let mut scopes = Vec::with_capacity(self.scopes.len() + 1);
        scopes.push(ProofScope::ROOT);
        for binder in &self.scopes {
            let parent = *scopes
                .get(binder.parent.0 as usize)
                .ok_or("invalid quantified proof parent")?;
            scopes.push(
                constraints
                    .universal_scope(parent, binder.count)
                    .map_err(|e| e.to_string())?,
            );
        }
        let mut next = regions.start;
        for block in &self.blocks {
            if block.proof.regions.start < next || block.proof.regions.end > regions.end {
                return Err("quantified proof intervals disagree".into());
            }
            for id in next..block.proof.regions.start {
                constraints
                    .variable(RegionId(id))
                    .map_err(|e| e.to_string())?;
            }
            let scope = *scopes
                .get(block.scope.0 as usize)
                .ok_or("invalid quantified proof scope")?;
            for id in block.proof.regions.clone() {
                constraints
                    .scoped_variable(RegionId(id), scope)
                    .map_err(|e| e.to_string())?;
            }
            next = block.proof.regions.end;
        }
        for id in next..regions.end {
            constraints
                .variable(RegionId(id))
                .map_err(|e| e.to_string())?;
        }
        for block in &self.blocks {
            block
                .proof
                .require_at(constraints, queries, scopes[block.scope.0 as usize], source)?;
        }
        Ok(())
    }
}
impl Obligations {
    pub(crate) fn scope_depth(&self, scope: ScopeId) -> Result<u16, &'static str> {
        if scope == ScopeId::ROOT {
            return Ok(0);
        }
        self.quantified
            .as_ref()
            .and_then(|quantified| quantified.scopes.get(scope.0 as usize - 1))
            .map(|binder| binder.depth)
            .ok_or("invalid quantified proof scope")
    }
    /// Introduce a universal binder relative to this evidence's caller scope.
    /// # Errors
    /// Rejects missing parents and exhausted scope/binder depth budgets.
    pub fn scope(&mut self, parent: ScopeId, count: u32) -> Result<ScopeId, &'static str> {
        let q = self.quantified.get_or_insert_with(Default::default);
        let depth = if parent == ScopeId::ROOT {
            0
        } else {
            q.scopes
                .get(parent.0 as usize - 1)
                .ok_or("invalid proof scope parent")?
                .depth
        };
        let depth = depth
            .checked_add(1)
            .filter(|d| *d < 0x8000)
            .ok_or("proof scope nesting exhausted")?;
        if q.scopes.len() >= 1_000_000 {
            return Err("quantified proof scope budget exhausted");
        }
        q.scopes.push(Binder {
            parent,
            count,
            depth,
        });
        Ok(ScopeId(
            u32::try_from(q.scopes.len()).map_err(|_| "too many proof scopes")?,
        ))
    }
    /// Attach flat, fresh evidence to its actual quantifier scope.
    /// # Errors
    /// Rejects foreign scopes, nested objects and noncontiguous intervals.
    pub fn append_at(&mut self, proof: Self, scope: ScopeId) -> Result<(), &'static str> {
        if scope == ScopeId::ROOT {
            return self.extend(proof);
        }
        if proof.has_quantified()
            || proof.regions.start != self.regions.end
            || proof.regions.end < proof.regions.start
        {
            return Err("invalid scoped proof evidence");
        }
        let q = self
            .quantified
            .as_mut()
            .ok_or("missing quantified proof scope")?;
        q.scopes
            .get(scope.0 as usize - 1)
            .ok_or("invalid quantified proof scope")?;
        self.regions.end = proof.regions.end;
        q.blocks.push(Block { scope, proof });
        Ok(())
    }
}
