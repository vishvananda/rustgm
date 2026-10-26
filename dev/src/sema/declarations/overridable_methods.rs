//! Nonoverlapping defaults keep the same real body and normal conformance checks.
use super::{Checker, DefId, Error, NodeKey};
use crate::{sema::associated::OwnerKind, syntax::ast::NodeId};
impl Checker<'_> {
    pub(super) fn validate_default_function(
        &self,
        key: NodeKey,
        definition: Option<DefId>,
        body: Option<NodeId>,
    ) -> Result<(), Error> {
        let Some(defaultness) = self.report.active_defaultness(key)? else {
            return Ok(());
        };
        if defaultness.node != key.node || body.is_none() {
            return Err(Error::at(
                key.file,
                defaultness.span.start,
                "default function requires its actual defining body",
            ));
        }
        let owner = definition
            .and_then(|id| self.facts.associated.owner(id))
            .and_then(|owner| self.facts.associated.get(owner));
        if owner.is_none_or(|owner| !matches!(owner.kind, OwnerKind::Impl { .. })) {
            return Err(Error::at(
                key.file,
                defaultness.span.start,
                "default function requires an actual impl owner",
            ));
        }
        Ok(())
    }
}
