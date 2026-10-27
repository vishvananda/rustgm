//! The attributed intrinsic returns the genuine completed source Location type.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::model::{Declarations, Function},
    support::bindings::{LangItem, SourceIntrinsic},
    types::{Kind, RegionId, TypeId, arguments::Argument, callable::Convention, instance::Instance},
};

pub(super) fn validate(facts: &Declarations, function: Function) -> Result<(), &'static str> {
    let Some(Kind::Function { bound_regions: 0, parameters, result, unsafe_: false,
        convention: Convention::RUST }) = facts.types.kind(function.signature)
    else { return Err("source caller_location requires a safe Rust signature without late regions"); };
    if facts.types.list(parameters).is_none_or(|parameters| !parameters.is_empty())
        || facts.generics.parameters.get(&function.definition).is_some_and(|parameters| !parameters.is_empty())
        || facts.lifetimes.functions.get(&function.definition).is_some_and(|regions| regions.total != 0)
        || facts.generics.declared_predicates_in_scope(function.definition).next().is_some()
        || !facts.is_const_function(function.definition) || function.body.is_some()
    { return Err("source caller_location requires a const bodyless declaration without parameters or predicates"); }
    let Some(Kind::Reference { referent, region: RegionId::STATIC, mutable: false }) = facts.types.kind(result)
    else { return Err("source caller_location requires a shared static Location reference"); };
    let Some(Kind::Adt(item, arguments)) = facts.types.kind(referent)
    else { return Err("source caller_location requires its actual Location nominal"); };
    let location = facts.source_bindings.as_deref().and_then(|catalog| catalog.language(LangItem::PanicLocation))
        .ok_or("source caller_location requires the actual panic_location declaration")?;
    if item.0 != location.definition.0 || facts.types.arguments(arguments).is_none_or(|mut arguments|
        arguments.next() != Some(Argument::Lifetime(RegionId::STATIC)) || arguments.next().is_some())
    { return Err("source caller_location changed its actual Location identity or static lifetime"); }
    Ok(())
}

/// Declaration shape data; actual reached callers still owe their complete proof.
pub(crate) struct CallerLocationSelection {
    pub definition: DefId,
    pub signature: TypeId,
    pub result: TypeId,
}
impl Catalog {
    pub(crate) fn caller_location_selection(&self, facts: &Declarations, instance: Instance,
        origin: BodyOrigin) -> Result<Option<CallerLocationSelection>, &'static str>
    {
        let Some(Kind::FunctionItem { item, arguments, signature }) = facts.types.kind(instance.ty())
        else { return Ok(None); };
        let definition = DefId(item.0);
        let Some((declaration, SourceIntrinsic::CallerLocation, expected)) = self.intrinsic_record(definition)?
        else { return Ok(None); };
        if origin.source().is_none() || matches!(self.bindings, super::binding_views::Bindings::Imported(_)) {
            return Err("imported caller_location requires transported operation policy");
        }
        if signature != expected || declaration.parent.is_some()
            || facts.types.arguments(arguments).is_none_or(|arguments| arguments.len() != 0)
        { return Err("source caller_location changed its complete instance or signature"); }
        let function = facts.function_indices.get(&definition).and_then(|&index| facts.functions.get(index))
            .ok_or("source caller_location lacks its actual completed declaration")?;
        if function.node != declaration.origin || function.signature != signature {
            return Err("source caller_location changed its declaration owner");
        }
        validate(facts, *function)?;
        self.validated_language(LangItem::PanicLocation)?
            .ok_or("source caller_location requires completed Location fields and lifetime")?;
        let Some(Kind::Function { result, .. }) = facts.types.kind(signature) else {
            return Err("source caller_location lost its function signature");
        };
        Ok(Some(CallerLocationSelection { definition, signature, result }))
    }
    pub(crate) fn require_caller_location_report(&self, report: &Report, facts: &Declarations,
        selected: &CallerLocationSelection) -> Result<(), &'static str>
    {
        self.require_provider_report(report)?;
        let Some((declaration, SourceIntrinsic::CallerLocation, signature)) = self.intrinsic_record(selected.definition)?
        else { return Err("source caller_location lost its actual attributed declaration"); };
        if signature != selected.signature { return Err("source caller_location signature changed"); }
        let view = facts.header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source caller_location lost its current header")?;
        if view.origin.source().is_none_or(|site| site.declaration != declaration.origin)
            || matches!(view.must_use, crate::sema::header_attributes::ContractKnowledge::Unknown)
            || matches!(view.inline, crate::sema::header_attributes::ContractKnowledge::Unknown)
        { return Err("source caller_location requires its exact completed source header"); }
        Ok(())
    }
}
