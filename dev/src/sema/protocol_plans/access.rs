//! Checked Field steps have one sorted owner and explicit evaluation-order links.
use super::{super::bodies::SourceCallUse, Plan};
use crate::{resolve::unit::NodeKey, syntax::ast::NodeId, types::TypeId};
use std::num::NonZeroU32;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Range {
    pub start: u32,
    pub end: u32,
    pub first: NonZeroU32,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Step {
    pub source_use: SourceCallUse,
    pub node: NodeId,
    pub call: super::super::iteration::Call,
    pub subject: TypeId,
    pub borrow: TypeId,
    pub target: TypeId,
    pub output_view: TypeId,
    pub dereferences: u32,
    pub next: Option<NonZeroU32>,
}
#[derive(Default)]
pub(in crate::sema) struct Rows {
    pub steps: Vec<Step>,
}
const _: [(); 40] = [(); size_of::<Step>()];

pub(crate) fn rows(
    facts: &super::super::model::Declarations,
    plan: Plan,
    owner: NodeKey,
    range: Range,
) -> Result<&[Step], &'static str> {
    plan.source_use.effect_for(facts, owner)?;
    let rows = facts
        .call_contexts()
        .and_then(super::super::bodies::CallContexts::access_steps)
        .ok_or("field plan lacks its actual checked step owner")?;
    let slice = rows
        .get(range.start as usize..range.end as usize)
        .ok_or("field plan has an invalid checked step range")?;
    let first = range.first.get() - 1;
    if slice.is_empty() || first < range.start || first >= range.end {
        return Err("field plan has an invalid first step");
    }
    Ok(slice)
}

pub(crate) fn current(
    facts: &super::super::model::Declarations,
    plan: Plan,
    owner: NodeKey,
    range: Range,
    index: u32,
) -> Result<Step, &'static str> {
    let rows = rows(facts, plan, owner, range)?;
    current_from(plan, range, rows, index)
}

/// An exact call lookup visits one sorted range, never every plan or predecessor.
/// It supplies source data; body/const execution proof remains independently required.
pub(crate) fn lookup(
    report: &crate::resolve::expansion::Report,
    facts: &super::super::model::Declarations,
    plan: Plan,
    owner: NodeKey,
    call: super::super::iteration::Call,
) -> Result<Step, crate::resolve::package::Error> {
    let step = retained(facts, plan, owner, call)?;
    let payments =
        super::super::header_attributes::source_payments(facts.must_use_contracts.as_deref())?;
    let pay = |work, bytes| payments.charge(work, bytes);
    let mut shapes = crate::types::structural::Normalized::<false>::new_charged(
        &facts.types,
        &pay,
        crate::types::structural::Memo::default(),
    );
    check_call(report, facts, plan, step, &mut shapes)?;
    Ok(step)
}

/// Borrow only the already published exact row. This supplies data, never a
/// source header, defining premise, region proof or const execution permission.
pub(crate) fn retained_call(
    facts: &super::super::model::Declarations,
    plan: Plan,
    owner: NodeKey,
    call: super::super::iteration::Call,
) -> Result<bool, crate::resolve::package::Error> {
    use crate::resolve::package::Error;
    if matches!(plan.kind, super::Kind::Receiver { call: selected, .. } if selected == call) {
        let Some(super::super::model::Operation::MethodReceiver { plan: id }) =
            facts.node(plan.node).and_then(|node| node.operation)
        else {
            return Err(Error::unlocated(
                "method access lost its current source plan",
            ));
        };
        if id
            .current(facts, owner, plan.node)
            .map_err(Error::unlocated)?
            != plan
        {
            return Err(Error::unlocated(
                "method access changed its current source owner",
            ));
        }
        check_method(facts, plan, call)?;
        return Ok(false);
    }
    retained(facts, plan, owner, call).map(|_| true)
}

