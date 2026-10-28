//! Lexical unsafe permissions survive suspension without granting const permissions.
use super::{Checker, Error, Kind, NodeId};
use crate::syntax::ast::BlockFlavor;
use crate::{sema::safety::Level, support::diagnostics::WarningCode};

/// Copy this context when parking a closure body; item bodies start a fresh one.
#[derive(Clone, Copy, Default)]
pub(super) struct Context {
    pub function: bool,
    blocks: u32,
}

impl Checker<'_> {
    pub(super) fn mark_union_write(&mut self, mut node: NodeId) -> Result<(), Error> {
        if !self.facts.aggregates.has_unions() {
            return Ok(());
        }
        for _ in 0..1_000_000 {
            match self.kind(node) {
                Kind::Group(base) | Kind::Attributed { value: base, .. } => node = base,
                Kind::Field { value: base, .. } => {
                    self.state.union_write_places.insert(node);
                    node = base;
                }
                _ => return Ok(()),
            }
        }
        Err(self.error(node, "union store place work budget exceeded"))
    }
    pub(super) fn check_union_access(&mut self, node: NodeId) -> Result<(), Error> {
        if self.state.union_write_places.contains(&node)
            || self.state.raw_address_places.contains(&node)
        {
            Ok(())
        } else {
            self.require_unsafe(node, "union field access requires an unsafe context")
        }
    }
    pub(super) fn check_union_receiver(
        &mut self,
        node: NodeId,
        mut place: NodeId,
    ) -> Result<(), Error> {
        if !self.facts.aggregates.has_unions()
            || !self.facts.autoderefs.contains_key(&self.key(node))
        {
            return Ok(());
        }
        for _ in 0..1_000_000 {
            match self.kind(place) {
                Kind::Group(value) | Kind::Attributed { value, .. } => place = value,
                Kind::Field { value, .. } => {
                    let union = self
                        .facts
                        .node(self.key(place))
                        .and_then(|f| f.operation)
                        .and_then(|op| {
                            if let super::Operation::Field {
                                field: crate::sema::members::Selection::Nominal(field),
                            } = op
                            {
                                Some(field)
                            } else {
                                None
                            }
                        })
                        .and_then(|field| self.facts.members.variant(field))
                        .map(|variant| self.facts.members.variants[variant.0 as usize].owner)
                        .and_then(|owner| self.facts.aggregates.get(owner))
                        .is_some_and(|d| d.kind == crate::types::aggregate::Kind::Union);
                    if union {
                        self.require_unsafe(
                            place,
                            "union field access requires an unsafe context",
                        )?;
                    }
                    place = value;
                }
                _ => return Ok(()),
            }
        }
        Err(self.error(node, "union receiver place work budget exceeded"))
    }
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
        &mut self,
        source: NodeId,
        mut place: NodeId,
    ) -> Result<(), Error> {
        let level = self.facts.safety_policies.reference(self.key(source));
        if self.facts.mutable_statics.is_none() || level == Level::Allow {
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
                Kind::Path { .. } | Kind::Name(..) => {
                    if self.facts.node(self.key(place)).and_then(|f| f.reference)
                        .is_some_and(|target| matches!(target,
                            crate::resolve::model::Target::Definition(id) if self.mutable_static(id)))
                    {
                        if matches!(level, Level::Warn | Level::Expect) {
                            return self.safety_warning(
                                source,
                                WarningCode::StaticMutRefs,
                                "creating a reference to a mutable static is discouraged",
                            );
                        }
                        return Err(self.error(source, "references to mutable statics are denied in edition 2024"));
                    }
                    return Ok(());
                }
                _ => return Ok(()),
            }
        }
        Err(self.error(source, "static reference place work budget exceeded"))
    }
    pub(super) fn check_static_access(
        &mut self,
        node: NodeId,
        id: super::DefId,
    ) -> Result<(), Error> {
        if self
            .facts
            .foreign
            .as_deref()
            .and_then(|registry| registry.global(id))
            .is_some_and(|global| {
                global.linkage == crate::support::linkage::Linkage::Import
                    && global.safety != crate::syntax::item::Safety::Safe
            })
            && !self.state.raw_address_places.contains(&node)
        {
            self.require_unsafe(node, "foreign static access requires an unsafe context")?;
        }
        if self.mutable_static(id) && !self.state.raw_address_places.contains(&node) {
            self.require_unsafe(node, "mutable static access requires an unsafe context")?;
        }
        Ok(())
    }
    pub(super) fn require_unsafe(
        &mut self,
        node: NodeId,
        operation: &'static str,
    ) -> Result<(), Error> {
        if self.state.safety.blocks != 0 {
            Ok(())
        } else if self.state.safety.function {
            match self.facts.safety_policies.operation(self.key(node)) {
                Level::Allow => Ok(()),
                Level::Warn | Level::Expect => self.safety_warning(
                    node,
                    WarningCode::UnsafeOpInUnsafeFn,
                    "unsafe operation in unsafe function requires an explicit unsafe block",
                ),
                Level::Deny | Level::Forbid => Err(self.error(
                    node,
                    "unsafe operation in unsafe function requires an explicit unsafe block",
                )),
            }
        } else {
            Err(self.error(node, operation))
        }
    }
    fn safety_warning(
        &mut self,
        node: NodeId,
        code: WarningCode,
        message: &'static str,
    ) -> Result<(), Error> {
        let key = self.key(node);
        let offset = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .node(node)
            .span
            .start;
        self.facts
            .must_use_diagnostic(self.report, key, offset, 0, code, message.to_owned())
    }
}
