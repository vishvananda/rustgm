//! Typed inherent candidate buckets and selected qualified source uses.
pub(super) mod overlap;
pub(super) mod receiver;
pub(super) mod selection;
pub(crate) mod slices;
mod visible;
use crate::{
    resolve::{model::DefId, names::NameId, unit::NodeKey},
    types::{ItemId, TypeId},
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
pub struct Qualified {
    pub prefix: NodeKey,
    pub annotation: bool,
    pub nominal: ItemId,
    pub implementation: DefId,
}

#[derive(Default)]
pub struct Methods {
    library: Option<Box<slices::Library>>,
    visible: Option<Box<visible::Index>>,
    comparison_borrows: Option<Box<ComparisonBorrows>>,
    pub(super) trait_names: HashSet<NameId>,
    /// Published owner members; frozen in resolver order after header preparation.
    pub(super) inherent: HashMap<(ItemId, NameId), Vec<DefId>>,
    pub(super) indices: HashMap<(ItemId, NameId), Candidates>,
    pub(super) shapes: Option<Box<crate::types::substitute::Bound>>,
    pub qualified: HashMap<NodeKey, Qualified>,
    /// Parameter-bound lookup restricts the selected trait application to caller evidence.
    pub(super) bound_uses: HashSet<NodeKey>,
    pub(super) receivers: HashMap<DefId, (TypeId, u32)>,
    pub calls: HashMap<NodeKey, Call>,
    /// Unique owner-pair probes, shared across names and independent of call demand.
    pub overlap_pairs: usize,
    pub overlap_steps: usize,
    pub selection_probes: usize,
    pub selection_steps: usize,
    pub selection_shape_steps: usize,
    pub trait_visibility_steps: usize,
}

#[derive(Default)]
pub(super) struct Candidates {
    pub closed: HashMap<TypeId, Vec<DefId>>,
    pub open: Vec<DefId>,
}

/// One selected method callable and the receiver place conversion consumed by lowering.
#[derive(Clone, Copy)]
pub struct Call {
    pub method: DefId,
    pub callable: TypeId,
    pub dereferences: u32,
    pub borrow: Option<TypeId>,
    pub receiver: TypeId,
}

#[derive(Default)]
struct ComparisonBorrows {
    right: HashMap<NodeKey, TypeId>,
}
impl Methods {
    pub(super) fn retain_comparison_borrow(&mut self, node: NodeKey, ty: TypeId) {
        self.comparison_borrows
            .get_or_insert_default()
            .right
            .insert(node, ty);
    }
    pub(crate) fn comparison_borrow(&self, node: NodeKey) -> Option<TypeId> {
        self.comparison_borrows
            .as_deref()?
            .right
            .get(&node)
            .copied()
    }
}
