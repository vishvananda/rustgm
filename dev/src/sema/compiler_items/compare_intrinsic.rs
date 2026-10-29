//! Genuine `three_way_compare` consumes scalar operands and the actual Ordering declaration.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{
        header_attributes::ContractKnowledge,
        members::VariantId,
        model::{Declarations, Function},
        values::Reader,
    },
    support::{
        bindings::{LangItem, SourceIntrinsic},
        constness::Constness,
    },
    types::{
        Kind, Primitive, TypeId, aggregate,
        arguments::Argument,
        callable::Convention,
        instance::Instance,
        parameters::{ParameterKind, clauses::Clause, predicates::Bound},
    },
};
/// Transient data only; caller/body/const/Copy proof remains with ordinary consumers.
pub(crate) struct ThreeWaySelection {
    pub definition: DefId,
    pub signature: TypeId,
    pub input: TypeId,
    pub output: TypeId,
    pub variants: [VariantId; 3],
}
impl ThreeWaySelection {
    pub(crate) fn pending_tag(
        &self,
        facts: &Declarations,
    ) -> Result<Option<VariantId>, &'static str> {
        for (id, expected) in self.variants.into_iter().zip([255, 0, 1]) {
            let Some((primitive, bits)) = Reader::new(facts).discriminant(id)? else {
                return Ok(Some(id));
            };
            if primitive != Primitive::I8 || bits != expected {
                return Err(
                    "source three_way_compare requires actual Ordering discriminants -1/0/1",
                );
            }
        }
        Ok(None)
    }
}
pub(super) fn validate(facts: &Declarations, function: Function) -> Result<(), &'static str> {
    let Some([parameter]) = facts
        .generics
        .parameters
        .get(&function.definition)
        .map(Vec::as_slice)
    else {
        return Err("source three_way_compare requires one own Copy type parameter");
    };
    let Argument::Type(input) = parameter.argument else {
        return Err("three_way_compare requires a type formal");
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(input) else {
        return Err("three_way_compare lost its type formal");
    };
    let lifetimes = facts
        .lifetimes
        .functions
        .get(&function.definition)
        .ok_or("three_way_compare requires its actual completed lifetime scheme")?;
    if parameter.anonymous
        || parameter.default.is_some()
        || facts.types.parameter(id).is_none_or(|p| {
            p.owner.0 != function.definition.0
                || p.index != 0
                || p.kind != ParameterKind::Type { sized: true }
        })
        || !facts.is_const_function(function.definition)
        || function.body.is_some()
        || lifetimes.total != 0
        || lifetimes.captures != 0
        || !lifetimes.early.is_empty()
        || !lifetimes.late.is_empty()
        || !lifetimes.bounds.is_empty()
    {
        return Err(
            "source three_way_compare requires its actual bodyless const declaration and own Sized formal",
        );
    }
    let Some(Kind::Function { result, .. }) = facts.types.kind(function.signature) else {
        return Err("three_way_compare lost its actual function signature");
    };
    selected_signature(facts, function.signature, input, result)?;
    ordering_identity(facts, result)?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("three_way_compare lacks its source catalog")?;
    copy_premise(facts, function.definition, input, catalog)
}
pub(super) fn copy_premise(
    facts: &Declarations,
    definition: DefId,
    input: TypeId,
    catalog: &Catalog,
) -> Result<(), &'static str> {
    let copying = catalog
        .language(LangItem::Copy)
        .ok_or("three_way_compare requires actual Copy")?
        .definition;
    let sized = catalog
        .header_role(LangItem::Sized)?
        .ok_or("three_way_compare requires its actual Sized declaration")?
        .definition;
    let meta = catalog
        .header_role(LangItem::MetaSized)?
        .ok_or("three_way_compare requires its actual MetaSized default")?
        .definition;
    // The existing prefix excludes elaborated Copy/Clone supertrait assumptions.
    // Those are consequences, not additional written intrinsic premises.
    let predicates = facts.generics.declared_predicates(definition);
    if predicates.len() > 3 {
        return Err("three_way_compare has extra own declaration premises");
    }
    let mut found = false;
    for predicate in predicates {
        let Clause::Predicate(predicate) = predicate.value else {
            return Err("three_way_compare has unsupported quantified premises");
        };
        if predicate.ty != input {
            return Err("three_way_compare premise belongs to another subject");
        }
        if predicate.bound == Bound::Sized {
            continue;
        }
        let Some((id, Constness::Ordinary)) = predicate.bound.trait_parts() else {
            return Err("three_way_compare requires its own ordinary source Copy premise");
        };
        let application = facts
            .types
            .trait_application(id)
            .ok_or("invalid three_way_compare premise")?;
        if (application.item.0 == sized.0 || application.item.0 == meta.0)
            && facts
                .types
                .arguments(application.arguments)
                .is_some_and(|a| a.len() == 0)
        {
            continue;
        }
        if application.item.0 != copying.0
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|a| a.len() != 0)
            || std::mem::replace(&mut found, true)
        {
            return Err("three_way_compare changed its exact own Copy premise");
        }
    }
    if !found {
        return Err("three_way_compare requires its actual own Copy premise");
    }
    Ok(())
}
fn selected_signature(
    facts: &Declarations,
    signature: TypeId,
    input: TypeId,
    output: TypeId,
) -> Result<(), &'static str> {
    if !matches!(facts.types.kind(signature), Some(Kind::Function {
        bound_regions:0, parameters, result, unsafe_:false, convention:Convention::RUST,
    }) if result == output && facts.types.list(parameters) == Some(&[input,input]))
    {
        return Err("three_way_compare requires two same-type values and actual Ordering result");
    }
    Ok(())
}
fn ordering_identity(
    facts: &Declarations,
    output: TypeId,
) -> Result<crate::types::ItemId, &'static str> {
    let declaration = facts
        .source_bindings
        .as_deref()
        .and_then(|c| c.language(LangItem::Ordering))
        .ok_or("three_way_compare requires its actual Ordering language declaration")?;
    let Some(Kind::Adt(item, arguments)) = facts.types.kind(output) else {
        return Err("three_way_compare result is not its actual Ordering nominal");
    };
    if item.0 != declaration.definition.0
        || facts
            .types
            .arguments(arguments)
            .is_none_or(|a| a.len() != 0)
        || facts
            .generics
            .parameters
            .get(&declaration.definition)
            .is_some_and(|p| !p.is_empty())
    {
        return Err("three_way_compare result changed actual Ordering identity or parameters");
    }
    Ok(item)
}
fn ordering(facts: &Declarations, output: TypeId) -> Result<[VariantId; 3], &'static str> {
    let item = ordering_identity(facts, output)?;
    let definition = facts
        .aggregates
        .get(item)
        .ok_or("Ordering declaration shape is pending")?;
    let variants = facts.members.variants(item);
    let [less, equal, greater] = variants else {
        return Err("Ordering requires exactly three actual variants");
    };
    if definition.kind != aggregate::Kind::Enum
        || definition.repr
            != (aggregate::Repr {
                tag: Some(Primitive::I8),
                ..aggregate::Repr::default()
            })
        || facts.aggregates.variants(definition).len() != 3
    {
        return Err("three_way_compare requires actual fieldless repr(i8) Ordering");
    }
    for (ordinal, id) in variants.iter().enumerate() {
        let member = facts
            .members
            .variants
            .get(id.0 as usize)
            .ok_or("Ordering variant disappeared")?;
        if member.owner != item || member.ordinal as usize != ordinal || member.start != member.end
        {
            return Err("Ordering variant belongs to another owner or has fields");
        }
    }
    Ok([*less, *equal, *greater])
}
impl Catalog {
    pub(crate) fn three_way_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<ThreeWaySelection>, &'static str> {
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
        if kind != SourceIntrinsic::ThreeWayCompare {
            return Ok(None);
        }
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err("imported three_way_compare body/call provenance is not transported");
        }
        crate::sema::header_attributes::charge_source_contract_query(facts, 64)
            .map_err(|_| "three_way_compare exceeds its existing source work budget")?;
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|i| facts.functions.get(*i))
            .ok_or("three_way_compare completed declaration is missing")?;
        if function.signature != original
            || function.node != declaration.origin
            || declaration.parent.is_some()
        {
            return Err("three_way_compare completed source header changed");
        }
        validate(facts, *function)?;
        self.validated_language(LangItem::Copy)?
            .ok_or("three_way_compare requires its completed actual Copy header")?;
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("invalid three_way_compare arguments")?;
        let (Some(Argument::Type(input)), None) = (args.next(), args.next()) else {
            return Err("three_way_compare requires its complete single-type instance");
        };
        if !matches!(facts.types.kind(input), Some(Kind::Primitive(p)) if p.integer().is_some() || p==Primitive::Char)
        {
            return Err("three_way_compare supports only actual integer/char scalar operands");
        }
        let Some(Kind::Function { result: output, .. }) = facts.types.kind(signature) else {
            return Err("three_way_compare instance lost its signature");
        };
        selected_signature(facts, signature, input, output)?;
        Ok(Some(ThreeWaySelection {
            definition,
            signature,
            input,
            output,
            variants: ordering(facts, output)?,
        }))
    }
    pub(crate) fn require_three_way_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &ThreeWaySelection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let Some((declaration, SourceIntrinsic::ThreeWayCompare, _)) =
            self.intrinsic_record(selected.definition)?
        else {
            return Err("three_way_compare lost its actual declaration record");
        };
        let current = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "three_way_compare current source header is unavailable")?;
        if current
            .origin
            .source()
            .is_none_or(|s| s.declaration != declaration.origin)
            || matches!(current.must_use, ContractKnowledge::Unknown)
            || matches!(current.inline, ContractKnowledge::Unknown)
        {
            return Err("three_way_compare requires positive actual source-header completion");
        }
        if ordering(facts, selected.output)? != selected.variants {
            return Err("Ordering actual variant identities changed");
        }
        for (id, expected) in selected
            .variants
            .into_iter()
            .zip(["Less", "Equal", "Greater"])
        {
            let origin = facts.members.variants[id.0 as usize]
                .origin
                .source()
                .ok_or("Ordering requires actual source variants")?;
            let unit = report.package.unit(origin.file);
            let crate::syntax::ast::Kind::Variant(index) =
                unit.configured.ast.node(origin.node).kind
            else {
                return Err("Ordering variant lost its actual parsed declaration");
            };
            if unit
                .spelling(unit.configured.ast.variants[index as usize].name)
                .map_err(|_| "invalid Ordering variant name")?
                != expected
            {
                return Err("three_way_compare requires actual Less/Equal/Greater source variants");
            }
        }
        Ok(())
    }
}
