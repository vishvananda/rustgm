//! Ownership opponents use the unchanged provider's actually checked raw rows.
use super::{HeaderIndex, Work};
use crate::support::crates::generics::source as s;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(super) fn actual() -> Result<s::SourceRows> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "source_domains")?;
    let provider = report
        .package
        .source_core()
        .ok_or("actual provider mode")?
        .provider
        .clone();
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    Ok(crate::lower::instances::inspect_source_transport(
        &report,
        &mut facts,
        &provider,
        analysis.prepared,
    )?
    .rows)
}
fn call_body(rows: &s::SourceRows) -> Result<usize> {
    rows.originals
        .iter()
        .position(|body| !body.calls.is_empty())
        .ok_or_else(|| "actual source Call".into())
}
fn foreign_formal(rows: &mut s::SourceRows, body: usize) -> Result {
    let foreign = rows
        .functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some("identity")
        })
        .ok_or("actual foreign generic function")?
        .declaration;
    let ty = rows
        .types
        .iter()
        .position(|ty| {
            matches!(ty,
                s::Type::Parameter(slot) if rows.slots[slot.0 as usize].owner == foreign)
        })
        .ok_or("actual foreign type formal")?;
    let target = rows.originals[body].calls[0].target;
    let argument = rows.function_applications[target.0 as usize]
        .arguments
        .start;
    rows.arguments[argument as usize] = s::Argument::Type(
        crate::support::crates::generics::TypeRef(u32::try_from(ty)?),
    );
    Ok(())
}

fn corrupt(rows: &mut s::SourceRows, case: usize) -> Result {
    let body = call_body(rows)?;
    match case {
        0 => {
            foreign_formal(rows, body)?;
        }
        1 => {
            let owner = rows.originals[body].declaration;
            let header = rows
                .functions
                .iter()
                .find(|header| header.declaration == owner)
                .ok_or("actual caller header")?;
            let old = header.original.binder;
            let public = header.callable.binder;
            let region = rows
                .regions
                .iter_mut()
                .find(|region| matches!(region, s::Region::Bound {binder, ..} if *binder == old))
                .ok_or("actual original region")?;
            let s::Region::Bound { binder, .. } = region else {
                return Err("bound region".into());
            };
            *binder = public;
        }
        2 => {
            let owner = rows.originals[body].body;
            let foreign = rows
                .originals
                .iter()
                .find(|body| body.body != owner && !body.variables.is_empty())
                .ok_or("foreign actual allocation owner")?
                .body;
            let region = rows
                .regions
                .iter_mut()
                .find(|region| matches!(region, s::Region::Variable {body, ..} if *body == owner))
                .ok_or("actual free region")?;
            *region = s::Region::Variable {
                body: foreign,
                slot: 0,
            };
        }
        3 => rows.originals[body].calls[0].site = 0,
        4 => {
            let signature = rows.originals[body].calls[0].signature;
            let s::Type::Function(signature) = &mut rows.types[signature.0 as usize] else {
                return Err("actual Call signature".into());
            };
            signature.unsafe_ = true;
        }
        5 => {
            let value = rows.originals[body].calls[0].value;
            let duplicate = rows.originals[body]
                .statements
                .iter()
                .find(|statement| statement.action == s::Action::Evaluate(value))
                .copied()
                .ok_or("actual Call evaluation")?;
            rows.originals[body].statements.push(duplicate);
        }
        6 => {
            let target = rows.originals[body].calls[0].target;
            rows.function_applications
                .push(rows.function_applications[target.0 as usize]);
        }
        _ => {
            let header = rows
                .functions
                .iter()
                .find(|header| header.regions.late.start < header.regions.late.end)
                .ok_or("actual late-bound source header")?;
            let index = header.regions.late.start as usize;
            let slot = rows
                .slots
                .iter()
                .position(|slot| matches!(slot.kind, s::ParameterKind::Type { .. }))
                .ok_or("actual nonregional formal")?;
            rows.region_slots[index] = s::SlotRef(u32::try_from(slot)?);
        }
    }
    Ok(())
}

#[test]
fn actual_binder_type_application_and_operation_domains_reject_foreign_equal_arity_opponents()
-> Result {
    let original = actual()?;
    for case in 0..8 {
        let mut rows = original.clone();
        corrupt(&mut rows, case)?;
        let mut work = Work::default();
        let headers = HeaderIndex::new(&rows, 0, &mut work)?;
        let before = work.steps;
        let Err(error) = headers.domains(&mut work) else {
            return Err(format!("case{case} unexpectedly returned an index").into());
        };
        assert!(work.steps > before);
        println!(
            "domain-opponent={case} attempted={} header-retained={} header-peak={} published=false error={error}",
            work.steps - before,
            headers.retained_bytes(),
            headers.peak_bytes()
        );
    }
    Ok(())
}

#[test]
fn domain_budget_failure_consumes_work_without_a_returned_index_or_descriptor() -> Result {
    let rows = actual()?;
    let mut work = Work::default();
    let headers = HeaderIndex::new(&rows, 0, &mut work)?;
    work.steps = crate::support::crates::MAX_COMPONENTS;
    assert!(headers.domains(&mut work).is_err());
    assert!(work.steps > crate::support::crates::MAX_COMPONENTS);
    Ok(())
}
