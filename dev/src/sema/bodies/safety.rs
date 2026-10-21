//! Lexical unsafe permissions survive suspension without granting const permissions.
use super::{Checker, Error, Kind, NodeId};
use crate::syntax::ast::BlockFlavor;

/// Copy this context when parking a closure body; item bodies start a fresh one.
#[derive(Clone, Copy, Default)]
pub(super) struct Context {
    pub function: bool,
    blocks: u32,
}

impl Checker<'_> {
    pub(super) fn enter_safety(&mut self, node: NodeId) -> Result<(), Error> {
        if matches!(
            self.kind(node),
            Kind::Block {
                flavor: BlockFlavor::Unsafe,
                ..
            }
        ) {
            self.state.safety.blocks = self
                .state
                .safety
                .blocks
                .checked_add(1)
                .ok_or_else(|| self.error(node, "unsafe block nesting budget exceeded"))?;
        }
        Ok(())
    }
    pub(super) fn leave_safety(&mut self, node: NodeId) -> Result<(), Error> {
        if matches!(
            self.kind(node),
            Kind::Block {
                flavor: BlockFlavor::Unsafe,
                ..
            }
        ) {
            self.state.safety.blocks = self
                .state
                .safety
                .blocks
                .checked_sub(1)
                .ok_or_else(|| self.error(node, "unbalanced unsafe block context"))?;
        }
        Ok(())
    }
    fn mutable_static(&self, id: super::DefId) -> bool {
        self.report.resolution.definitions[id.0 as usize].node.is_some_and(|key| {
            matches!(super::Declarations::kind(self.report, key), Kind::Item(item)
                if matches!(self.report.package.unit(key.file).configured.ast.items[item as usize].kind,
                    crate::syntax::item::ItemKind::Static { mutable: true, .. }))
        })
    }
    pub(super) fn check_static_access(&self, node: NodeId, id: super::DefId) -> Result<(), Error> {
        if self.mutable_static(id) && !self.state.raw_address_places.contains(&node) {
            self.require_unsafe(node, "mutable static access requires an unsafe context")?;
        }
        Ok(())
    }
    pub(super) fn require_unsafe(
        &self,
        node: NodeId,
        operation: &'static str,
    ) -> Result<(), Error> {
        if self.state.safety.function || self.state.safety.blocks != 0 {
            Ok(())
        } else {
            Err(self.error(node, operation))
        }
    }
}
