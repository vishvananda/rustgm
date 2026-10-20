//! Scope indexes are local to one evidence object; extension remaps them once.
use super::{
    Obligations, Range,
    origins::{Origins, Sources},
};
use crate::types::{
    RegionId,
    outlives::Queries,
    relations::{Constraints, ProofScope},
};
use std::sync::Arc;

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
    pub universal_aliases: bool,
}
#[derive(Clone, Default)]
pub(super) struct Quantified {
    pub scopes: Arc<Vec<Binder>>,
    pub blocks: Vec<Block>,
    pub gaps: Vec<Range<u32>>,
    pub origins: Option<Box<Origins>>,
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
        let mut incoming = Arc::unwrap_or_clone(other.scopes);
        for scope in &mut incoming {
            if scope.parent != ScopeId::ROOT {
                scope.parent.0 += offset;
            }
        }
        for block in &mut other.blocks {
            if block.scope != ScopeId::ROOT {
                block.scope.0 += offset;
            }
        }
        if let Some(origins) = other.origins.take() {
            self.origins
                .get_or_insert_with(Default::default)
                .extend(self.blocks.len(), *origins)?;
        }
        Arc::make_mut(&mut self.scopes).extend(incoming);
        self.blocks.extend(other.blocks);
        for gap in other.gaps {
            self.exclude(gap);
        }
        Ok(())
    }
    pub fn require(
        &self,
        proof: &Obligations,
        constraints: &mut Constraints<'_>,
        queries: &mut Queries<'_>,
        sources: Sources,
    ) -> Result<(), String> {
        let mut scopes = Vec::with_capacity(self.scopes.len() + 1);
        scopes.push(ProofScope::ROOT);
        for binder in self.scopes.iter() {
            let parent = *scopes
                .get(binder.parent.0 as usize)
                .ok_or("invalid quantified proof parent")?;
            scopes.push(
                constraints
                    .universal_scope(parent, binder.count)
                    .map_err(|e| e.to_string())?,
            );
        }
        let mut owned = proof.owned_regions().peekable();
        let mut next = proof.regions.start;
        for block in &self.blocks {
            if block.proof.regions.start < next || block.proof.regions.end > proof.regions.end {
                return Err("quantified proof intervals disagree".into());
            }
            while let Some(interval) = owned.peek_mut()
                && interval.start < block.proof.regions.start
            {
                let end = interval.end.min(block.proof.regions.start);
                for id in interval.start..end {
                    constraints
                        .variable(RegionId(id))
                        .map_err(|e| e.to_string())?;
                }
                interval.start = end;
                if interval.start == interval.end {
                    owned.next();
                }
            }
            let scope = *scopes
                .get(block.scope.0 as usize)
                .ok_or("invalid quantified proof scope")?;
            if !block.proof.regions.is_empty() {
                let interval = owned
                    .peek_mut()
                    .ok_or("unowned quantified proof interval")?;
                if interval.start != block.proof.regions.start
                    || interval.end < block.proof.regions.end
                {
                    return Err("unowned quantified proof interval".into());
                }
                interval.start = block.proof.regions.end;
                if interval.start == interval.end {
                    owned.next();
                }
            }
            for (index, id) in block.proof.regions.clone().enumerate() {
                if block.universal_aliases {
                    let index = u16::try_from(index).map_err(|_| "too many universal aliases")?;
                    constraints.scoped_alias(RegionId(id), scope, index)
                } else {
                    constraints.scoped_variable(RegionId(id), scope)
                }
                .map_err(|e| e.to_string())?;
            }
            next = block.proof.regions.end;
        }
        for id in owned.flatten() {
            constraints
                .variable(RegionId(id))
                .map_err(|e| e.to_string())?;
        }
        for (index, block) in self.blocks.iter().enumerate() {
            block.proof.require_at(
                constraints,
                queries,
                scopes[block.scope.0 as usize],
                sources.resolve(self.origin(index))?,
            )?;
        }
        Ok(())
    }
}
impl Obligations {
    /// Borrow retained evidence blocks in proof order. Scope identities belong
    /// to this object; the first block is its ordinary root evidence.
    pub(crate) fn blocks(&self) -> impl Iterator<Item = (ScopeId, bool, u32, &Self)> {
        std::iter::once((ScopeId::ROOT, false, 0, self)).chain(self.quantified.iter().flat_map(
            |q| {
                q.blocks.iter().enumerate().map(|(index, block)| {
                    (
                        block.scope,
                        block.universal_aliases,
                        q.origin(index),
                        &block.proof,
                    )
                })
            },
        ))
    }

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
        Arc::make_mut(&mut q.scopes).push(Binder {
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
        self.append_at_origin(proof, scope, 0)
    }
    pub(super) fn append_at_origin(
        &mut self,
        proof: Self,
        scope: ScopeId,
        origin: u32,
    ) -> Result<(), &'static str> {
        self.append_block(proof, scope, origin, false)
    }
    /// The opening interval names this binder's universal slots in order.
    /// It allocates no independent existential and introduces no equality debt.
    pub(crate) fn append_universals(
        &mut self,
        regions: Range<u32>,
        scope: ScopeId,
    ) -> Result<(), &'static str> {
        let mut proof = Self::new(regions.start);
        proof.regions = regions;
        self.append_block(proof, scope, 0, true)
    }
    pub(super) fn append_block(
        &mut self,
        proof: Self,
        scope: ScopeId,
        origin: u32,
        universal_aliases: bool,
    ) -> Result<(), &'static str> {
        if scope == ScopeId::ROOT && origin == 0 && !universal_aliases {
            return self.extend(proof);
        }
        if proof.has_quantified()
            || proof.has_gaps()
            || proof.regions.start != self.regions.end
            || proof.regions.end < proof.regions.start
        {
            return Err("invalid scoped proof evidence");
        }
        self.scope_depth(scope)?;
        let q = self.quantified.get_or_insert_with(Default::default);
        if universal_aliases {
            let binder = scope
                .0
                .checked_sub(1)
                .and_then(|id| q.scopes.get(id as usize))
                .ok_or("universal aliases require their own binder")?;
            if proof.regions.end - proof.regions.start != binder.count
                || !proof.equalities.is_empty()
                || !proof.outlives.is_empty()
                || !proof.types.is_empty()
            {
                return Err("universal aliases disagree with their binder");
            }
        }
        self.regions.end = proof.regions.end;
        let start = q.blocks.len();
        q.blocks.push(Block {
            scope,
            proof,
            universal_aliases,
        });
        q.label(start, origin)
    }
}
