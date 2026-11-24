//! A real `ConstParamTy` header supplies no implementation or const-value evidence.
use super::{Catalog, Declaration, Declarations, Error, LangItem, Report, RoleSet};
use crate::{
    resolve::model::DefId,
    sema::compiler_items::headers::{Flags, Trait},
    support::constness::Constness,
};

pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    header: &Trait<'_>,
    complete: &mut RoleSet,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    crate::sema::header_attributes::charge_source_contract_query(
        facts,
        16_usize.saturating_add(header.inputs.len().saturating_mul(2)),
    )?;
    let structural = catalog
        .language(LangItem::StructuralPartialEq)
        .ok_or_else(|| error("ConstParamTy requires actual StructuralPartialEq"))?;
    let eq = report
        .resolution
        .source_bindings
        .as_deref()
        .and_then(crate::resolve::bindings::Reservations::eq_declaration)
        .ok_or_else(|| error("ConstParamTy requires actual Eq diagnostic identity"))?;
    matching_supertraits(facts, header, structural.definition, eq.definition).map_err(error)?;
    let marker = Trait::source(report, facts, structural).map_err(error)?;
    structural_header(facts, &marker).map_err(error)?;
    Trait::source(report, facts, eq)
        .map_err(error)?
        .validate_self(&facts.types)
        .map_err(error)?;
    complete.insert(LangItem::StructuralPartialEq);
    Ok(())
}

pub(super) fn imported(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    steps: &mut usize,
    complete: &mut RoleSet,
) -> Result<(), &'static str> {
    let structural = catalog.role(LangItem::StructuralPartialEq)?;
    if let Some(role) = structural {
        structural_header(facts, &Trait::current(report, facts, role, steps)?)?;
        complete.insert(LangItem::StructuralPartialEq);
    }
    let Some(declaration) = catalog.role(LangItem::ConstParamTy)? else {
        return Ok(());
    };
    let structural = structural.ok_or("ConstParamTy requires actual StructuralPartialEq")?;
    let eq = catalog
        .imported_eq(steps)?
        .ok_or("ConstParamTy requires actual Eq diagnostic identity")?;
    let header = Trait::current(report, facts, declaration, steps)?;
    header.profile(&facts.types, ordinary(), 0)?;
    *steps = steps.saturating_add(header.inputs.len().saturating_mul(2));
    matching_supertraits(facts, &header, structural.definition, eq.definition)?;
    Trait::current(report, facts, eq, steps)?.validate_self(&facts.types)?;
    complete.insert(LangItem::ConstParamTy);
    Ok(())
}

fn matching_supertraits(
    facts: &Declarations,
    header: &Trait<'_>,
    structural: DefId,
    eq: DefId,
) -> Result<(), &'static str> {
    if header.written.supertraits != 2
        || !header.members.is_empty()
        || !header.requires(&facts.types, structural, None, Constness::Ordinary)
        || !header.requires(&facts.types, eq, None, Constness::Ordinary)
    {
        return Err("ConstParamTy requires its exact StructuralPartialEq and Eq supertraits");
    }
    Ok(())
}

const fn ordinary() -> Flags {
    Flags {
        unsafe_: false,
        constant: false,
        auto_: false,
    }
}

pub(super) fn structural_header(
    facts: &Declarations,
    marker: &Trait<'_>,
) -> Result<(), &'static str> {
    marker.profile(&facts.types, ordinary(), 0)?;
    if marker.written.supertraits != 0 || !marker.members.is_empty() {
        return Err("source StructuralPartialEq requires its actual empty trait");
    }
    Ok(())
}
