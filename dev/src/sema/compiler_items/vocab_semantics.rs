//! Declaration recognition cannot silently grant an unimplemented compiler builtin.
use crate::{
    resolve::{bindings::Reservations, expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
};

pub(super) fn require_supported(report: &Report, reserved: &Reservations) -> Result<(), Error> {
    for role in LangItem::DECLARATIONS {
        if matches!(
            role,
            LangItem::UseCloned | LangItem::FnPtrTrait | LangItem::FnPtrAddr
        ) {
            // Reserve the actual ordinary trait, then require its canonical header
            // completion in roles::validate. This supplies no implementation proof
            // or implicit use/clone operation; those have separate typed consumers.
            continue;
        }
        if matches!(role, LangItem::Pin | LangItem::PinNewUnchecked) {
            // Actual nominal/member completion is separate from pinned receivers,
            // implicit pin construction and async state-machine permission.
            continue;
        }
        if matches!(
            role,
            LangItem::Deref | LangItem::DerefMut | LangItem::DerefPure | LangItem::DerefTarget
        ) {
            // Canonical source owners still complete below; dereference use
            // requires ordinary trait selection and the actual method contract.
            continue;
        }
        if matches!(role, LangItem::CompilerMove | LangItem::CompilerCopy) {
            // These hooks keep their actual ordinary bodies. Current codegen emits no
            // profiling annotations; an invoked hook must still execute its source body.
            continue;
        }
        if let Some(declaration) = reserved.language(role) {
            return Err(Declarations::error(
                report,
                declaration.origin,
                "source compiler language item semantic contract is not implemented",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/vocab_claims.rs"]
mod tests;
