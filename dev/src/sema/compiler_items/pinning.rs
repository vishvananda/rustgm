//! Pin construction retains the real inherent source body and ordinary call checks.
#[cfg(test)]
use super::headers::functions::declaration_signature;
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
};
#[cfg(test)]
#[path = "../../../tests/invariants/pin_headers.rs"]
mod tests;

pub(in crate::sema) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(declaration) =
        super::headers::functions::inherent(report, facts, LangItem::PinNewUnchecked)?
    else {
        return Ok(());
    };
    super::roles::complete_constructor(facts, LangItem::PinNewUnchecked, declaration)
}
