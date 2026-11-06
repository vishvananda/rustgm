//! Immutable selected execution data; this view supplies no const or loan permission.
use super::glue;
use crate::{
    body::ir::{BlockId, Place},
    support::hash::HashMap,
    types::{TypeId, instance::Instance},
};
pub(crate) type BlockLanes = Vec<std::ops::Range<usize>>;
pub(crate) type EdgeLanes = HashMap<(BlockId, BlockId), std::ops::Range<usize>>;

#[derive(Clone, Copy, Debug)]
pub enum Effect {
    /// Ordinary inspection point, after ordered scope exits and returns.
    Point,
    /// Execute the existing body statement at this location.
    Statement,
    /// Materialized storage, including places with no owned initialization path.
    Birth(crate::body::ir::ValueId),
    /// One scope or initializer temporary, after its cleanup and before the next.
    StorageEnd(crate::body::storage::End),
    /// Ownership ends for this sparse bit interval. Borrow analysis consumes this
    /// even when no runtime conditional flag is needed; lowering emits no code.
    Deinitialize { start: u32, end: u32 },
    /// Index into `Plan::cleanups`, including storage endpoints without a call.
    Cleanup(u32),
    /// Assign the selected flag indices in this half-open range.
    Flags {
        range: glue::Range,
        initialized: bool,
    },
}
#[derive(Clone, Copy, Debug)]
pub struct Event {
    pub location: crate::body::storage::Location,
    pub effect: Effect,
}

#[derive(Clone, Copy)]
pub(crate) struct Call {
    pub cleanup: u32,
    pub place: Place,
    pub target: TypeId,
    pub instance: Instance,
    pub recipe: glue::Id,
}

pub(crate) struct View {
    pub events: Vec<Event>,
    pub recipes: glue::Recipes,
    pub calls: Vec<Call>,
    pub active: Vec<u64>,
    pub cleanup_count: u32,
    pub blocks: Vec<std::ops::Range<usize>>,
    pub edges: HashMap<(BlockId, BlockId), std::ops::Range<usize>>,
}
impl View {
    fn active(&self, cleanup: u32) -> Result<bool, &'static str> {
        if cleanup >= self.cleanup_count {
            return Err("execution cleanup ordinal exceeds its actual plan");
        }
        Ok(self
            .active
            .get(cleanup as usize / 64)
            .ok_or("execution cleanup lost its active evidence")?
            & (1 << (cleanup % 64))
            != 0)
    }
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if self.active.len() != (self.cleanup_count as usize).div_ceil(64)
            || self
                .active
                .iter()
                .map(|word| word.count_ones() as usize)
                .sum::<usize>()
                != self.calls.len()
            || self
                .calls
                .windows(2)
                .any(|pair| pair[0].cleanup >= pair[1].cleanup)
        {
            return Err("execution view changed its complete selected-call evidence");
        }
        if !self.cleanup_count.is_multiple_of(64)
            && self
                .active
                .last()
                .is_some_and(|word| word >> (self.cleanup_count % 64) != 0)
        {
            return Err("execution view has active bits outside its actual cleanup extent");
        }
        for call in &self.calls {
            if !self.active(call.cleanup)? {
                return Err("execution call has no actual active cleanup");
            }
        }
        Ok(())
    }
    pub(crate) fn call(&self, cleanup: u32) -> Result<Option<Call>, &'static str> {
        let active = self.active(cleanup)?;
        let call = self
            .calls
            .binary_search_by_key(&cleanup, |call| call.cleanup)
            .ok()
            .map(|index| self.calls[index]);
        if active != call.is_some() {
            return Err("execution cleanup changed its complete selected-call evidence");
        }
        Ok(call)
    }
    pub(crate) fn allocation_bytes(&self) -> usize {
        use crate::support::retained::{map, sum, vector};
        sum([
            vector(&self.events),
            self.recipes.allocation_bytes(),
            vector(&self.calls),
            vector(&self.active),
            vector(&self.blocks),
            map(&self.edges),
        ])
    }
}
