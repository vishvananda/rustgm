//! Direct defining member calls retain the ordinary call receiver, not permission.
use super::{Checker, Constructor, DefId, Error, NodeId, TermId};
use crate::sema::function_context::FunctionContext;

impl Checker<'_> {
    pub(super) fn retain_defining_member(
        &mut self,
        node: NodeId,
        callee: NodeId,
        receiver: TermId,
    ) -> Result<(), Error> {
        let Some(owner) = self.state.environment_owner else {
            return Ok(());
        };
        let Some(trait_owner) = self.facts.associated.trait_owner(owner) else {
            return Ok(());
        };
        if self.facts.associated.constant_trait(trait_owner) != Some(true) {
            return Ok(());
        }
        self.state
            .infer
            .account_work(8)
            .map_err(|e| self.error(node, e))?;
        let function = self
            .facts
            .function_indices
            .get(&owner)
            .and_then(|&index| self.facts.functions.get(index))
            .ok_or_else(|| self.error(node, "defining member requires its completed function"))?;
        // Closure bodies retain their enclosing environment but not its context.
        if function.body != Some(self.state.body) {
            return Ok(());
        }
        if !matches!(
            self.facts
                .function_context(owner, self.state.body)
                .map_err(|e| self.error(node, e))?,
            FunctionContext::ConstTraitDefault { .. }
        ) {
            return Ok(());
        }
        let Some(Constructor::FunctionItem(item, _) | Constructor::FunctionItemArgs(item, _)) =
            self.head(node, receiver)?
        else {
            return Ok(());
        };
        self.state
            .infer
            .account_work(4)
            .map_err(|e| self.error(node, e))?;
        let definition = DefId(item.0);
        let Some(target_owner) = self.facts.associated.trait_owner(definition) else {
            return Ok(());
        };
        if self.facts.associated.constant_trait(target_owner) != Some(true) {
            return Ok(());
        }
        self.facts
            .function_header(definition)
            .map_err(|e| self.error(node, e))?
            .ok_or_else(|| self.error(node, "defining member target header is pending"))?;
        // The existing inference finish resolves this full receiver and all its
        // arguments before publish_invocations consumes Solved.types. Partial
        // inference never publishes a completed receiver or defining permission.
        self.retain_invocation(node, callee, receiver, 0, None)
    }
}
