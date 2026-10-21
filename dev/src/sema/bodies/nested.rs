//! Nested body states are parked in one flat scheduler with body-specific payloads.
use super::{NodeId, State};
use crate::types::infer::refinement::Refinement;

/// Parked states never own their scheduler, so source nesting cannot create a
/// recursive ownership tree. The active state owns this one shared frontier.
pub(super) struct Scheduler<P> {
    pub entries: Vec<Entry<P>>,
    pub stack: Vec<(Option<usize>, Box<State>)>,
    pub active: Option<usize>,
}

/// Only the isolated interface is exchanged with the parent inference context.
/// Body-specific control, lifetimes and publication remain in the payload owner.
pub(super) struct Entry<P> {
    pub source: NodeId,
    pub inputs: Refinement,
    pub state: Option<Box<State>>,
    pub payload: P,
}

impl<P> Default for Scheduler<P> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            stack: Vec::new(),
            active: None,
        }
    }
}
