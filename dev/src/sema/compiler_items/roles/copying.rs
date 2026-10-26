//! Copying roles validate declared contracts; no type gains a Copy permission here.
use super::super::headers::{Method, Trait, clone_method};
use super::{Catalog, Declaration, Declarations, Error, LangItem, Report, TypeId};
use crate::support::constness::Constness;

pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    role: LangItem,
    header: &Trait<'_>,
    complete: &mut u32,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let expected = match role {
        LangItem::Copy => LangItem::Clone,
        LangItem::Clone => LangItem::Sized,
        LangItem::LegacyReceiver => LangItem::PointeeSized,
        _ => return Err(error("invalid copying role")),
    };
    require_header_super(facts, catalog, header, expected, Constness::Ordinary).map_err(error)?;
    let members = header.members;
    if role != LangItem::Clone {
        if !members.is_empty() {
            return Err(error("source copying marker must not declare members"));
        }
        return Ok(());
    }
    let method = catalog
        .language(LangItem::CloneFn)
        .ok_or_else(|| error("source Clone requires its actual clone_fn declaration"))?;
    let function = facts
        .declaration_header(method.definition)
        .map_err(error)?
        .ok_or_else(|| error("source clone_fn signature is not prepared"))?;
    if function.declaration != method.origin.into() {
        return Err(error("source clone_fn belongs to another declaration"));
    }
    clone_method(
        &facts.types,
        header,
        &Method {
            definition: method.definition,
            parent: method.parent,
            signature: function.signature,
            parameters: facts
                .generics
                .parameters
                .get(&method.definition)
                .map_or(0, Vec::len),
        },
    )
    .map_err(error)?;
    *complete |= 1 << LangItem::CloneFn.index();
    Ok(())
}

pub(super) fn require_super(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    self_type: TypeId,
    expected: LangItem,
    constness: Constness,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let header = super::super::headers::Trait::source(report, facts, declaration).map_err(error)?;
    if header.self_type != self_type {
        return Err(error("source role requires its exact pinned supertrait"));
    }
    require_header_super(facts, catalog, &header, expected, constness).map_err(error)
}

pub(super) fn require_header_super(
    facts: &Declarations,
    catalog: &Catalog,
    header: &Trait<'_>,
    expected: LangItem,
    constness: Constness,
) -> Result<(), &'static str> {
    if header.written.supertraits != 1 {
        return Err("source role requires its exact pinned supertrait");
    }
    let expected = catalog
        .role(expected)?
        .ok_or("source role is missing its prerequisite declaration")?;
    let found = header.requires(
        &facts.types,
        expected.definition,
        catalog
            .role(LangItem::Sized)?
            .map(|declaration| declaration.definition),
        constness,
    );
    if !found {
        return Err("source role is missing its exact typed prerequisite");
    }
    Ok(())
}
