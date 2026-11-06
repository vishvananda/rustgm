//! Real original body/row ownership survives consumption; malformed batches cannot start.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture() -> Result<(crate::resolve::expansion::Report, Declarations, Program)> {
    let text = crate::test_source::fixture("pa31/tests/spec/source-const-cleanup/mfield.rs")?;
    let report = crate::test_source::source_provider(&text, "actual_original_rows")?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let program = crate::body::build_concrete(&report, &facts)?;
    Ok((report, facts, program))
}
#[test]
fn constructor_free_struct_reads_keep_their_actual_original_requirement() -> Result {
    let (report, facts, mut program) = fixture()?;
    let count = program.bodies.len();
    let mut original = Originals::take(&report, &facts, &mut program)?;
    assert_eq!(original.bodies().len(), count);
    assert!(program.bodies.is_empty());
    assert!(program.drops.as_ref().ok_or("pending")?.bodies.is_empty());
    let mut found = false;
    while let Some((body, requirements)) = original.pop()? {
        let row = requirements.ok_or("actual ordinary body row")?;
        for (index, value) in body.values.iter().enumerate() {
            if matches!(value.kind, crate::body::ir::Rvalue::Read(_))
                && matches!(facts.types.kind(value.ty), Some(crate::types::Kind::Adt(item, _))
                    if facts.aggregates.get(item).is_some_and(|d|d.kind==crate::types::aggregate::Kind::Struct))
                && row.values.get(index) == Some(&true)
            {
                found = true;
            }
        }
    }
    assert!(
        found,
        "no constructor-free struct read retained its paid row"
    );
    assert!(original.pop()?.is_none());
    Ok(())
}
#[test]
fn missing_extra_and_wrong_extent_rows_fail_before_body_consumption() -> Result {
    for mode in ["missing", "extra", "extent"] {
        let (report, facts, mut program) = fixture()?;
        let count = program.bodies.len();
        let rows = &mut program.drops.as_mut().ok_or("pending")?.bodies;
        match mode {
            "missing" => {
                rows.pop().ok_or("row")?;
            }
            "extra" => rows.push(rows.first().ok_or("row")?.clone()),
            _ => rows.first_mut().ok_or("row")?.values.push(false),
        }
        assert!(
            Originals::take(&report, &facts, &mut program).is_err(),
            "{mode}"
        );
        assert_eq!(program.bodies.len(), count);
    }
    Ok(())
}

#[test]
fn body_only_moves_need_no_new_source_payer_but_actual_rows_do() -> Result {
    let report = crate::test_source::resolve("fn one()->u8{7}fn main(){let _=one();}", true)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let mut program = crate::body::build_concrete(&report, &facts)?;
    assert!(program.drops.is_none());
    let count = program.bodies.len();
    // Exhaust the actual payer without editing its private owner. Moving
    // existing body storage confers no body or cleanup permission.
    assert!(facts.charge_caller_inputs(usize::MAX, 0).is_err());
    assert!(facts.charge_caller_inputs(1, 0).is_err());
    let mut originals = Originals::take(&report, &facts, &mut program)?;
    originals.reverse(&facts)?;
    while originals.pop()?.is_some() {
        originals.shrink(&facts)?;
    }
    assert!(originals.bodies().is_empty());
    assert!(count > 0);
    let (report, facts, mut program) = fixture()?;
    let count = program.bodies.len();
    assert!(facts.charge_caller_inputs(usize::MAX, 0).is_err());
    assert!(Originals::take(&report, &facts, &mut program).is_err());
    assert_eq!(program.bodies.len(), count);
    Ok(())
}

#[test]
fn ordinary_drop_rows_pay_the_actual_report_or_legacy_owner() -> Result {
    let source =
        crate::test_source::fixture("pa34/tests/spec/temporary-editions/if-failure-2024.rs")?;
    for attributed in [false, true] {
        let text = if attributed {
            format!("#![allow(dead_code)]\n{source}")
        } else {
            source.clone()
        };
        let report = crate::test_source::resolve(&text, true)?;
        let facts = crate::pipeline::analyze_source(&report)?.facts;
        let mut program = crate::body::build_concrete(&report, &facts)?;
        assert!(report.package.source_core().is_none());
        assert_eq!(report.lint_producer().is_some(), attributed);
        assert!(program.drops.is_some());
        let count = program.bodies.len();
        let mut originals = Originals::take(&report, &facts, &mut program)?;
        originals.reverse(&facts)?;
        let mut consumed = 0;
        while let Some((_, row)) = originals.pop()? {
            assert!(row.is_some());
            consumed += 1;
            originals.shrink(&facts)?;
        }
        assert_eq!(consumed, count);
    }
    println!(
        "Originals={} Rows={} Payer={}",
        size_of::<Originals<'_>>(),
        size_of::<Rows<'_>>(),
        size_of::<rowpay::Payer<'_>>()
    );
    Ok(())
}

#[test]
fn source_mode_never_falls_back_for_missing_foreign_or_stale_inputs() -> Result {
    for mode in ["catalog", "report", "foreign", "empty", "header"] {
        let (mut report, mut facts, mut program) = fixture()?;
        let count = program.bodies.len();
        match mode {
            "catalog" => facts.source_bindings = None,
            "report" => report.resolution.source_bindings = None,
            "foreign" => {
                let text =
                    crate::test_source::fixture("pa31/tests/spec/source-const-cleanup/mfield.rs")?;
                report = crate::test_source::source_provider(&text, "foreign_original_rows")?;
            }
            "empty" => report.package.modules.clear(),
            _ => {
                let legacy = crate::test_source::resolve("fn main(){}", true)?;
                let mut replacement = crate::pipeline::analyze_source(&legacy)?.facts;
                assert!(replacement.charge_caller_inputs(1, 0).is_err());
                replacement.source_bindings = facts.source_bindings.take();
                facts = replacement;
            }
        }
        assert!(
            Originals::take(&report, &facts, &mut program).is_err(),
            "{mode}"
        );
        assert_eq!(program.bodies.len(), count);
    }
    Ok(())
}

#[test]
fn exhausted_row_owner_stays_failed_before_reversal() -> Result {
    let text =
        crate::test_source::fixture("pa34/tests/spec/temporary-editions/if-failure-2024.rs")?;
    for attributed in [false, true] {
        let text = if attributed {
            format!("#![allow(dead_code)]\n{text}")
        } else {
            text.clone()
        };
        let report = crate::test_source::resolve(&text, true)?;
        let facts = crate::pipeline::analyze_source(&report)?.facts;
        let mut program = crate::body::build_concrete(&report, &facts)?;
        let mut originals = Originals::take(&report, &facts, &mut program)?;
        let first = originals.bodies().first().ok_or("body")?.key();
        let rows = originals.requirements.as_ref().ok_or("rows")?;
        assert!(rows.payer.charge(&facts, usize::MAX, 0).is_err());
        assert!(rows.payer.charge(&facts, 1, 0).is_err());
        assert!(originals.reverse(&facts).is_err());
        assert_eq!(originals.bodies().first().ok_or("body")?.key(), first);
    }
    Ok(())
}
