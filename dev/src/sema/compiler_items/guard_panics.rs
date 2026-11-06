//! Ordinary panic declarations are distinct from compiler-generated guard calls.
use super::Catalog;
use crate::{
    resolve::{
        bindings::Declaration, body_origin::DeclarationOrigin, expansion::Report, package::Error,
    },
    sema::model::Declarations,
    support::bindings::LangItem,
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
        completed[slot] = super::headers::functions::complete(report, facts, catalog, role)?;
    }
    Ok(completed)
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
    super::headers::functions::current(
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
