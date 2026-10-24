//! Convert an exact original IR statement into a neutral defining-call query.
use super::invocations::Work;
use crate::{
    body::ir::{Action, Body, Owner, Rvalue, Statement},
    resolve::model::DefId,
    sema::{
        function_context::{Call, DefiningCalls},
        model::Declarations,
    },
    types::Kind,
};

/// One preparation is shared by the readonly and cumulative-budget entrypoints.
pub(super) fn defining(
    facts: &Declarations,
    body: &Body,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let Owner::Function {
        definition,
        signature,
        execution,
    } = body.owner
    else {
        return Ok(false);
    };
    // Only actual const-trait members enter defining preparation. Ordinary
    // source bodies need no body walk, context allocation or validation owner.
    let Some(id) = facts.associated.owner(definition) else {
        return Ok(false);
    };
    let Some(owner) = facts.associated.get(id) else {
        work.charge(1)?;
        return Err("provided defining function has an invalid associated owner");
    };
    if !matches!(
        owner.kind,
        crate::sema::associated::OwnerKind::Trait { constant: true, .. }
    ) {
        return Ok(false);
    }
    let source = body
        .source
        .source()
        .ok_or("imported provided-body defining checks are not implemented")?;
    work.charge(5)?;
    if !matches!(
        facts.function_context(definition, source)?,
        crate::sema::function_context::FunctionContext::ConstTraitDefault { .. }
    ) {
        return Ok(false);
    }
    if let Some(instance) = body.instance {
        work.charge(1)?;
        if execution.is_none()
            || !matches!(facts.types.kind(instance.ty()), Some(Kind::FunctionItem {
                item, signature: actual, ..
            }) if item.0 == definition.0 && actual == signature)
        {
            return Err("provided runtime body lost its actual materialized callable relation");
        }
        // Runtime instances still require the normal paired original/instance
        // proof. Actual CTFE entry rejects this declaration independently.
        return Ok(false);
    }
    work.charge(2)?;
    let header = facts
        .function_header(definition)?
        .ok_or("provided defining function header is pending")?;
    if execution.is_some() || header.signature != signature || header.body != Some(body.source) {
        return Err("provided defining body changed its exact original signature or source");
    }
    Ok(true)
}

pub(super) fn prepare<'a>(
    facts: &'a Declarations,
    body: &Body,
    definition: DefId,
    work: &mut Work,
) -> Result<DefiningCalls<'a>, &'static str> {
    if body.instance.is_some() {
        return Err("specialized abstract const-trait defining calls are not implemented");
    }
    let source = body
        .source
        .source()
        .ok_or("imported const member facts are not implemented")?;
    let Owner::Function {
        signature,
        execution: None,
        ..
    } = body.owner
    else {
        return Err("const member defining body changed its original public callable");
    };
    work.charge(2)?;
    let header = facts
        .function_header(definition)?
        .ok_or("const member defining body requires its completed function header")?;
    if header.signature != signature || header.body != Some(body.source) {
        return Err("const member defining body changed its exact original header");
    }
    DefiningCalls::new(facts, definition, source, |n| work.charge(n))
}

pub(super) fn check(
    context: &DefiningCalls<'_>,
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    work: &mut Work,
) -> Result<(), &'static str> {
    let Action::Evaluate(id) = statement.action else {
        return Err("const member requires an evaluated statement");
    };
    work.charge(1)?;
    let value = body
        .values
        .get(id.0 as usize)
        .ok_or("invalid const member value")?;
    let Rvalue::Call {
        callee, signature, ..
    } = value.kind
    else {
        return Err("const member changed its direct Call operation");
    };
    work.charge(1)?;
    let target = body
        .values
        .get(callee.0 as usize)
        .ok_or("invalid const member callee")?;
    work.charge(1)?;
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(target.ty) else {
        return Err("const member callee lost its full actual function item");
    };
    if !matches!(target.kind, Rvalue::Function(definition) if definition.0 == item.0) {
        return Err("const member callee value changed its exact actual definition");
    }
    work.charge(1)?;
    let site = body
        .source
        .proof_site(statement.source)?
        .source()
        .ok_or("imported const member sites are not implemented")?;
    context.check(
        Call {
            site,
            target: target.ty,
            signature,
            result: value.ty,
        },
        |n| work.charge(n),
    )
}

#[cfg(test)]
#[path = "../../../tests/invariants/defining_members.rs"]
mod tests;
