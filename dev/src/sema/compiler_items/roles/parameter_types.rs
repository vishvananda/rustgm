//! A real `ConstParamTy` header supplies no implementation or const-value evidence.
use super::{Catalog, Declaration, Declarations, Error, LangItem, Report, RoleSet};
use crate::{
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
    if header.written.supertraits != 2
        || !header.members.is_empty()
        || !header.requires(
            &facts.types,
            structural.definition,
            None,
            Constness::Ordinary,
        )
        || !header.requires(&facts.types, eq.definition, None, Constness::Ordinary)
    {
        return Err(error(
            "ConstParamTy requires its exact StructuralPartialEq and Eq supertraits",
        ));
    }
    let marker = Trait::source(report, facts, structural).map_err(error)?;
    marker
        .profile(
            &facts.types,
            Flags {
                unsafe_: false,
                constant: false,
                auto_: false,
            },
            0,
        )
        .map_err(error)?;
    if marker.written.supertraits != 0 || !marker.members.is_empty() {
        return Err(error(
            "ConstParamTy requires its actual empty StructuralPartialEq trait",
        ));
    }
    Trait::source(report, facts, eq)
        .map_err(error)?
        .validate_self(&facts.types)
        .map_err(error)?;
    complete.insert(LangItem::StructuralPartialEq);
    Ok(())
}
