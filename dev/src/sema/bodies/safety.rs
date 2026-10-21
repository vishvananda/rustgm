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
    pub(super) fn mutable_static(&self, id: super::DefId) -> bool {
        self.facts
            .mutable_statics
            .as_ref()
            .is_some_and(|ids| ids.contains(&id))
    }
    pub(super) fn check_static_binding(
        &mut self,
        source: NodeId,
        pattern: crate::sema::patterns::PatternId,
        input: Option<NodeId>,
    ) -> Result<(), Error> {
        if let Some(input) = input
            && matches!(
                self.facts.patterns.nodes[pattern.0 as usize].kind,
                crate::sema::patterns::Kind::Binding {
                    capture: crate::sema::patterns::Capture::Borrow { .. },
                    ..
                }
            )
        {
            self.check_static_reference(source, input)?;
        }
        Ok(())
    }
    pub(super) fn check_static_reference(
        &self,
        source: NodeId,
        mut place: NodeId,
    ) -> Result<(), Error> {
        if self.facts.mutable_statics.is_none() {
            return Ok(());
        }
        for _ in 0..1_000_000 {
            if self.facts.autoderefs.contains_key(&self.key(place)) {
                return Ok(());
            }
            match self.kind(place) {
                Kind::Group(base)
                | Kind::Attributed { value: base, .. }
                | Kind::Field { value: base, .. }
                | Kind::Index { value: base, .. } => place = base,
                Kind::Path { .. } | Kind::Name(_) => {
                    if self.facts.node(self.key(place)).and_then(|f| f.reference)
                        .is_some_and(|target| matches!(target,
                            crate::resolve::model::Target::Definition(id) if self.mutable_static(id)))
                    {
                        return Err(self.error(source, "references to mutable statics are denied in edition 2024"));
                    }
                    return Ok(());
                }
                _ => return Ok(()),
            }
        }
        Err(self.error(source, "static reference place work budget exceeded"))
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