pub(crate) fn retained(
    facts: &super::super::model::Declarations,
    plan: Plan,
    owner: NodeKey,
    call: super::super::iteration::Call,
) -> Result<Step, crate::resolve::package::Error> {
    use crate::resolve::package::Error;
    let (_, _, _, steps) = plan
        .kind
        .access()
        .ok_or_else(|| Error::unlocated("access lookup received another actual protocol"))?;
    let id = match (
        plan.kind,
        facts.node(plan.node).and_then(|node| node.operation),
    ) {
        (
            super::Kind::Field { field, .. },
            Some(super::super::model::Operation::Field {
                field: actual,
                plan: Some(id),
            }),
        ) if field == actual => id,
        (
            super::Kind::Receiver { .. },
            Some(super::super::model::Operation::MethodReceiver { plan: id }),
        ) => id,
        _ => {
            return Err(Error::unlocated(
                "access lookup lost its current source plan",
            ));
        }
    };
    if id
        .current(facts, owner, plan.node)
        .map_err(Error::unlocated)?
        != plan
    {
        return Err(Error::unlocated(
            "access lookup changed its current owner or plan",
        ));
    }
    let entries = rows(facts, plan, owner, steps).map_err(Error::unlocated)?;
    super::super::header_attributes::charge_source_contract_query(
        facts,
        entries
            .len()
            .checked_ilog2()
            .map_or(1, |log| log as usize + 2),
    )?;
    let ordinal = entries
        .binary_search_by_key(&call.callable.0, |step| step.call.callable.0)
        .map_err(|_| {
            Error::unlocated("field call is absent from its actual checked source plan")
        })?;
    let index = steps
        .start
        .checked_add(
            u32::try_from(ordinal).map_err(|_| Error::unlocated("field lookup index overflow"))?,
        )
        .ok_or_else(|| Error::unlocated("field lookup range overflow"))?;
    let step = current(facts, plan, owner, steps, index).map_err(Error::unlocated)?;
    if step.call != call {
        return Err(Error::unlocated(
            "field call changed its complete actual signature",
        ));
    }
    Ok(step)
}

fn current_from(plan: Plan, range: Range, rows: &[Step], index: u32) -> Result<Step, &'static str> {
    if range
        .end
        .checked_sub(range.start)
        .map(|count| count as usize)
        != Some(rows.len())
    {
        return Err("field steps changed their actual checked range length");
    }
    let index = index
        .checked_sub(range.start)
        .ok_or("field cursor precedes its actual step range")?;
    let step = rows
        .get(index as usize)
        .copied()
        .ok_or("field cursor exceeds its actual step range")?;
    if step.node != plan.node.node || step.source_use != plan.source_use {
        return Err("field step belongs to another actual source or body");
    }
    if step
        .next
        .is_some_and(|next| next.get() <= range.start || next.get() > range.end)
    {
        return Err("field step link leaves its actual checked range");
    }
    Ok(step)
}

pub(crate) fn next(
    facts: &super::super::model::Declarations,
    plan: Plan,
    owner: NodeKey,
    range: Range,
    calls: &mut super::Calls,
) -> Result<Option<Step>, &'static str> {
    let entries = rows(facts, plan, owner, range)?;
    next_from(plan, range, entries, calls)
}

fn next_from(
    plan: Plan,
    range: Range,
    rows: &[Step],
    calls: &mut super::Calls,
) -> Result<Option<Step>, &'static str> {
    let remaining = calls
        .remaining
        .checked_sub(u32::from(matches!(plan.kind, super::Kind::Receiver { .. })))
        .ok_or("access call cursor exceeded its final call")?;
    if remaining == 0 {
        return Ok(None);
    }
    let step = current_from(plan, range, rows, calls.next)?;
    if (remaining == 1) != step.next.is_none() {
        return Err("field evaluation links do not visit its complete actual range");
    }
    calls.next = step.next.map_or(0, |next| next.get() - 1);
    calls.remaining -= 1;
    Ok(Some(step))
}

/// Whole-chain authentication is linear and precedes genuine CFG construction.
pub(crate) fn check(
    report: &crate::resolve::expansion::Report,
    facts: &super::super::model::Declarations,
    owner: NodeKey,
    plan: Plan,
) -> Result<(), crate::resolve::package::Error> {
    let error = |message| super::super::model::Declarations::error(report, plan.node, message);
    let Some((_, _, _, steps)) = plan.kind.access() else {
        return Err(error("field validation received another actual protocol"));
    };
    let entries = rows(facts, plan, owner, steps).map_err(error)?;
    check_rows(report, facts, owner, plan, entries)
}

