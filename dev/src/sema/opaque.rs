//! Defining contexts retain hidden types separately from published result identities.
mod captures;
mod closing;
pub(in crate::sema) mod overrides;
pub(in crate::sema) mod refinement;
mod reveal;
use crate::support::hash::HashMap;
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    types::{TypeId, opaque::OpaqueId, parameters::clauses::Clause},
};

pub struct Declaration {
    pub owner: DefId,
    pub source: NodeKey,
    pub application: TypeId,
    pub bounds: Vec<Clause>,
    pub hidden: Option<TypeId>,
    pub(crate) inferred: Option<TypeId>,
    pub(crate) free_regions: Vec<crate::types::RegionId>,
}

/// A proved override result in the impl parent and trait method parameter context.
pub(crate) struct Implementation {
    pub ty: TypeId,
    pub parent_regions: u32,
    /// Trait method source slots mapped to this template's lifetime coordinates.
    pub regions: std::sync::Arc<[crate::types::RegionId]>,
}

#[derive(Default)]
pub struct Opaques {
    pub declarations: HashMap<OpaqueId, Declaration>,
    pub owners: HashMap<DefId, Vec<OpaqueId>>,
    pub body_results: HashMap<DefId, TypeId>,
    pub(crate) implementations: HashMap<(DefId, OpaqueId), Implementation>,
    storage_cursor: usize,
    storage_layouts: Option<Box<crate::abi::templates::Templates>>,
}
