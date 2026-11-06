//! Ordinary panic declarations are distinct from compiler-generated guard calls.
use super::Catalog;
use crate::{
    resolve::{
        bindings::Declaration, body_origin::DeclarationOrigin, expansion::Report, model::DefKind,
        package::Error,
    },
    sema::model::Declarations,
    support::bindings::LangItem,
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::Kind,
};

/// One protocol family over existing language identities; no separate authority.
pub(in crate::sema) const ROLES: [LangItem; 28] = [
    LangItem::PanicBoundsCheck,
    LangItem::PanicMisalignedPointerDereference,
    LangItem::PanicCannotUnwind,
    LangItem::PanicInCleanup,
    LangItem::PanicNullPointerDereference,
    LangItem::PanicInvalidEnumConstruction,
    LangItem::PanicAddOverflow,
    LangItem::PanicSubOverflow,
    LangItem::PanicMulOverflow,
    LangItem::PanicDivOverflow,
    LangItem::PanicRemOverflow,
    LangItem::PanicNegOverflow,
    LangItem::PanicShrOverflow,
    LangItem::PanicShlOverflow,
    LangItem::PanicDivZero,
    LangItem::PanicRemZero,
    LangItem::PanicCoroutineResumed,
    LangItem::PanicAsyncFnResumed,
    LangItem::PanicAsyncGenFnResumed,
    LangItem::PanicGenFnNone,
    LangItem::PanicCoroutineResumedPanic,
    LangItem::PanicAsyncFnResumedPanic,
    LangItem::PanicAsyncGenFnResumedPanic,
    LangItem::PanicGenFnNonePanic,
    LangItem::PanicCoroutineResumedDrop,
    LangItem::PanicAsyncFnResumedDrop,
    LangItem::PanicAsyncGenFnResumedDrop,
    LangItem::PanicGenFnNoneDrop,
];

pub(super) fn complete(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
) -> Result<[Option<LangItem>; 28], Error> {
    let mut completed = [None; 28];
    for (slot, role) in ROLES.into_iter().enumerate() {
        let Some(declaration) = catalog.language(role) else {
            continue;
        };
        let error = |message| Declarations::error(report, declaration.origin, message);
        crate::sema::header_attributes::charge_source_contract_query(facts, 4)?;
        catalog.require_provider_report(report).map_err(error)?;
        let actual = report
            .resolution
            .definitions
            .get(declaration.definition.0 as usize)
            .ok_or_else(|| error("source panic hook lost its declaration identity"))?;
        if actual.kind != DefKind::Function || actual.node != Some(declaration.origin) {
            return Err(error(
                "source panic hook changed its actual function declaration",
            ));
        }
        let header = current(facts, declaration).map_err(error)?;
        let ast = &report.package.unit(declaration.origin.file).configured.ast;
        if !matches!(ast.node(declaration.origin.node).kind,
            Syntax::Item(index) if matches!(ast.items[index as usize].kind,
                ItemKind::Function { body: Some(body), .. }
                    if header.body.and_then(crate::resolve::body_origin::BodyOrigin::source)
                        == Some(crate::resolve::unit::NodeKey { file: declaration.origin.file, node: body })))
        {
            return Err(error(
                "source panic hook requires its actual defined function",
            ));
        }
        completed[slot] = Some(role);
    }
    Ok(completed)
}

fn current(
    facts: &Declarations,
    declaration: Declaration,
) -> Result<crate::sema::function_headers::FunctionHeader<'_>, &'static str> {
    let header = facts
        .function_header(declaration.definition)?
        .ok_or("source panic hook lacks its completed function header")?;
    if declaration.parent.is_some()
        || header.declaration.source() != Some(declaration.origin)
        || header.body.is_none()
    {
        return Err("source panic hook requires its actual defined free function");
    }
    let Some(Kind::FunctionItem {
        item,
        signature,
        arguments,
    }) = facts
        .definitions
        .get(declaration.definition.0 as usize)
        .copied()
        .flatten()
        .and_then(|ty| facts.types.kind(ty))
    else {
        return Err("source panic hook lost its canonical function item");
    };
    if item.0 != declaration.definition.0
        || signature != header.signature
        || !matches!(facts.types.kind(signature), Some(Kind::Function { .. }))
        || facts.types.arguments(arguments).is_none()
    {
        return Err("source panic hook changed its actual canonical function header");
    }
    Ok(header)
}

/// A reached compiler guard cannot silently substitute the course abort for an
/// installed source hook. Ordinary calls still use the checked authored body.
pub(in crate::sema) fn require_guard(
    facts: &Declarations,
    role: LangItem,
) -> Result<(), &'static str> {
    if installed(facts, role)? {
        return Err(
            "compiler assertion requires checked source panic signature, caller and runtime routing",
        );
    }
    Ok(())
}

/// The current generated future combines completed/poisoned states. It cannot
/// select a source panic cause by guessing from that merged invalid-state edge.
pub(in crate::sema) fn require_future_guard(facts: &Declarations) -> Result<(), &'static str> {
    for role in [
        LangItem::PanicAsyncFnResumed,
        LangItem::PanicAsyncFnResumedPanic,
    ] {
        if installed(facts, role)? {
            return Err(
                "generated future invalid-state branch requires distinct checked source panic causes",
            );
        }
    }
    Ok(())
}

fn installed(facts: &Declarations, role: LangItem) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    if !ROLES.contains(&role) {
        return Err("compiler assertion requested a different language role");
    }
    let Some(actual) = catalog.validated_role(role)? else {
        return Ok(false);
    };
    let DeclarationOrigin::Source(origin) = actual.origin else {
        return Err("imported compiler assertion requires authenticated panic operation transport");
    };
    crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
        .map_err(|_| "source compiler assertion lacks or exhausted its completed producer budget")?
        .charge(4, 0)?;
    current(
        facts,
        Declaration {
            definition: actual.definition,
            parent: actual.parent,
            origin,
        },
    )?;
    Ok(true)
}

#[cfg(test)]
#[path = "../../../tests/invariants/assertion_guards.rs"]
mod tests;
