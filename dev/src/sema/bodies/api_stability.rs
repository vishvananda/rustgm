//! Same-crate API naming is allowed; recursively const-stable bodies are separate.
use super::{Checker, DefId, Error, NodeId};
use crate::{
    resolve::stability::Library,
    syntax::ast::Kind,
    types::{Kind as TypeKind, infer::Solved},
};

impl Checker<'_> {
    pub(super) fn check_const_library_call(
        &mut self,
        node: NodeId,
        solved: &Solved,
    ) -> Result<(), Error> {
        let report = self.report;
        let Some(inventory) = report.package.library_policy().and_then(Library::inventory) else {
            return Ok(());
        };
        let Some(owner) = self.state.environment_owner else {
            return Ok(());
        };
        if !inventory.checks_const_body(owner) {
            return Ok(());
        }
        let function = self
            .facts
            .function_indices
            .get(&owner)
            .and_then(|&i| self.facts.functions.get(i))
            .ok_or_else(|| {
                self.error(
                    node,
                    "const API body requires its actual completed function",
                )
            })?;
        // Inline/closure children do not acquire the enclosing function's policy.
        if function.body != Some(self.state.body) {
            return Ok(());
        }
        self.state
            .infer
            .account_work(4)
            .map_err(|e| self.error(node, e))?;
        let callable = if let Some(method) = self.facts.methods.calls.get(&self.key(node)) {
            method.callable
        } else if let Kind::Call { callee, .. } = self.kind(node) {
            let term = self.term(callee)?;
            *solved
                .types
                .get(term.0 as usize)
                .ok_or_else(|| self.error(node, "const API call lost its solved callee"))?
        } else {
            return Err(self.error(
                node,
                "const-stable generated call API policy is not implemented",
            ));
        };
        let Some(TypeKind::FunctionItem { item, .. }) = self.facts.types.kind(callable) else {
            return Err(self.error(
                node,
                "const-stable indirect call API policy is not implemented",
            ));
        };
        inventory
            .const_body_call(owner, DefId(item.0))
            .map_err(|e| self.error(node, e))
    }
}
