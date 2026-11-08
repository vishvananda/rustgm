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

#[cfg(test)]
#[path = "../../../tests/invariants/fallback_exposure.rs"]
mod tests;

impl Checker<'_> {
    pub(super) fn check_const_library_call(
        &mut self,
        node: NodeId,
        solved: &Solved,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        let comptime_callable = if self.facts.comptime_calls_need_checking() {
            let callable = self.source_call_callable(node, solved)?;
            self.check_comptime_callable(node, callable)?;
            Some(callable)
        } else {
            None
        };
        let owner = self.const_library_owner(node)?;
        if self
            .report
            .resolution
            .crates
            .as_deref()
            .is_none_or(|imports| !imports.has_availability())
            && owner.is_none()
            && (self.facts.source_bindings.is_none()
                || self.callee_use_effect()? == crate::support::constness::Constness::Ordinary)
        {
            return Ok(());
        }
        let callable = match comptime_callable {
            Some(callable) => callable,
            None => self.source_call_callable(node, solved)?,
        };
        self.check_const_library_target(node, owner, callable, allowances)
    }

    fn source_call_callable(&mut self, node: NodeId, solved: &Solved) -> Result<TypeId, Error> {
        Ok(
            if let Some(method) = self.facts.methods.calls.get(&self.key(node)) {
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
            },
        )
    }

    /// Generated protocol calls retain their actual callable independently.
    pub(super) fn check_const_library_callable(
        &mut self,
        node: NodeId,
        callable: TypeId,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        self.check_comptime_callable(node, callable)?;
        let owner = self.const_library_owner(node)?;
        if self
            .report
            .resolution
            .crates
            .as_deref()
            .is_none_or(|imports| !imports.has_availability())
            && owner.is_none()
            && (self.facts.source_bindings.is_none()
                || self.callee_use_effect()? == crate::support::constness::Constness::Ordinary)
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
        if !self.safe_on_stable(inventory, owner)? {
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
        let context = self
            .facts
            .source_function_context(report, owner, self.state.body)
            .map_err(|error| self.error(node, error))?;
        // Always bodies are actual Const contexts, including when their API is
        // const-stable. Recursive ConstFn API restrictions do not apply there.
        if !context.checks_const()
            || context == crate::sema::function_context::FunctionContext::Comptime
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
        crate::resolve::stability::check_imported_api(self.report, self.key(node), target, false)?;
        if self.callee_use_effect()? != crate::support::constness::Constness::Ordinary {
            crate::resolve::stability::check_imported_api(
                self.report,
                self.key(node),
                target,
                true,
            )?;
        }
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
                    Kind::OffsetOf { .. } => {
                        crate::resolve::require_offset(self.report, self.key(node))?
                    }
                    _ => ast.node(node).span,
                };
                if !requested && !allowances.allows_at(self.state.file, span, &feature)? {
                    return Err(self.error(node, "source intrinsic const API requires its checked feature or exact token allowance"));
                }
            }
            let exempt = self.const_body_exemption(node, owner, policy, inventory)?;
            if owner.is_some()
                && !exempt
                && !marker
                && policy.is_none_or(|policy| !matches!(policy.level, Level::Stable(_)))
                && !self.exposed_fallback(inventory, target)?
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
        if self.const_constructor(node, target, callable)? {
            return Ok(());
        }
        let inventory = inventory
            .ok_or_else(|| self.error(node, "const API call lost its actual library inventory"))?;
        let exempt = self.const_body_exemption(
            node,
            owner,
            inventory.const_declaration(target),
            Some(inventory),
        )?;
        if exempt || self.safe_on_stable(inventory, target)? {
            Ok(())
        } else {
            Err(self.error(
                node,
                "const-stable body calls a function without const-stable API policy",
            ))
        }
    }
    /// Const-stable, explicitly or implicitly const-stable-indirect bodies are
    /// recursively checked, so they are safe to expose to stable const code.
    fn safe_on_stable(
        &self,
        inventory: &crate::resolve::stability::Inventory,
        definition: DefId,
    ) -> Result<bool, Error> {
        Ok(inventory.checks_const_body(definition)
            || (inventory.const_declaration(definition).is_some()
                && self.report.package.source_core().is_some()
                && self.facts.const_panic_markers(self.report, definition)?.1))
    }
    /// An intrinsic without its own indirect marker is still exposable when its
    /// source fallback body is itself recursively checked; a bodiless one is not.
    fn exposed_fallback(
        &self,
        inventory: Option<&crate::resolve::stability::Inventory>,
        target: DefId,
    ) -> Result<bool, Error> {
        let Some(inventory) = inventory else {
            return Ok(false);
        };
        let fallback = self
            .facts
            .function_indices
            .get(&target)
            .and_then(|&index| self.facts.functions.get(index))
            .is_some_and(|function| function.definition == target && function.body.is_some());
        Ok(fallback && self.safe_on_stable(inventory, target)?)
    }
    fn const_body_exemption(
        &mut self,
        node: NodeId,
        owner: Option<DefId>,
        policy: Option<&crate::resolve::stability::Policy>,
        inventory: Option<&crate::resolve::stability::Inventory>,
    ) -> Result<bool, Error> {
        let (Some(owner), Some(policy), Some(inventory)) = (owner, policy, inventory) else {
            return Ok(false);
        };
        let feature = policy.feature_name().map_err(|e| self.error(node, e))?;
        inventory
            .const_body_allows(owner, &feature, &mut self.state.steps)
            .map_err(|e| self.error(node, e))
    }
    // Tuple/variant construction is a language operation, not a function body
    // whose API can be declared const-stable. Ordinary API and field/type checks
    // remain at their existing call/constructor boundaries.
    fn const_constructor(
        &mut self,
        node: NodeId,
        target: DefId,
        callable: TypeId,
    ) -> Result<bool, Error> {
        use crate::{
            resolve::model::{DefKind, Namespace},
            sema::members::ConstructorKind,
        };
        self.state
            .infer
            .account_work(1)
            .map_err(|e| self.error(node, e))?;
        let Some(constructor) = self.facts.members.constructors.get(&target) else {
            return Ok(false);
        };
        if constructor.kind != ConstructorKind::Variant {
            return Ok(false);
        }
        self.state
            .infer
            .account_work(11)
            .map_err(|e| self.error(node, e))?;
        let definition = self
            .report
            .resolution
            .definitions
            .get(target.0 as usize)
            .ok_or_else(|| self.error(node, "const constructor lost its actual declaration"))?;
        let variant = self
            .facts
            .members
            .variants
            .get(constructor.variant.0 as usize)
            .ok_or_else(|| self.error(node, "const constructor lost its actual variant"))?;
        if !matches!(definition.kind, DefKind::Constructor | DefKind::Variant)
            || definition.namespace != Namespace::Value
            || definition.node.is_none()
            || definition.node != variant.origin.source()
            || variant.shape != crate::syntax::item::Shape::Tuple
            || self
                .facts
                .members
                .variants(variant.owner)
                .get(variant.ordinal as usize)
                != Some(&constructor.variant)
        {
            return Err(self.error(node, "const constructor requires its actual source owner"));
        }
        let Some(TypeKind::FunctionItem {
            item, signature, ..
        }) = self.facts.types.kind(callable)
        else {
            return Err(self.error(node, "const constructor lost its actual callable"));
        };
        let Some(TypeKind::Function {
            parameters,
            result,
            unsafe_: false,
            convention,
            ..
        }) = self.facts.types.kind(signature)
        else {
            return Err(self.error(node, "const constructor lost its safe signature"));
        };
        let fields = variant
            .end
            .checked_sub(variant.start)
            .ok_or_else(|| self.error(node, "const constructor has invalid field coordinates"))?;
        if item.0 != target.0
            || convention != crate::types::callable::Convention::RUST
            || !matches!(self.facts.types.kind(result),Some(TypeKind::Adt(owner, _)) if owner==variant.owner)
            || self
                .facts
                .types
                .list(parameters)
                .is_none_or(|parameters| parameters.len() != fields as usize)
        {
            return Err(self.error(
                node,
                "const constructor differs from its actual owner or field signature",
            ));
        }
        Ok(true)
    }
}
