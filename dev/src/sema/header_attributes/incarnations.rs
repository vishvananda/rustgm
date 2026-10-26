//! A source image handle authenticates current syntax without copying its bytes.
use crate::{
    resolve::unit::Unit,
    support::source::OriginalImage,
    syntax::ast::{Kind, NodeId},
};
use std::sync::Arc;

#[derive(Clone)]
pub(in crate::sema) struct SourceImage {
    incarnation: Arc<OriginalImage>,
    syntax: Kind,
}
impl SourceImage {
    /// The producer prepays retained storage before taking this existing handle.
    pub(in crate::sema) fn capture(unit: &Unit, node: NodeId) -> Self {
        Self {
            incarnation: Arc::clone(unit.source.original_image()),
            syntax: unit.configured.ast.node(node).kind,
        }
    }
    pub(in crate::sema) fn current(&self, unit: &Unit, node: NodeId) -> bool {
        Arc::ptr_eq(&self.incarnation, unit.source.original_image())
            && unit
                .configured
                .ast
                .nodes
                .get(node.get() as usize)
                .is_some_and(|actual| actual.kind == self.syntax)
            && unit.configured.active.get(node.get() as usize) == Some(&true)
    }
}
