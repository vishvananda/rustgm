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
    let caller_policy = if view.rows().descriptor().profile().declaration_policies()
        && view.signature().is_some()
    {
        crate::sema::target_features::imported_policy(facts, view.definition())?
    } else {
        None
    };
    let caller_unsafe = if let Some(signature) = view.signature() {
        let defining_header = facts
            .function_header(view.definition())?
            .ok_or("source call safety lost its actual installed caller header")?;
        if defining_header.signature != signature || defining_header.body != Some(body.source) {
            return Err("source call safety changed its actual defining body/header");
        }
        let Some(Kind::Function { unsafe_, .. }) = facts.types.kind(signature) else {
            return Err("source call safety caller signature is not a function");
        };
        unsafe_ && !matches!(caller_policy, Some((_, false)))
    } else {
        if body.owner != super::rebuilt_operations::owner(view) || body.source != view.source()? {
            return Err("source call safety changed its actual initializer body");
        }
        false
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
        let target = view.function(request.target, &mut work.steps)?;
        if !matches!(callee.kind, Rvalue::Function(definition) if definition == target) {
            return Err("source call safety changed its actual selected declaration");
        }
        if !safe_target_call(facts, caller_policy, target, unsafe_, request.safety)? {
            matching(unsafe_, request.safety, caller_unsafe)?;
        }
    }
    // Version thirteen: a call through a fn pointer carries its own context.
    for (index, value) in view.raw().values.iter().enumerate() {
        let s::ValueKind::PointerCall {
            signature, safety, ..
        } = value.kind
        else {
            continue;
        };
        work.charge(1)?;
        let Some(Rvalue::Call {
            signature: rebuilt, ..
        }) = body.values.get(index).map(|value| value.kind)
        else {
            return Err("source call safety belongs to another operation kind");
        };
        if rebuilt != view.ty(signature, &mut work.steps)? {
            return Err("source call safety changed its fresh opened signature");
        }
        let Some(Kind::Function { unsafe_, .. }) = facts.types.kind(rebuilt) else {
            return Err("source call safety lost its actual function signature");
        };
        matching(unsafe_, safety, caller_unsafe)?;
    }
    // Inline assembly's recorded context follows the rule for unsafe calls.
    for instruction in view
        .raw()
        .assembly
        .iter()
        .flat_map(|rows| &rows.instructions)
    {
        work.charge(1)?;
        match instruction.safety {
            s::CallSafety::ExplicitUnsafeBlock => {}
            s::CallSafety::UnsafeFunction if caller_unsafe => {}
            _ => return Err("source inline assembly has no matching unsafe context"),
        }
    }
    work.charge(0)
}
/// A recorded context discharges exactly its callee's unsafety.
fn matching(unsafe_: bool, safety: s::CallSafety, caller_unsafe: bool) -> Result<(), &'static str> {
    match (unsafe_, safety) {
        (false, s::CallSafety::Safe) | (true, s::CallSafety::ExplicitUnsafeBlock) => Ok(()),
        (true, s::CallSafety::UnsafeFunction) if caller_unsafe => Ok(()),
        _ => Err("source call safety has no matching actual caller obligation"),
    }
}

fn safe_target_call(
    facts: &Declarations,
    caller_policy: Option<(crate::sema::target_features::FeatureSet, bool)>,
    target: crate::resolve::model::DefId,
    unsafe_: bool,
    safety: s::CallSafety,
) -> Result<bool, &'static str> {
    if !unsafe_ || safety != s::CallSafety::Safe {
        return Ok(false);
    }
    let Some((available, _)) = caller_policy else {
        return Ok(false);
    };
    Ok(
        crate::sema::target_features::imported_policy(facts, target)?.is_some_and(
            |(required, declared_unsafe)| !declared_unsafe && available.covers(required),
        ),
    )
}

#[cfg(test)]
#[path = "../../../tests/invariants/discharged_bodies.rs"]
mod tests;
