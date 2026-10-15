//! Typed inherent candidate buckets and selected qualified source uses.
pub(super) mod overlap;
use crate::{
    resolve::{model::DefId, names::NameId, unit::NodeKey},
    types::ItemId,
};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct Qualified {
    pub prefix: NodeKey,
    pub annotation: bool,
    pub nominal: ItemId,
    pub implementation: DefId,
}

#[derive(Default)]
pub struct Methods {
    /// Members enter each bucket in resolver declaration order.
    pub(super) inherent: HashMap<(ItemId, NameId), Vec<DefId>>,
    pub qualified: HashMap<NodeKey, Qualified>,
    /// Unique owner-pair probes, shared across names and independent of call demand.
    pub overlap_pairs: usize,
    pub overlap_steps: usize,
}
