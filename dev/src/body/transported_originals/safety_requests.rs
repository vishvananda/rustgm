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
    let (caller_policy, caller_unsafe) = caller_context(facts, view, body)?;
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
        let literal = matches!(callee.kind, Rvalue::Function(definition) if definition == target);
        let typed = view.rows().descriptor().profile().declaration_policies()
            && matches!(facts.types.kind(callee.ty), Some(Kind::FunctionItem { item, .. }) if item.0 == target.0);
        if !(literal || typed) {
            return Err("source call safety changed its actual selected declaration");
        }
        if view.rows().function_applications[request.target.0 as usize]
            .declaration
            .artifact
            .0
            != 0
        {
            let Some(Kind::FunctionItem {
                signature: declared,
                ..
            }) = facts.types.kind(callee.ty)
            else {
                return Err("source dependency call has no applied signature");
            };
            dependency_opening(facts, view, request, (declared, signature), work)?;
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

fn dependency_opening(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    request: &s::CallRequest,
    (declared, opened): (crate::types::TypeId, crate::types::TypeId),
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    use std::cell::Cell;
    let steps = Cell::new(0_usize);
    let bytes = Cell::new(0_usize);
    let base = work.retained_bytes();
    let actual = crate::types::callable::late_result::rigid_opening_regions(
        &facts.types,
        declared,
        opened,
        &|steps_used, allocated| {
            steps.set(steps.get().saturating_add(steps_used));
            bytes.set(bytes.get().saturating_add(allocated));
            if work.steps.saturating_add(steps.get()) > crate::support::crates::MAX_COMPONENTS
                || base.saturating_add(bytes.get()) > crate::support::crates::MAX_BYTES
            {
                return Err("source dependency call opening exceeds its work/storage budget");
            }
            Ok(())
        },
    )?;
    work.charge(steps.get())?;
    let actual = actual.ok_or("source dependency call changed its applied signature")?;
    let regions = request.opening.map_or(Ok(&[][..]), |opening| {
        opening.regions.slice(&view.raw().openings)
    })?;
    if actual.len() != regions.len() {
        return Err("source dependency call changed its actual late-region count");
    }
    for (actual, &recorded) in actual.into_iter().zip(regions) {
        if let Some(actual) = actual
            && actual != view.region(recorded, &mut work.steps)?
        {
            return Err("source dependency call changed its actual opened region");
        }
    }
    Ok(())
}

type CallerPolicy = Option<(crate::sema::target_features::FeatureSet, bool)>;

fn caller_context(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
) -> Result<(CallerPolicy, bool), &'static str> {
    let caller_policy = if view.rows().descriptor().profile().declaration_policies()
        && view.signature().is_some()
        && view.closure_owner().is_none()
    {
        crate::sema::target_features::imported_policy(facts, view.definition())?
    } else {
        None
    };
    let caller_unsafe = if view.closure_owner().is_some() {
        if body.owner != super::rebuilt_operations::owner(view) || body.source != view.source()? {
            return Err("source call safety changed its actual closure body");
        }
        false
    } else if let Some(signature) = view.signature() {
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
    Ok((caller_policy, caller_unsafe))
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
