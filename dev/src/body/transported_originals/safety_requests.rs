//! Imported lexical facts are checked against the fresh caller and exact Call data.
use super::ImportedOriginals;
use crate::{
    body::ir::{Body, Rvalue},
    sema::{crates::SourceOriginal, model::Declarations},
    support::crates::generics::source as s,
    types::Kind,
};

pub(super) fn check(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    if !view.rows().descriptor().profile().call_safety() {
        return Ok(());
    }
    work.charge(1)?;
    let defining_header = facts
        .function_header(view.definition())?
        .ok_or("source call safety lost its actual installed caller header")?;
    if defining_header.signature != view.signature() || defining_header.body != Some(body.source) {
        return Err("source call safety changed its actual defining body/header");
    }
    let Some(Kind::Function {
        unsafe_: caller_unsafe,
        ..
    }) = facts.types.kind(defining_header.signature)
    else {
        return Err("source call safety caller signature is not a function");
    };
    for request in &view.raw().calls {
        work.charge(1)?;
        view.site(request.site, &mut work.steps)?;
        let value = body
            .values
            .get(request.value as usize)
            .ok_or("source call safety has no actual reconstructed value")?;
        let Rvalue::Call {
            signature, callee, ..
        } = value.kind
        else {
            return Err("source call safety belongs to another operation kind");
        };
        if signature != view.ty(request.signature, &mut work.steps)? {
            return Err("source call safety changed its fresh opened signature");
        }
        let Some(Kind::Function { unsafe_, .. }) = facts.types.kind(signature) else {
            return Err("source call safety lost its actual function signature");
        };
        let callee = body
            .values
            .get(callee.0 as usize)
            .ok_or("source call safety has no actual function operand")?;
        if !matches!(callee.kind, Rvalue::Function(definition) if definition == view.function(request.target, &mut work.steps)?)
        {
            return Err("source call safety changed its actual selected declaration");
        }
        match (unsafe_, request.safety) {
            (false, s::CallSafety::Safe) | (true, s::CallSafety::ExplicitUnsafeBlock) => {}
            (true, s::CallSafety::UnsafeFunction) if caller_unsafe => {}
            _ => return Err("source call safety has no matching actual caller obligation"),
        }
    }
    work.charge(0)
}

#[cfg(test)]
#[path = "../../../tests/invariants/discharged_bodies.rs"]
mod tests;
