//! Declaration recognition cannot silently grant an unimplemented compiler builtin.
use crate::{
    resolve::{bindings::Reservations, expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
};

pub(super) fn require_supported(report: &Report, reserved: &Reservations) -> Result<(), Error> {
    for role in LangItem::DECLARATIONS {
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
