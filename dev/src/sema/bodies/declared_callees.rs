//! Direct defining member calls retain the ordinary call receiver, not permission.
use super::{Checker, Constructor, DefId, Error, NodeId, TermId};
use crate::sema::function_context::FunctionContext;

impl Checker<'_> {
    /// Only decides whether to inspect effects; membership grants no permission.
    fn const_declaration(&self, owner: DefId) -> bool {
        self.facts.is_const_function(owner)
            || self
                .facts
                .associated
                .trait_owner(owner)
                .is_some_and(|parent| self.facts.associated.constant_trait(parent) == Some(true))
    }

    pub(super) fn invoked_value_obligations(
        &mut self,
        node: NodeId,
        mut receiver: TermId,
    ) -> Result<(), Error> {
        if self.callee_use_effect()? == crate::support::constness::Constness::Ordinary {
            return Ok(());
        }
        let owner = loop {
            self.state
                .infer
                .account_work(1)
                .map_err(|e| self.error(node, e))?;
            match self.head(node, receiver)? {
                Some(Constructor::Reference { .. }) => receiver = self.child(node, receiver, 0)?,
                Some(
                    Constructor::FunctionItem(item, _) | Constructor::FunctionItemArgs(item, _),
                ) => {
                    break DefId(item.0);
                }
                // An erased pointer retains its separate const-call rejection.
                _ => return Ok(()),
            }
        };
        if !self.const_declaration(owner) {
            return Ok(());
        }
        let arguments = self
            .state
            .infer
            .item_arguments(&self.facts.types, receiver)
            .map_err(|e| self.error(node, e))?;
        let parameters = self.callable_parameters(node, owner)?;
        let actual = arguments
            .iter()
            .copied()
            .filter(|argument| {
                !matches!(
                    argument,
                    crate::types::infer::arguments::Argument::Lifetime(_)
                )
            })
            .collect::<Vec<_>>();
        self.state
            .infer
            .account_work(arguments.len().saturating_add(parameters.len()))
            .map_err(|e| self.error(node, e))?;
        if parameters.len() != actual.len() {
            return Err(self.error(node, "invoked function value changed its parameter order"));
        }
        let bindings = parameters.into_iter().zip(actual).collect::<Vec<_>>();
        self.bound_obligations_for(
            node,
            owner,
            &arguments,
            &bindings,
            super::bounds::Use::InvokedValue,
        )
    }

    pub(super) fn retain_defining_member(
        &mut self,
        node: NodeId,
        callee: NodeId,
        receiver: TermId,
    ) -> Result<(), Error> {
        let Some(owner) = self.state.environment_owner else {
            return Ok(());
        };
        self.state
            .infer
            .account_work(4)
            .map_err(|e| self.error(node, e))?;
        if !self.const_declaration(owner) {
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
            FunctionContext::ConstFunction
                | FunctionContext::Comptime
                | FunctionContext::ConstTraitDefault { .. }
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
