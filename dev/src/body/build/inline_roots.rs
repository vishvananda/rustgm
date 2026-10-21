//! Anonymous roots retain their own typed view beside the parent's const expression.
use super::{Builder, NodeId, Owner, TypeId};
use crate::{
    sema::{inline_constants::RootFacts, model::ReferenceAdjustment},
    types::unsizing::Conversion,
};
impl Builder<'_> {
    pub(super) fn inline_root(&self, node: NodeId) -> Option<&RootFacts> {
        if !matches!(self.body.owner, Owner::AnonymousConstant) || node != self.body.source.node {
            return None;
        }
        self.facts
            .inline_constants
            .as_deref()?
            .get(self.body.source)?
            .root
            .as_deref()
    }
    pub(super) fn reference_adjustment(&self, node: NodeId) -> Option<ReferenceAdjustment> {
        self.inline_root(node).map_or_else(
            || {
                self.facts
                    .reference_adjustments
                    .get(&super::NodeKey {
                        node,
                        ..self.body.source
                    })
                    .copied()
            },
            |root| root.reference,
        )
    }
    pub(super) fn pointer_adjustment(&self, node: NodeId) -> Option<TypeId> {
        self.inline_root(node).map_or_else(
            || {
                self.facts
                    .pointer_adjustments
                    .get(&super::NodeKey {
                        node,
                        ..self.body.source
                    })
                    .copied()
            },
            |root| root.pointer,
        )
    }
    pub(super) fn unsizing_adjustment(&self, node: NodeId) -> Option<Conversion<TypeId>> {
        self.inline_root(node).map_or_else(
            || {
                self.facts
                    .unsizing_adjustments
                    .get(&super::NodeKey {
                        node,
                        ..self.body.source
                    })
                    .copied()
            },
            |root| root.unsizing,
        )
    }
}