pub(in crate::sema) fn check_rows(
    report: &crate::resolve::expansion::Report,
    facts: &super::super::model::Declarations,
    owner: NodeKey,
    plan: Plan,
    entries: &[Step],
) -> Result<(), crate::resolve::package::Error> {
    let error = |message| super::super::model::Declarations::error(report, plan.node, message);
    let (operand, receiver, dereferences, steps) = plan
        .kind
        .access()
        .ok_or_else(|| error("access validation received another actual protocol"))?;
    let syntax = report
        .package
        .unit(plan.node.file)
        .configured
        .ast
        .node(plan.node.node)
        .kind;
    let valid = match plan.kind {
        super::Kind::Field { field, .. } => {
            matches!(syntax, crate::syntax::ast::Kind::Field { value, .. } if value == operand)
                && matches!(facts.node(plan.node).and_then(|node| node.operation), Some(super::super::model::Operation::Field { field: actual, .. }) if actual == field)
        }
        super::Kind::Receiver { call, .. } => {
            matches!(syntax,
                crate::syntax::ast::Kind::MethodCall { receiver, .. }
                | crate::syntax::ast::Kind::GenericMethodCall { receiver, .. }
                if receiver == operand)
                && facts
                    .methods
                    .calls
                    .get(&plan.node)
                    .is_some_and(|selected| selected.callable == call.callable)
        }
        _ => false,
    };
    if plan.node.file != owner.file || !valid {
        return Err(error(
            "access plan changed its actual source operation or member",
        ));
    }
    plan.source_use.effect_for(facts, owner).map_err(error)?;
    super::super::header_attributes::charge_source_contract_query(
        facts,
        entries.len().saturating_mul(3),
    )?;
    let reference_work = entries.iter().fold(dereferences as usize, |work, step| {
        work.saturating_add(step.dereferences as usize)
    });
    super::super::header_attributes::charge_source_contract_query(facts, reference_work)?;
    for pair in entries.windows(2) {
        if pair[0].call.callable.0 >= pair[1].call.callable.0 {
            return Err(error(
                "field step owner lost its strict complete-callable ordering",
            ));
        }
    }
    let mut ty = facts
        .node(NodeKey {
            node: operand,
            ..plan.node
        })
        .and_then(|node| node.ty)
        .ok_or_else(|| error("field plan lost its actual original operand type"))?;
    let payments =
        super::super::header_attributes::source_payments(facts.must_use_contracts.as_deref())?;
    let pay = |work, bytes| payments.charge(work, bytes);
    let mut shapes = crate::types::structural::Normalized::<false>::new_charged(
        &facts.types,
        &pay,
        crate::types::structural::Memo::default(),
    );
    let mut calls = plan.calls().map_err(error)?;
    while let Some(step) = next_from(plan, steps, entries, &mut calls).map_err(error)? {
        ty = references(facts, ty, step.dereferences).map_err(error)?;
        if ty != step.subject {
            return Err(error("field step changed its actual current subject"));
        }
        check_call(report, facts, plan, step, &mut shapes)?;
        ty = step.target;
    }
    if references(facts, ty, dereferences).map_err(error)? != receiver {
        return Err(error("field step chain changed its actual final receiver"));
    }
    if let super::Kind::Receiver { call, .. } = plan.kind {
        check_method(facts, plan, call)?;
    }
    Ok(())
}

pub(crate) fn references(
    facts: &super::super::model::Declarations,
    mut ty: TypeId,
    count: u32,
) -> Result<TypeId, &'static str> {
    if count > 1_000_000 {
        return Err("field reference depth exceeds its existing work limit");
    }
    for _ in 0..count {
        let Some(crate::types::Kind::Reference { referent, .. }) = facts.types.kind(ty) else {
            return Err("field step lost its actual built-in reference prefix");
        };
        ty = referent;
    }
    Ok(ty)
}

