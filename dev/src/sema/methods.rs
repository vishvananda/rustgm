//! Typed inherent candidate buckets and selected qualified source uses.
mod adjusted_targets;
pub(in crate::sema) mod buckets;
mod dispatched;
pub(in crate::sema) mod names;
pub(super) mod overlap;
pub(crate) mod pointers;
pub(super) mod receiver;
pub(super) mod selection;
pub(crate) mod slices;
mod target_chain;
mod visible;
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::{model::DefId, names::NameId, unit::NodeKey},
    types::{ItemId, TypeId},
};

#[derive(Clone, Copy)]
pub enum PrefixKind {
    Nominal(ItemId),
    Annotation,
}

#[derive(Clone, Copy)]
pub struct Qualified {
    pub prefix: NodeKey,
    pub kind: PrefixKind,
    pub implementation: DefId,
}

#[derive(Default)]
pub struct Methods {
    names: Option<Box<names::Index>>,
    library: Option<Box<slices::Library>>,
    pointers: Option<Box<pointers::Library>>,
    /// Cold operand facts of comparisons and index expressions.
    pub(super) operands: Option<Box<Operands>>,
    pub(super) trait_names: HashSet<NameId>,
    /// Inherent method names whose validated source receiver wraps Self, as
    /// `self: Pin<&mut Self>` does. Only these discover owners through the
    /// legacy receiver chain. Cold: absent until such a receiver is validated.
    pub(super) smart_names: Option<Box<HashSet<NameId>>>,
    /// Published owner members; frozen in resolver order after header preparation.
    pub(super) inherent: HashMap<(ItemId, NameId), Vec<DefId>>,
    pub(super) indices: HashMap<(ItemId, NameId), Candidates>,
    pub(super) shapes: Option<Box<crate::types::substitute::Bound>>,
    pub qualified: HashMap<NodeKey, Qualified>,
    /// Caller-only evidence belongs to this selected member, not other calls at its node.
    pub(super) bound_uses: HashMap<NodeKey, DefId>,
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

/// Operand facts that only some bodies have, behind one cold pointer.
#[derive(Default)]
pub(super) struct Operands {
    /// The borrowed type of a comparison's right operand.
    right: HashMap<NodeKey, TypeId>,
    /// Exact source-original dispatch results, consumed after the body proof seals.
    dispatched: Option<Box<dispatched::Targets>>,
    /// An index expression whose operand reaches its indexed place through
    /// Deref calls keeps their plan beside its own index selection. Only a
    /// source core that declares `Deref` and `Index` has any.
    pub(super) indexed: HashMap<NodeKey, super::bodies::ProtocolPlanId>,
    /// A call expression whose callee reaches its callable target through
    /// Deref calls keeps their plan beside its own invocation selection.
    pub(super) invoked: HashMap<NodeKey, super::bodies::ProtocolPlanId>,
}
impl Methods {
    /// The plan of the Deref calls an index expression's operand takes.
    pub(crate) fn index_operand(&self, node: NodeKey) -> Option<super::bodies::ProtocolPlanId> {
        self.operands.as_deref()?.indexed.get(&node).copied()
    }
    /// The plan of the Deref calls a call expression's callee takes.
    pub(crate) fn invocation_operand(
        &self,
        node: NodeKey,
    ) -> Option<super::bodies::ProtocolPlanId> {
        self.operands.as_deref()?.invoked.get(&node).copied()
    }
    pub(super) fn retain_comparison_borrow(&mut self, node: NodeKey, ty: TypeId) {
        self.operands.get_or_insert_default().right.insert(node, ty);
    }
    pub(crate) fn comparison_borrow(&self, node: NodeKey) -> Option<TypeId> {
        self.operands.as_deref()?.right.get(&node).copied()
    }
}
