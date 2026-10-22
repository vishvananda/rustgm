//! Intrinsic syntax never authorizes an ordinary body or a reified callable.
use super::{Checker, Constructor, DefId, Error, NodeId};
impl Checker<'_> {
    pub(super) fn require_source_closure_bindings(&self, node: NodeId) -> Result<(), Error> {
        self.facts
            .source_bindings
            .as_deref()
            .map_or(
                Ok(()),
                crate::sema::compiler_items::Catalog::require_closure_bindings,
            )
            .map_err(|e| self.error(node, e))
    }
    pub(super) fn require_reifiable_head(
        &self,
        node: NodeId,
        head: Option<Constructor>,
    ) -> Result<(), Error> {
        if let Some(Constructor::FunctionItem(item, _) | Constructor::FunctionItemArgs(item, _)) =
            head
        {
            self.require_reifiable_item(node, DefId(item.0))?;
        }
        Ok(())
    }
    pub(super) fn source_intrinsic(&self, node: NodeId, definition: DefId) -> Result<bool, Error> {
        self.facts
            .source_bindings
            .as_deref()
            .map_or(Ok(false), |c| c.intrinsic(definition))
            .map_err(|e| self.error(node, e))
    }
    pub(super) fn require_reifiable_item(
        &self,
        node: NodeId,
        definition: DefId,
    ) -> Result<(), Error> {
        if self.source_intrinsic(node, definition)? {
            return Err(self.error(
                node,
                "source intrinsic cannot be coerced or cast to a function pointer",
            ));
        }
        Ok(())
    }
}
