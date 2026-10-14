//! Typed destruction recipes. A recipe is not a completed borrow proof or executable token.
pub(crate) mod dependencies;
mod flags;
pub use crate::body::drops::glue;
pub(crate) mod schedule;
mod select;
pub mod symbolic;
pub(crate) mod timeline;
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

/// Proved plans in body order, with their immutable shared glue recipes.
/// Construction is restricted to the combined ownership/lifetime/loan boundary.
pub struct Proof {
    pub(super) plans: Vec<Option<Plan>>,
    pub(super) recipes: glue::Recipes,
}
impl Proof {
    #[must_use]
    pub fn plan(&self, body: usize) -> Option<&Plan> {
        self.plans.get(body).and_then(Option::as_ref)
    }
    #[must_use]
    pub const fn recipes(&self) -> &glue::Recipes {
        &self.recipes
    }
}
