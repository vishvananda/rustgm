//! Actual safety consumers retain bounded diagnostics without granting permission.
use super::*;
use crate::pipeline::{self, Analysis};
use std::fmt::Write as _;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const OPERATION: &str = "unsafe operation in unsafe function requires an explicit unsafe block";
fn analyze(source: &str) -> TestResult<(Report, Declarations)> {
    let report = crate::test_source::resolve(source, true)?;
    let Analysis { facts, .. } = pipeline::analyze_source(&report)?;
    Ok((report, facts))
}
fn codes(facts: &Declarations) -> Vec<WarningCode> {
    facts.warnings().map(|warning| warning.code).collect()
}
#[test]
fn actual_operation_and_reference_levels_emit_only_their_required_warnings() -> TestResult {
    for (source, expected) in [
        (
            "unsafe fn f(p:*const u8)->u8{*p}",
            vec![WarningCode::UnsafeOpInUnsafeFn],
        ),
        (
            "#[warn(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
            vec![WarningCode::UnsafeOpInUnsafeFn],
        ),
        (
            "#[allow(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
            vec![],
        ),
        ("unsafe fn f(p:*const u8)->u8{unsafe{*p}}", vec![]),
        (
            "static mut X:u8=0;#[warn(static_mut_refs)]fn f(){unsafe{let _=&X;}}",
            vec![WarningCode::StaticMutRefs],
        ),
        (
            "static mut X:u8=0;#[allow(static_mut_refs)]fn f(){unsafe{let _=&X;}}",
            vec![],
        ),
        ("static mut X:u8=0;fn f(){let _=&raw const X;}", vec![]),
    ] {
        let (_, facts) = analyze(source)?;
        assert_eq!(codes(&facts), expected, "{source}");
    }
    for source in [
        "#[warn(unsafe_op_in_unsafe_fn)]fn f(p:*const u8)->u8{*p}",
        "#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
        "#[forbid(unsafe_op_in_unsafe_fn)]#[allow(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
        "static mut X:u8=0;fn f(){unsafe{let _=&X;}}",
    ] {
        assert!(analyze(source).is_err(), "{source}");
    }
    Ok(())
}
#[test]
fn distinct_operations_and_static_reference_keep_distinct_diagnostic_identities() -> TestResult {
    let source = "unsafe fn f(p:*const u8){let _=*p;let _=*p;}";
    let (_, facts) = analyze(source)?;
    assert_eq!(codes(&facts), vec![WarningCode::UnsafeOpInUnsafeFn; 2]);
    let offsets: Vec<_> = facts
        .warnings()
        .map(|w| w.diagnostic.span.start as usize)
        .collect();
    assert_eq!(
        offsets,
        source
            .match_indices("*p")
            .map(|(i, _)| i)
            .collect::<Vec<_>>()
    );
    for source in [
        "#![warn(static_mut_refs)]static mut X:u8=0;unsafe fn f(){let _=&X;}",
        "#![warn(static_mut_refs)]static mut X:u8=0;unsafe fn f(){let ref _r=X;}",
    ] {
        let (_, facts) = analyze(source)?;
        assert_eq!(
            codes(&facts),
            [WarningCode::UnsafeOpInUnsafeFn, WarningCode::StaticMutRefs]
        );
    }
    Ok(())
}
#[test]
fn macro_expansions_keep_logical_keys_before_shared_physical_projection() -> TestResult {
    let source = "macro_rules!read{($p:expr)=>{*$p}}unsafe fn f(p:*const u8){let _=read!(p);let _=read!(p);}";
    let (_, facts) = analyze(source)?;
    assert_eq!(codes(&facts), vec![WarningCode::UnsafeOpInUnsafeFn; 2]);
    let records: Vec<_> = facts.warnings().collect();
    assert!(!records[0].diagnostic.expansions.is_empty());
    assert!(!records[1].diagnostic.expansions.is_empty());
    let table = facts.must_use_contracts.as_deref().ok_or("warning owner")?;
    assert_eq!(table.diagnosed.len(), 2);
    assert_eq!(table.work.warnings, 2);
    Ok(())
}
#[test]
fn original_generic_warning_is_not_repeated_by_runtime_instances() -> TestResult {
    let mut report = crate::test_source::resolve(
        "unsafe fn f<T:Copy>(p:*const T)->T{*p}fn main(){let a=1u8;let b=2u16;unsafe{let _=f::<u8>(&a as *const u8);let _=f::<u8>(&a as *const u8);let _=f::<u16>(&b as *const u16);}}",
        true,
    )?;
    let Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    assert_eq!(codes(&facts), [WarningCode::UnsafeOpInUnsafeFn]);
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    let program = crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    assert!(!program.symbols.is_empty());
    assert_eq!(codes(&facts), [WarningCode::UnsafeOpInUnsafeFn]);
    Ok(())
}
#[test]
fn prior_safety_warning_survives_the_actual_later_primary_failure() -> TestResult {
    let baseline = crate::test_source::resolve("fn bad(){let _:bool=1;}", true)?;
    let baseline = pipeline::analyze_source(&baseline)
        .err()
        .ok_or("baseline primary")?;
    let source = "unsafe fn first(p:*const u8)->u8{*p}fn bad(){let _:bool=1;}";
    let report = crate::test_source::resolve(source, true)?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("later primary")?;
    assert_eq!(error.message, baseline.message);
    assert_eq!(error.warnings().len(), 1);
    assert_eq!(error.warnings()[0].code, WarningCode::UnsafeOpInUnsafeFn);
    assert_eq!(error.warnings()[0].diagnostic.message, OPERATION);
    assert!(error.to_string().contains(baseline.message.as_str()));
    Ok(())
}
#[test]
fn warning_storage_is_linear_and_no_policy_or_warning_owner_appears_without_a_use() -> TestResult {
    let (_, ordinary) = analyze("fn ordinary(){}")?;
    assert!(ordinary.must_use_contracts.is_none());
    let mut payload = None;
    for width in [1, 8, 32, 128] {
        let mut source = String::new();
        for index in 0..width {
            writeln!(source, "unsafe fn f{index}(p:*const u8)->u8{{*p}}")?;
        }
        let (_, facts) = analyze(&source)?;
        assert_eq!(facts.warnings().count(), width);
        assert_eq!(facts.retained_policy_storage(), (0, 0, 0));
        let table = facts
            .must_use_contracts
            .as_deref()
            .ok_or("actual cold sink")?;
        assert_eq!(
            table.warning_bytes,
            width * *payload.get_or_insert(table.warning_bytes)
        );
        println!(
            "unsafe warnings width={width} steps={} bytes={} capacity={} Error={} Body={} WarningCode={}",
            table.warning_steps,
            table.warning_bytes,
            table.warnings.capacity(),
            size_of::<Error>(),
            size_of::<crate::body::ir::Body>(),
            size_of::<WarningCode>()
        );
        assert_eq!(size_of::<crate::body::ir::Body>(), 456);
        assert_eq!(size_of::<WarningCode>(), 1);
    }
    Ok(())
}
#[test]
fn actual_sink_revisits_deduplicate_and_failed_new_records_keep_attempted_work() -> TestResult {
    let (report, mut facts) = analyze("unsafe fn f(p:*const u8){let _=*p;let _=*p;}")?;
    let mut keys: Vec<_> = facts
        .must_use_contracts
        .as_deref()
        .ok_or("sink")?
        .diagnosed
        .iter()
        .copied()
        .collect();
    keys.sort_unstable_by_key(|(_, _, offset, _)| *offset);
    let (code, key, offset, issue) = keys[0];
    let before = facts
        .must_use_contracts
        .as_deref()
        .ok_or("sink")?
        .warning_steps;
    for _ in 0..8 {
        facts.must_use_diagnostic(&report, key, offset, issue, code, OPERATION.into())?;
    }
    let table = facts.must_use_contracts.as_deref_mut().ok_or("sink")?;
    assert_eq!(table.warnings.len(), 2);
    assert_eq!(table.warning_steps, before + 8);
    let (code, key, offset, issue) = keys[1];
    // Fault-inject storage exhaustion before retrying this genuine source event.
    table.diagnosed.remove(&(code, key, offset, issue));
    table.warnings.pop();
    table.warning_bytes = 64 * 1024 * 1024;
    for _ in 0..2 {
        let before = facts
            .must_use_contracts
            .as_deref()
            .ok_or("sink")?
            .warning_steps;
        let error = facts
            .must_use_diagnostic(&report, key, offset, issue, code, OPERATION.into())
            .err()
            .ok_or("exhausted warning store")?;
        assert!(error.message.contains("warning work or storage budget"));
        let table = facts.must_use_contracts.as_deref().ok_or("sink")?;
        assert!(table.warning_steps > before);
        assert_eq!(table.warnings.len(), 1);
        assert!(!table.diagnosed.contains(&(code, key, offset, issue)));
    }
    Ok(())
}
