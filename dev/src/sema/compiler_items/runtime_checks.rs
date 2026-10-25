//! Runtime-check declarations and containing-session policy carry no call permission.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, model::DefId, package::Package},
    sema::model::{Declarations, Function},
    support::{
        bindings::SourceIntrinsic,
        runtime_checks::{RuntimeCheckBinding, RuntimeCheckConfig},
    },
    types::{Kind, Primitive, TypeId, callable::Convention, instance::Instance},
};

#[cfg(test)]
#[path = "../../../tests/invariants/ub_checks.rs"]
mod tests;

pub(super) struct RuntimeCheckSession {
    config: RuntimeCheckConfig,
    binding: RuntimeCheckBinding,
}
impl RuntimeCheckSession {
    pub(super) fn new(package: &Package) -> Self {
        let config = package.runtime_checks();
        Self {
            config,
            binding: RuntimeCheckBinding::for_crate(package.preserves_ub_checks(), config),
        }
    }
    pub(super) fn require_report(&self, package: &Package) -> Result<(), &'static str> {
        let expected = Self::new(package);
        if self.config != expected.config || self.binding != expected.binding {
            return Err("source catalog belongs to another runtime-check session policy");
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RuntimeCheckSelection {
    pub definition: DefId,
    pub signature: TypeId,
    pub fallback: bool,
}

pub(super) fn validate(facts: &Declarations, function: Function) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("source ub_checks requires a safe Rust signature without late regions");
    };
    if facts.types.list(parameters).is_none_or(|p| !p.is_empty())
        || result != TypeId::primitive(Primitive::Bool)
        || facts
            .generics
            .parameters
            .get(&function.definition)
            .is_some_and(|p| !p.is_empty())
        || facts
            .lifetimes
            .functions
            .get(&function.definition)
            .is_some_and(|r| r.total != 0)
        || facts
            .generics
            .declared_predicates_in_scope(function.definition)
            .next()
            .is_some()
    {
        return Err("source ub_checks requires no parameters or predicates and bool result");
    }
    if !facts.is_const_function(function.definition) {
        return Err("source ub_checks declaration must be const");
    }
    Ok(())
}

impl Catalog {
    pub(crate) fn runtime_check_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
    ) -> Result<Option<RuntimeCheckSelection>, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(instance.ty())
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        let Some((declaration, kind, expected)) = self.intrinsic_record(definition)? else {
            return Ok(None);
        };
        if kind != SourceIntrinsic::UbChecks {
            return Ok(None);
        }
        if signature != expected
            || facts
                .types
                .arguments(arguments)
                .is_none_or(|a| a.len() != 0)
        {
            return Err("selected source ub_checks changed its complete signature or arguments");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&i| facts.functions.get(i))
            .ok_or("source ub_checks has no completed declaration")?;
        if function.definition != definition
            || function.node != declaration.origin
            || function.signature != expected
        {
            return Err("source ub_checks completed declaration changed");
        }
        validate(facts, *function)?;
        Ok(Some(RuntimeCheckSelection {
            definition,
            signature,
            fallback: function.body.is_some(),
        }))
    }

    /// This supplies policy data only after the actual caller's proof is checked.
    pub(crate) fn runtime_check_binding(
        &self,
        origin: BodyOrigin,
    ) -> Result<(RuntimeCheckBinding, RuntimeCheckConfig), &'static str> {
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err("imported runtime-check operations require transported policy provenance");
        }
        Ok((self.runtime_checks.binding, self.runtime_checks.config))
    }
}
