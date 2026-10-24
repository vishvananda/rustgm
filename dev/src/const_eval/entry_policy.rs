//! A source function root has no reached caller from which to obtain conditional authority.
use super::{Failure, Machine};
use crate::body::ir::Owner;
impl Machine<'_, '_> {
    pub(super) fn check_source_function_root(&self, index: usize) -> Result<(), Failure> {
        self.evaluator.check_source_call_policy()?;
        let facts = self.evaluator.facts();
        if let Some(code) = self.bodies.code(index)
            && let Owner::Function { definition, .. } = code.owner
        {
            facts.check_const_member_execution(definition)?;
        }
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(());
        };
        self.bodies.validate_entry(index)?;
        let code = self
            .bodies
            .code(index)
            .ok_or("source function root has no Code")?;
        let Owner::Function { definition, .. } = code.owner else {
            return Ok(());
        };
        let header = facts
            .function_header(definition)?
            .ok_or("source function root header is pending")?;
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("source function root has no actual declaration")?;
        if header.declaration.source().is_none()
            || function.body != code.source.source()
            || !facts.is_const_function(definition)
        {
            return Err("source function root differs from its actual const owner".into());
        }
        let public = code
            .owner
            .signatures()
            .map(|(public, _)| public)
            .ok_or("source function root has no public signature")?;
        if let Some(instance) = code.instance {
            if !matches!(facts.types.kind(instance.ty()), Some(crate::types::Kind::FunctionItem { item, signature, .. })
                if item.0 == definition.0 && signature == public)
            {
                return Err(
                    "source function root changed its full instance or public signature".into(),
                );
            }
        } else if public != function.signature {
            return Err("source function root changed its original declaration signature".into());
        }
        if catalog.intrinsic_kind(definition)?.is_some() {
            return Err(
                "source intrinsic root execution requires its own reached caller witness".into(),
            );
        }
        if crate::sema::compiler_items::drop_glue_member(facts, definition)? {
            return Err("source drop glue constant cleanup execution is not implemented".into());
        }
        if catalog.requires_const_payment(definition)? {
            return Err(
                "source Conditional function root requires a reached caller payment".into(),
            );
        }
        if facts.generics.has_parameters(definition) {
            return Err("source generic function root execution without a reached caller is not implemented".into());
        }
        if let Some(instance) = code.instance {
            let Some(crate::types::Kind::FunctionItem { arguments, .. }) =
                facts.types.kind(instance.ty())
            else {
                return Err("source function root lost its complete function item".into());
            };
            if facts
                .types
                .arguments(arguments)
                .is_none_or(|mut arguments| arguments.next().is_some())
            {
                return Err("source function root has unexpected generic arguments".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/function_roots.rs"]
mod tests;