fn check_call(
    report: &crate::resolve::expansion::Report,
    facts: &super::super::model::Declarations,
    plan: Plan,
    step: Step,
    shapes: &mut crate::types::structural::Normalized<
        '_,
        false,
        crate::types::structural::PaidCost<'_>,
    >,
) -> Result<(), crate::resolve::package::Error> {
    use crate::types::{Kind, callable::Convention};
    let error = |message| super::super::model::Declarations::error(report, plan.node, message);
    let Some(Kind::Reference {
        mutable, referent, ..
    }) = facts.types.kind(step.borrow)
    else {
        return Err(error("field step lacks its actual operand borrow"));
    };
    if referent != step.subject {
        return Err(error("field borrow changed its actual place identity"));
    }
    check_opening(report, facts, plan, step, mutable)?;
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
        ..
    }) = facts.types.kind(step.call.signature)
    else {
        return Err(error("field step lost its actual safe Rust call signature"));
    };
    let Some([receiver]) = facts.types.list(parameters) else {
        return Err(error("field step changed its actual call parameters"));
    };
    let Some(Kind::Reference {
        mutable: expected_mutable,
        region,
        ..
    }) = facts.types.kind(*receiver)
    else {
        return Err(error("field step lost its actual expected receiver"));
    };
    let Some(Kind::Reference {
        mutable: output_mutable,
        region: output_region,
        referent: _,
    }) = facts.types.kind(result)
    else {
        return Err(error("field step lost its actual reference result"));
    };
    let Some(Kind::Reference {
        mutable: view_mutable,
        region: view_region,
        referent: view_target,
    }) = facts.types.kind(step.output_view)
    else {
        return Err(error("field output view lost its actual reference type"));
    };
    if view_mutable != output_mutable
        || view_region != output_region
        || view_target != step.target
        || !shapes
            .matches_charged(result, step.output_view)
            .map_err(error)?
    {
        return Err(error(
            "field output view changed its actual result shape, region or target",
        ));
    }
    if output_mutable != mutable || output_region != region || expected_mutable != mutable {
        return Err(error(
            "field step changed its actual result target, borrow mutability or lifetime",
        ));
    }
    Ok(())
}

fn check_opening(
    report: &crate::resolve::expansion::Report,
    facts: &super::super::model::Declarations,
    plan: Plan,
    step: Step,
    mutable: bool,
) -> Result<(), crate::resolve::package::Error> {
    use crate::types::Kind;
    let error = |message| super::super::model::Declarations::error(report, plan.node, message);
    let selected =
        super::super::compiler_items::dereferencing::query(report, facts, plan.node, mutable)?;
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(step.call.callable)
    else {
        return Err(error(
            "field step lost its complete actual selected callable",
        ));
    };
    if item.0 != selected.method.0 {
        return Err(error("field step changed its actual selected method"));
    }
    let payments =
        super::super::header_attributes::source_payments(facts.must_use_contracts.as_deref())?;
    if !crate::types::callable::late_result::signature_opening(
        &facts.types,
        signature,
        step.call.signature,
        &|work, bytes| payments.charge(work, bytes),
    )
    .map_err(error)?
    {
        return Err(super::super::model::Declarations::error(
            report,
            plan.node,
            format!(
                "field step changed its actual call opening: declared {signature:?} {:?}, opened {:?} {:?}",
                facts.types.kind(signature),
                step.call.signature,
                facts.types.kind(step.call.signature)
            ),
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/access_guards.rs"]
mod tests;

fn check_method(
    facts: &super::super::model::Declarations,
    plan: Plan,
    call: super::super::iteration::Call,
) -> Result<(), crate::resolve::package::Error> {
    let fail = || {
        crate::resolve::package::Error::unlocated(
            "method access changed its actual selected call or signature",
        )
    };
    let selected = facts.methods.calls.get(&plan.node).ok_or_else(fail)?;
    let Some(crate::types::Kind::FunctionItem { item, .. }) = facts.types.kind(call.callable)
    else {
        return Err(fail());
    };
    let Some(crate::types::Kind::Function { parameters, .. }) = facts.types.kind(call.signature)
    else {
        return Err(fail());
    };
    if item.0 != selected.method.0
        || selected.callable != call.callable
        || facts
            .types
            .list(parameters)
            .and_then(|parameters| parameters.first())
            .copied()
            != Some(selected.receiver)
    {
        return Err(fail());
    }
    Ok(())
}
