//! Same-crate API naming is allowed; recursively const-stable bodies are separate.
use super::{Checker, DefId, Error, NodeId};
use crate::{
    resolve::stability::Library,
    syntax::ast::Kind,
    types::{Kind as TypeKind, TypeId, infer::Solved},
};

impl Checker<'_> {
    pub(super) fn check_const_library_call(
        &mut self,
        node: NodeId,
        solved: &Solved,
    ) -> Result<(), Error> {
        let Some(owner) = self.const_library_owner(node)? else {
            return Ok(());
        };
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
        self.check_const_library_target(node, owner, callable)
    }

    /// Generated protocol calls retain their actual callable independently.
    pub(super) fn check_const_library_callable(
        &mut self,
        node: NodeId,
        callable: TypeId,
    ) -> Result<(), Error> {
        let Some(owner) = self.const_library_owner(node)? else {
            return Ok(());
        };
        self.check_const_library_target(node, owner, callable)
    }

    fn const_library_owner(&mut self, node: NodeId) -> Result<Option<DefId>, Error> {
        let report = self.report;
        let Some(inventory) = report.package.library_policy().and_then(Library::inventory) else {
            return Ok(None);
        };
        let Some(owner) = self.state.environment_owner else {
            return Ok(None);
        };
        if !inventory.checks_const_body(owner) {
            return Ok(None);
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
            return Ok(None);
        }
        self.state
            .infer
            .account_work(4)
            .map_err(|e| self.error(node, e))?;
        Ok(Some(owner))
    }

    fn check_const_library_target(
        &mut self,
        node: NodeId,
        owner: DefId,
        callable: TypeId,
    ) -> Result<(), Error> {
        let Some(TypeKind::FunctionItem { item, .. }) = self.facts.types.kind(callable) else {
            return Err(self.error(
                node,
                "const-stable indirect call API policy is not implemented",
            ));
        };
        self.report
            .package
            .library_policy()
            .and_then(Library::inventory)
            .ok_or_else(|| self.error(node, "const API call lost its actual library inventory"))?
            .const_body_call(owner, DefId(item.0))
            .map_err(|e| self.error(node, e))
    }
}
