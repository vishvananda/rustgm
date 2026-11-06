//! Real source Field rows authenticate complete call openings and their one owner.
use super::*;
use crate::{
    resolve::expansion::Report,
    sema::model::Declarations,
    types::{Kind, RegionId, callable::Convention},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn prepared() -> Result<(Report, Declarations, NodeKey, Plan, Vec<Step>)> {
    let source = crate::test_source::fixture("pa31/tests/spec/source-field-deref/lifetime.rs")?;
    let report = crate::test_source::source_provider(&source, "access_guards")?;
    let facts =
        crate::sema::bodies::analyze(&report, crate::sema::declarations::analyze(&report)?)?;
    let owner = facts
        .functions
        .iter()
        .find_map(|function| {
            (report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "probe")
                .then_some(function.body)
                .flatten()
        })
        .ok_or("actual probe body")?;
    let (node, id) = facts.nodes[owner.file.0 as usize]
        .iter()
        .enumerate()
        .find_map(|(index, node)| {
            if let Some(crate::sema::model::Operation::Field { plan: Some(id), .. }) =
                node.operation
            {
                Some((index, id))
            } else {
                None
            }
        })
        .ok_or("actual retained field")?;
    let key = NodeKey {
        file: owner.file,
        node: NodeId::new(u32::try_from(node)?),
    };
    let plan = id.current(&facts, owner, key)?;
    let super::super::Kind::Field { steps, .. } = plan.kind else {
        return Err("actual field plan".into());
    };
    let entries = rows(&facts, plan, owner, steps)?.to_vec();
    Ok((report, facts, owner, plan, entries))
}

fn changed_signature(
    facts: &mut Declarations,
    call: super::super::super::iteration::Call,
    regions: u32,
    unsafe_: bool,
    convention: Convention,
    result: Option<TypeId>,
) -> Result<TypeId> {
    let Some(Kind::Function {
        parameters,
        result: actual,
        ..
    }) = facts.types.kind(call.signature)
    else {
        return Err("actual opened signature".into());
    };
    Ok(facts.types.intern(Kind::Function {
        bound_regions: regions,
        parameters,
        result: result.unwrap_or(actual),
        unsafe_,
        convention,
    })?)
}

#[test]
fn exact_field_opening_rejects_qualifiers_binders_and_inconsistent_slots() -> Result {
    let (report, mut facts, owner, plan, entries) = prepared()?;
    check_rows(&report, &facts, owner, plan, &entries)?;
    let call = entries[0].call;
    for (regions, unsafe_, convention) in [
        (1, false, Convention::RUST),
        (0, true, Convention::RUST),
        (0, false, Convention::C),
    ] {
        let mut changed = entries.clone();
        changed[0].call.signature =
            changed_signature(&mut facts, call, regions, unsafe_, convention, None)?;
        assert!(check_rows(&report, &facts, owner, plan, &changed).is_err());
    }
    let Some(Kind::Function { result, .. }) = facts.types.kind(call.signature) else {
        return Err("signature".into());
    };
    let Some(Kind::Reference {
        mutable, referent, ..
    }) = facts.types.kind(result)
    else {
        return Err("reference result".into());
    };
    let inconsistent = facts.types.intern(Kind::Reference {
        mutable,
        referent,
        region: RegionId(100),
    })?;
    let mut changed = entries.clone();
    changed[0].call.signature = changed_signature(
        &mut facts,
        call,
        0,
        false,
        Convention::RUST,
        Some(inconsistent),
    )?;
    assert!(check_rows(&report, &facts, owner, plan, &changed).is_err());
    let Some(Kind::Adt(item, _)) = facts.types.kind(referent) else {
        return Err("actual nominal target".into());
    };
    let arguments = facts.types.intern_regions(&[RegionId::STATIC])?;
    let target = facts.types.intern(Kind::Adt(item, arguments))?;
    let Some(Kind::Reference { region, .. }) = facts.types.kind(result) else {
        return Err("result".into());
    };
    let changed_result = facts.types.intern(Kind::Reference {
        mutable,
        region,
        referent: target,
    })?;
    changed[0].call.signature = changed_signature(
        &mut facts,
        call,
        0,
        false,
        Convention::RUST,
        Some(changed_result),
    )?;
    assert!(check_rows(&report, &facts, owner, plan, &changed).is_err());
    changed = entries.clone();
    changed[0].output_view = inconsistent;
    assert!(check_rows(&report, &facts, owner, plan, &changed).is_err());
    Ok(())
}

#[test]
fn field_rows_reject_foreign_owner_source_range_and_stale_call() -> Result {
    let (report, facts, owner, plan, entries) = prepared()?;
    check_rows(&report, &facts, owner, plan, &entries)?;
    let wrong_owner = NodeKey {
        node: NodeId::new(u32::MAX),
        ..owner
    };
    assert!(check_rows(&report, &facts, wrong_owner, plan, &entries).is_err());
    assert!(retained(&facts, plan, wrong_owner, entries[0].call).is_err());
    let mut changed = entries.clone();
    changed[0].node = NodeId::new(u32::MAX);
    assert!(check_rows(&report, &facts, owner, plan, &changed).is_err());
    changed = entries.clone();
    changed[0].next = NonZeroU32::new(u32::MAX);
    assert!(check_rows(&report, &facts, owner, plan, &changed).is_err());
    assert!(check_rows(&report, &facts, owner, plan, &[]).is_err());
    changed = entries.clone();
    changed.push(entries[0]);
    assert!(check_rows(&report, &facts, owner, plan, &changed).is_err());
    let mut wrong_plan = plan;
    if let super::super::Kind::Field { steps, .. } = &mut wrong_plan.kind {
        steps.end = u32::MAX;
    }
    assert!(check_rows(&report, &facts, owner, wrong_plan, &entries).is_err());
    let mut wrong_call = entries[0].call;
    wrong_call.signature = TypeId::UNIT;
    assert!(retained(&facts, plan, owner, wrong_call).is_err());
    let source = crate::test_source::fixture("pa31/tests/spec/source-field-deref/lifetime.rs")?;
    let foreign = crate::test_source::source_provider(&source, "access_foreign")?;
    assert!(check_rows(&foreign, &facts, owner, plan, &entries).is_err());
    Ok(())
}
