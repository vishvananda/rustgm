//! Same-crate API naming is allowed; recursively const-stable bodies are separate.
use super::{Checker, DefId, Error, NodeId};
use crate::{
    resolve::{
        expansion::MacroAllowanceSession,
        stability::{Level, Library},
    },
    syntax::ast::Kind,
    types::{Kind as TypeKind, TypeId, infer::Solved},
};

impl Checker<'_> {
    pub(super) fn check_const_library_call(
        &mut self,
        node: NodeId,
        solved: &Solved,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        let owner = self.const_library_owner(node)?;
        if owner.is_none()
            && (self.facts.source_bindings.is_none()
                || self.callee_use_effect()? == crate::support::constness::Constness::Ordinary)
        {
            return Ok(());
        }
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
        self.check_const_library_target(node, owner, callable, allowances)
    }

    /// Generated protocol calls retain their actual callable independently.
    pub(super) fn check_const_library_callable(
        &mut self,
        node: NodeId,
        callable: TypeId,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        let owner = self.const_library_owner(node)?;
        if owner.is_none()
            && self.callee_use_effect()? == crate::support::constness::Constness::Ordinary
        {
            return Ok(());
        }
        self.check_const_library_target(node, owner, callable, allowances)
    }

    fn const_library_owner(&mut self, node: NodeId) -> Result<Option<DefId>, Error> {
        let report = self.report;
        let Some(inventory) = report.package.library_policy().and_then(Library::inventory) else {
            return Ok(None);
        };
        let Some(owner) = self.state.environment_owner else {
            return Ok(None);
        };
        if !inventory.checks_const_body(owner)
            && !(inventory.const_declaration(owner).is_some()
                && self.report.package.source_core().is_some()
                && self.facts.const_panic_markers(report, owner)?.1)
        {
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
        if !self
            .facts
            .source_function_context(report, owner, self.state.body)
            .map_err(|error| self.error(node, error))?
            .checks_const()
        {
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
        owner: Option<DefId>,
        callable: TypeId,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        let Some(TypeKind::FunctionItem { item, .. }) = self.facts.types.kind(callable) else {
            return if owner.is_some() {
                Err(self.error(
                    node,
                    "const-stable indirect call API policy is not implemented",
                ))
            } else {
                Ok(())
            };
        };
        let target = DefId(item.0);
        let intrinsic = self
            .facts
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.intrinsic_kind(target))
            .transpose()
            .map_err(|e| self.error(node, e))?
            .flatten();
        let library = self.report.package.library_policy();
        let inventory = library.and_then(Library::inventory);
        if intrinsic.is_some()
            && self.callee_use_effect()? != crate::support::constness::Constness::Ordinary
        {
            let marker = self
                .facts
                .intrinsic_const_stable_indirect(self.report, target)?;
            let policy = inventory.and_then(|inventory| inventory.const_declaration(target));
            if let Some(policy) =
                policy.filter(|policy| matches!(policy.level, Level::Unstable { .. }))
            {
                let feature = policy.feature_name().map_err(|e| self.error(node, e))?;
                let requested = library
                    .ok_or_else(|| self.error(node, "intrinsic const API lost its library owner"))?
                    .requested_feature(&feature, &mut self.state.steps)
                    .map_err(|e| self.error(node, e))?;
                let ast = &self.report.package.unit(self.state.file).configured.ast;
                let span = match ast.node(node).kind {
                    Kind::Call { callee, .. } => ast.node(callee).span,
                    Kind::MethodCall { name, .. } => name,
                    Kind::GenericMethodCall { method, .. } => ast.node(method).span,
                    _ => ast.node(node).span,
                };
                if !requested && !allowances.allows_at(self.state.file, span, &feature)? {
                    return Err(self.error(node, "source intrinsic const API requires its checked feature or exact token allowance"));
                }
            }
            if owner.is_some()
                && !marker
                && policy.is_none_or(|policy| !matches!(policy.level, Level::Stable(_)))
            {
                return Err(self.error(
                    node,
                    "const-stable body cannot expose this unstable intrinsic",
                ));
            }
            return Ok(());
        }
        if owner.is_none() {
            return Ok(());
        }
        let inventory = inventory
            .ok_or_else(|| self.error(node, "const API call lost its actual library inventory"))?;
        if inventory.checks_const_body(target)
            || (inventory.const_declaration(target).is_some()
                && self.report.package.source_core().is_some()
                && self.facts.const_panic_markers(self.report, target)?.1)
        {
            Ok(())
        } else {
            Err(self.error(
                node,
                "const-stable body calls a function without const-stable API policy",
            ))
        }
    }
}
