//! Optional diagnostic origins accompany shared proof blocks without copying prefixes.
use super::{Obligations, quantified::Quantified};
#[cfg(test)]
mod source_tests;

/// An append position in the proof that produced it, not cross-proof authority.
#[derive(Clone, Copy)]
pub struct SourceCheckpoint {
    blocks: usize,
    equalities: usize,
    outlives: usize,
    types: usize,
}

#[derive(Clone, Default)]
pub(super) struct Origins {
    labels: Vec<u32>,
    maximum: u32,
}
impl Origins {
    pub(super) fn get(&self, block: usize) -> u32 {
        self.labels.get(block).copied().unwrap_or(0)
    }
    fn label(&mut self, start: usize, end: usize, source: u32) -> Result<(), &'static str> {
        if start < self.labels.len() || start > end {
            return Err("proof sources would overwrite an earlier origin");
        }
        if end > 1_000_000 {
            return Err("proof source storage budget exhausted");
        }
        self.labels.resize(start, 0);
        self.labels.resize(end, source);
        self.maximum = self.maximum.max(source);
        Ok(())
    }
    pub(super) fn extend(&mut self, start: usize, other: Self) -> Result<(), &'static str> {
        if start < self.labels.len() || start.saturating_add(other.labels.len()) > 1_000_000 {
            return Err("invalid or exhausted proof source extension");
        }
        self.labels.resize(start, 0);
        self.labels.extend(other.labels);
        self.maximum = self.maximum.max(other.maximum);
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(super) struct Sources {
    pub first: u32,
    count: u32,
}
impl Sources {
    pub(super) fn new(first: u32, count: u32) -> Result<Self, &'static str> {
        if count == 0 || first.checked_add(count - 1).is_none() {
            return Err("invalid lifetime proof source range");
        }
        Ok(Self { first, count })
    }
    pub(super) fn resolve(self, label: u32) -> Result<u32, &'static str> {
        if label >= self.count {
            return Err("lifetime proof source is outside its source range");
        }
        Ok(self.first + label)
    }
    pub(super) fn validate(self, proof: &Obligations) -> Result<(), &'static str> {
        if let Some(origins) = proof.quantified.as_ref().and_then(|q| q.origins.as_ref()) {
            self.resolve(origins.maximum)?;
        }
        Ok(())
    }
}

impl Obligations {
    /// Capture append boundaries before producing a query's evidence.
    #[must_use]
    pub fn source_checkpoint(&self) -> SourceCheckpoint {
        SourceCheckpoint {
            blocks: self.quantified.as_ref().map_or(0, |q| q.blocks.len()),
            equalities: self.equalities.len(),
            outlives: self.outlives.len(),
            types: self.types.len(),
        }
    }

    /// Label newly appended evidence blocks with a caller-owned source slot.
    /// Slot zero denotes the fallback source and requires no origin allocation.
    /// The checkpoint belongs to this proof. Previously labelled blocks cannot
    /// be relabelled; source slots never grant lifetime or allocation authority.
    /// # Errors
    /// Rejects stale boundaries, overlapping labels and exhausted source storage.
    pub fn label_sources_since(
        &mut self,
        checkpoint: SourceCheckpoint,
        source: u32,
    ) -> Result<(), &'static str> {
        let end = self.source_checkpoint();
        if checkpoint.blocks > end.blocks
            || checkpoint.equalities > end.equalities
            || checkpoint.outlives > end.outlives
            || checkpoint.types > end.types
        {
            return Err("proof source checkpoint is outside its proof");
        }
        if source == 0 {
            return Ok(());
        }
        let root = checkpoint.equalities < end.equalities
            || checkpoint.outlives < end.outlives
            || checkpoint.types < end.types;
        if checkpoint.blocks == end.blocks && !root {
            return Ok(());
        }
        let q = self.quantified.get_or_insert_with(Default::default);
        let labels = q.origins.get_or_insert_with(Default::default);
        labels.label(checkpoint.blocks, end.blocks + usize::from(root), source)?;
        if root {
            // Move only new root constraints, once. Their variables were already
            // allocated at the root; this empty interval claims no new identities.
            let mut proof = Self::new(self.regions.end);
            proof.equalities = self.equalities.split_off(checkpoint.equalities);
            proof.outlives = self.outlives.split_off(checkpoint.outlives);
            proof.types = self.types.split_off(checkpoint.types);
            q.blocks.push(super::quantified::Block {
                scope: super::ScopeId::ROOT,
                proof,
            });
        }
        Ok(())
    }
}
impl Quantified {
    pub(super) fn origin(&self, block: usize) -> u32 {
        self.origins.as_ref().map_or(0, |labels| labels.get(block))
    }
    pub(super) fn label(&mut self, start: usize, source: u32) -> Result<(), &'static str> {
        if source == 0 {
            return Ok(());
        }
        self.origins
            .get_or_insert_with(Default::default)
            .label(start, self.blocks.len(), source)
    }
}
