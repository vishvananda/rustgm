//! `assume` binds its real source declaration; its fallback body is never a constant result.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{header_attributes::ContractKnowledge, model::Declarations},
    support::bindings::SourceIntrinsic,
    types::{Kind, Primitive, TypeId, callable::Convention, instance::Instance},
};

#[cfg(test)]
#[path = "../../../tests/invariants/assume_constants.rs"]
mod tests;

/// Only the pinned `assume` identity; every other declaration keeps its own owner.
pub(crate) fn is_assumption(kind: SourceIntrinsic) -> bool {
    matches!(kind, SourceIntrinsic::Declaration(identity) if identity.name() == "assume")
}

/// A validated `unsafe const fn(bool)` declaration of the current source provider.
pub(crate) struct AssumeSelection {
    definition: DefId,
    signature: TypeId,
}
impl AssumeSelection {
    pub(crate) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) const fn signature(&self) -> TypeId {
        self.signature
    }
}

impl Catalog {
    pub(crate) fn assumption(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<AssumeSelection>, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(instance.ty())
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        let Some((declaration, kind, original)) = self.intrinsic_record(definition)? else {
            return Ok(None);
        };
        if !is_assumption(kind) {
            return Ok(None);
        }
        crate::sema::header_attributes::charge_source_contract_query(facts, 12)
            .map_err(|_| "source assume selection work budget exhausted")?;
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err("source assume provenance requires its wider transport profile");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("source assume requires its completed actual function")?;
        if declaration.parent.is_some()
            || function.definition != definition
            || function.node != declaration.origin
            || function.signature != original
            || signature != original
        {
            return Err("source assume completed declaration changed");
        }
        if facts
            .types
            .arguments(arguments)
            .is_none_or(|actual| actual.len() != 0)
        {
            return Err("source assume instance must have no generic arguments");
        }
        let condition = TypeId::primitive(Primitive::Bool);
        if !matches!(facts.types.kind(signature), Some(Kind::Function {
            bound_regions: 0, parameters, result, unsafe_: true, convention: Convention::RUST,
        }) if result == TypeId::UNIT && facts.types.list(parameters) == Some(&[condition]))
        {
            return Err("source assume changed its exact unsafe Rust (bool) signature");
        }
        Ok(Some(AssumeSelection {
            definition,
            signature,
        }))
    }
    pub(crate) fn require_assume_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &AssumeSelection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let Some((declaration, kind, _)) = self.intrinsic_record(selected.definition)? else {
            return Err("source assume lost its attributed declaration");
        };
        if !is_assumption(kind) {
            return Err("source assume changed its intrinsic identity");
        }
        let current = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source assume lost its current source header")?;
        if current
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(current.must_use, ContractKnowledge::Unknown)
            || matches!(current.inline, ContractKnowledge::Unknown)
            || !facts.is_const_function(selected.definition)
        {
            return Err("source assume requires its actual completed const declaration");
        }
        Ok(())
    }
}
