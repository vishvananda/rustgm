//! Actual builtin headers and structured assumptions precede the all-values relation.
mod options;
use super::{Catalog, headers::Trait};
use crate::{
    resolve::{bindings::Declaration, expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{
        GenericArgs, ItemId, Kind,
        arguments::{Argument, ConstKind},
        parameters::ParameterKind,
    },
};
pub(in crate::sema) use options::decode;

pub(super) fn validate_trait(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    header: &Trait<'_>,
) -> Result<(), Error> {
    use crate::resolve::bindings::HeaderAttribute;
    let error = |message| Declarations::error(report, declaration.origin, message);
    crate::sema::header_attributes::charge_source_contract_query(facts, 32)?;
    if !report
        .resolution
        .source_bindings
        .as_deref()
        .is_some_and(|claims| claims.denies_implementation(declaration.definition))
        || !header.attributes.contains(HeaderAttribute::Coinductive)
        || !header.attributes.contains(HeaderAttribute::DynIncompatible)
    {
        return Err(error(
            "transmute_trait requires its actual deny-implementation, coinductive and dyn restrictions",
        ));
    }
    if header.written.parameters != 2 || header.written.supertraits != 0 {
        return Err(error(
            "transmute_trait requires exactly Src and const Assume formals",
        ));
    }
    let [source, assumptions] = header.parameters else {
        return Err(error("transmute_trait lost its actual generic formals"));
    };
    let Argument::Type(ty) = source.argument else {
        return Err(error(
            "transmute_trait Src must be an actual type parameter",
        ));
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
        return Err(error("transmute_trait Src lost its rigid type identity"));
    };
    if !facts.types.parameter(id).is_some_and(|formal| {
        formal.owner == ItemId(declaration.definition.0)
            && formal.index == 0
            && formal.kind == ParameterKind::Type { sized: false }
    }) {
        return Err(error(
            "transmute_trait Src requires its own unsized-capable formal",
        ));
    }
    let Argument::Const(id) = assumptions.argument else {
        return Err(error(
            "transmute_trait assumptions must be an actual const parameter",
        ));
    };
    let Some(ConstKind::Parameter(id)) = facts.types.const_kind(id) else {
        return Err(error(
            "transmute_trait assumptions lost their rigid identity",
        ));
    };
    let opts = catalog
        .language(LangItem::TransmuteOpts)
        .ok_or_else(|| error("transmute_trait requires its actual transmute_opts declaration"))?;
    let Some(formal) = facts.types.parameter(id) else {
        return Err(error(
            "transmute_trait assumptions lack their actual declaration",
        ));
    };
    if formal.owner != ItemId(declaration.definition.0)
        || formal.index != 1
        || !matches!(formal.kind, ParameterKind::Const { ty }
            if matches!(facts.types.kind(ty), Some(Kind::Adt(item, GenericArgs::EMPTY))
                if item.0 == opts.definition.0))
    {
        return Err(error(
            "transmute_trait assumptions must have the actual transmute_opts type",
        ));
    }
    Ok(())
}

pub(in crate::sema) fn complete_options(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    let Some(declaration) = catalog.language(LangItem::TransmuteOpts) else {
        return Ok(());
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    catalog.require_provider_report(report).map_err(error)?;
    crate::sema::header_attributes::charge_source_contract_query(facts, 64)?;
    options::slots(facts, declaration).map_err(error)?;
    super::roles::complete_nominal(facts, LangItem::TransmuteOpts, declaration)
}

#[cfg(test)]
#[path = "../../../tests/invariants/assumption_headers.rs"]
mod tests;
