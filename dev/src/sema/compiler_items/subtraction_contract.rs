//! Subtraction descriptors bind real source declarations, never unsafe/const permission.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{
        header_attributes::ContractKnowledge,
        model::{Declarations, Function},
    },
    support::bindings::{LangItem, SourceIntrinsic},
    types::{
        Kind, Primitive, TypeId, arguments::Argument, callable::Convention, instance::Instance,
        parameters::ParameterKind,
    },
};

pub(crate) struct SubtractionSelection {
    pub definition: DefId,
    pub signature: TypeId,
    pub input: TypeId,
    pub output: TypeId,
    pub kind: SourceIntrinsic,
}
impl SubtractionSelection {
    pub(crate) fn opened_signature_matches(
        &self,
        store: &crate::types::Store,
        signature: TypeId,
    ) -> bool {
        super::intrinsic_contracts::opened_signature_matches(
            store,
            signature,
            &[self.input; 2],
            self.output,
            self.kind == SourceIntrinsic::UncheckedSub,
            Convention::RUST,
        )
    }
}
pub(super) fn validate(
    facts: &Declarations,
    function: Function,
    kind: SourceIntrinsic,
) -> Result<(), &'static str> {
    let Some([parameter]) = facts
        .generics
        .parameters
        .get(&function.definition)
        .map(Vec::as_slice)
    else {
        return Err("source subtraction requires one own Copy type formal");
    };
    let Argument::Type(input) = parameter.argument else {
        return Err("source subtraction requires a type formal");
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(input) else {
        return Err("source subtraction lost its actual formal");
    };
    let lifetime = facts
        .lifetimes
        .functions
        .get(&function.definition)
        .ok_or("source subtraction requires its completed lifetime scheme")?;
    if parameter.anonymous
        || parameter.default.is_some()
        || facts.types.parameter(id).is_none_or(|p| {
            p.owner.0 != function.definition.0
                || p.index != 0
                || p.kind != ParameterKind::Type { sized: true }
        })
        || function.body.is_some()
        || !facts.is_const_function(function.definition)
        || lifetime.total != 0
        || lifetime.captures != 0
        || !lifetime.early.is_empty()
        || !lifetime.late.is_empty()
        || !lifetime.bounds.is_empty()
    {
        return Err(
            "source subtraction requires its own bodyless const declaration without regions",
        );
    }
    signature(facts, function.signature, input, kind)?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source subtraction has no actual catalog")?;
    super::compare_intrinsic::copy_premise(facts, function.definition, input, catalog)
}
fn signature(
    facts: &Declarations,
    signature: TypeId,
    input: TypeId,
    kind: SourceIntrinsic,
) -> Result<TypeId, &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err("source subtraction requires its Rust signature without late regions");
    };
    if unsafe_ != (kind == SourceIntrinsic::UncheckedSub)
        || facts.types.list(parameters) != Some(&[input; 2])
    {
        return Err("source subtraction changed its safety or two same-type inputs");
    }
    let valid = match kind {
        SourceIntrinsic::UncheckedSub => result == input,
        SourceIntrinsic::SubWithOverflow => {
            matches!(facts.types.kind(result), Some(Kind::Tuple(fields)) if facts.types.list(fields)==Some(&[input,TypeId::primitive(Primitive::Bool)]))
        }
        _ => false,
    };
    if !valid {
        return Err("source subtraction changed its exact scalar or (scalar,bool) result");
    }
    Ok(result)
}
impl Catalog {
    pub(crate) fn subtraction_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<SubtractionSelection>, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature: selected_signature,
        }) = facts.types.kind(instance.ty())
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        let Some((declaration, kind, original)) = self.intrinsic_record(definition)? else {
            return Ok(None);
        };
        if !matches!(
            kind,
            SourceIntrinsic::UncheckedSub | SourceIntrinsic::SubWithOverflow
        ) {
            return Ok(None);
        }
        crate::sema::header_attributes::charge_source_contract_query(facts, 32)
            .map_err(|_| "source subtraction selection work exhausted")?;
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err(
                "source subtraction operation provenance requires its wider transport profile",
            );
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|i| facts.functions.get(*i))
            .ok_or("source subtraction declaration is missing")?;
        if function.definition != definition
            || function.node != declaration.origin
            || function.signature != original
            || declaration.parent.is_some()
        {
            return Err("source subtraction changed its actual declaration");
        }
        validate(facts, *function, kind)?;
        self.validated_language(LangItem::Copy)?
            .ok_or("source subtraction requires its completed actual Copy header")?;
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("invalid source subtraction arguments")?;
        let (Some(Argument::Type(input)), None) = (args.next(), args.next()) else {
            return Err("source subtraction requires its complete single-type instance");
        };
        if !matches!(facts.types.kind(input),Some(Kind::Primitive(p)) if p.integer().is_some()) {
            return Err("source subtraction execution requires its closed integer scalar type");
        }
        let output = signature(facts, selected_signature, input, kind)?;
        let roots =
            super::intrinsic_contracts::item_roots(&facts.types, [input, output], |work| {
                crate::sema::header_attributes::charge_source_contract_query(facts, work)
                    .map_err(|_| "source subtraction opening work exhausted")
            })?
            .ok_or("source subtraction has no canonical caller roots")?;
        if roots != [input, output] {
            return Err("closed source subtraction roots changed through their callable binder");
        }
        Ok(Some(SubtractionSelection {
            definition,
            signature: selected_signature,
            input,
            output,
            kind,
        }))
    }
    pub(crate) fn require_subtraction_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &SubtractionSelection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let Some((declaration, kind, _)) = self.intrinsic_record(selected.definition)? else {
            return Err("source subtraction lost its attributed declaration");
        };
        if kind != selected.kind
            || signature(facts, selected.signature, selected.input, kind)? != selected.output
        {
            return Err("source subtraction changed its selected signature or intrinsic kind");
        }
        let header = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source subtraction has no current source header")?;
        if header
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(header.must_use, ContractKnowledge::Unknown)
            || matches!(header.inline, ContractKnowledge::Unknown)
        {
            return Err("source subtraction requires its positive actual source header");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/subtracting_source.rs"]
mod tests;
