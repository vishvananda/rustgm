//! Completed pattern scopes retain cursors while values use the enclosing query graph.
use super::{
    PatternScope,
    instances::{Context, ValueScope},
};
use crate::support::hash::HashMap;
use crate::{
    resolve::unit::NodeKey,
    sema::{bodies::PreparedScope, dispatch::Dispatcher, traits::obligations::Reservation},
    support::retained::Allocation,
};

pub(super) const LIMIT: usize = 64 * 1024 * 1024;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct FrameId(pub u32);
pub(super) struct Saved {
    pub scope: Option<PatternScope>,
    pub dispatcher: Option<Dispatcher>,
    pub value: Option<ValueScope>,
    pub context: Context,
    pub bytes: usize,
}
pub(super) struct Frame {
    pub prepared: Option<PreparedScope>,
    pub inputs: Option<Box<crate::sema::callee_premises::OriginalInputs>>,
    pub source_regions: Option<Box<crate::types::substitute::Bound>>,
    pub cursor: usize,
    pub saved: Option<Saved>,
    pub reservation: Option<Reservation>,
    pub bytes: usize,
}
pub(super) enum ScopeState {
    Queued(PreparedScope),
    Active,
    Complete,
}
#[derive(Default)]
pub(super) struct Frames {
    pub batches: HashMap<NodeKey, ScopeState>,
    pub slots: Vec<Option<Frame>>,
    pub active: Vec<FrameId>,
    pub children: usize,
}
impl Frames {
    pub fn allocation(&self) -> Allocation {
        let mut total = Allocation::default();
        total.boxed(self);
        total.map(&self.batches);
        total.vector(&self.slots);
        total.vector(&self.active);
        total.bytes = total.bytes.saturating_add(self.children);
        total
    }
    pub fn frame(&mut self, id: FrameId) -> Result<&mut Frame, &'static str> {
        self.slots
            .get_mut(id.0 as usize)
            .and_then(Option::as_mut)
            .ok_or("missing constant pattern continuation frame")
    }
}

pub(super) fn vector_growth<T>(values: &Vec<T>, additional: usize) -> usize {
    let required = values.len().saturating_add(additional);
    if required <= values.capacity() {
        return 0;
    }
    required
        .saturating_mul(4)
        .saturating_sub(values.capacity())
        .saturating_mul(size_of::<T>())
}
