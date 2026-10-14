//! Typed destruction recipes. A recipe is not a completed borrow proof or executable token.
mod flags;
pub use crate::body::drops::glue;
mod select;
use crate::{body::ir::Selection, borrow::ownership::initialization::Candidate, types::TypeId};
pub use select::select;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Flag(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    Always,
    Initialized(Flag),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeId(pub u32);
#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Empty,
    Glue(glue::Id),
    Fields(glue::Range),
    Array(glue::Range),
    Variants(glue::Range),
}
pub struct Node {
    pub condition: Condition,
    pub operation: Operation,
}
pub struct Field {
    pub selection: Selection,
    pub node: NodeId,
}
pub struct Interval {
    pub start: u64,
    pub end: u64,
    /// One condition for the entire range, outside the element loop.
    pub condition: Condition,
    pub element: NodeId,
}
pub struct Variant {
    pub selection: crate::body::ir::VariantId,
    pub node: NodeId,
}
pub struct Cleanup {
    pub candidate: Candidate,
    pub node: Option<NodeId>,
    /// Original type for drop checking after field moves, even when no runtime
    /// field calls remain. A later query must select only Drop-relevant regions.
    pub live_type: Option<TypeId>,
}
#[derive(Clone, Copy, Debug)]
pub enum Effect {
    /// Ordinary inspection point, after ordered scope exits and returns.
    Point,
    /// Execute the existing body statement at this location.
    Statement,
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
/// Selected runtime operations and distinct destructor-liveness obligations.
/// Flags initially hold false and change in event order. Identities are sparse
/// initialization bits; loan proof and lowering are still separate obligations.
/// This is not an executable token.
#[derive(Default)]
pub struct Plan {
    pub cleanups: Vec<Cleanup>,
    pub nodes: Vec<Node>,
    pub fields: Vec<Field>,
    pub intervals: Vec<Interval>,
    pub variants: Vec<Variant>,
    pub flags: Vec<Flag>,
    pub events: Vec<Event>,
    pub steps: usize,
}
