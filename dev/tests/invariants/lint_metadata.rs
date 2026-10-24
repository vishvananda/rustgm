//! Lint metadata is authenticated provenance, never permission for an unsafe operation.
use super::*;
fn unsafe_provider(definitions: &str, expression: &str) -> TestResult<Report> {
    let source = format!(
        "{}{}pub fn answer()->u32{{{expression}}}",
        ROOT.replace(
            "allow_internal_unstable)",
            "allow_internal_unstable,allow_internal_unsafe)"
        ),
        definitions
    );
    Ok(crate::test_source::source_provider(
        &source,
        "unsafe_lint_metadata",
    )?)
}
#[test]
fn unsafe_lint_metadata_uses_only_actual_outer_literal_and_captured_contexts() -> TestResult {
    for (definitions, expression, expected) in [
        (
            "#[allow_internal_unsafe]macro_rules!route{()=>{probe()}}",
            "route!()",
            true,
        ),
        (
            "#[allow_internal_unsafe]macro_rules!route{($e:expr)=>{$e}}",
            "route!(probe())",
            false,
        ),
        (
            "#[allow_internal_unsafe]macro_rules!inner{()=>{probe()}}macro_rules!outer{($e:expr)=>{$e}}",
            "outer!(inner!())",
            true,
        ),
        (
            "macro_rules!inner{()=>{probe()}}#[allow_internal_unsafe]macro_rules!outer{()=>{inner!()}}",
            "outer!()",
            false,
        ),
        ("macro_rules!route{()=>{probe()}}", "route!()", false),
    ] {
        let report = unsafe_provider(definitions, expression)?;
        let actual = sites(&report, "probe")?;
        assert_eq!(actual.len(), 1);
        let mut session = report.macro_allowances();
        for _ in 0..32 {
            assert_eq!(session.allows_unsafe_code_lint(actual[0])?, expected);
        }
        assert!(!session.allows(actual[0], &KnownFeature::CoreIntrinsics.name())?);
    }
    Ok(())
}
#[test]
fn unsafe_lint_metadata_keeps_witness_and_global_failed_budget_on_warm_queries() -> TestResult {
    let mut report = unsafe_provider(
        "#[allow_internal_unsafe]macro_rules!route{()=>{probe()}}",
        "route!()",
    )?;
    let site = sites(&report, "probe")?[0];
    let mut session = report.macro_allowances();
    assert!(session.allows_unsafe_code_lint(site)?);
    let memo = session.memo.take();
    drop(session);
    super::identity::replace_source(
        &mut report,
        "#[allow_internal_unsafe]macro_rules!",
        "#[allow_internal_unsaff]macro_rules!",
    )?;
    let mut cold = report.macro_allowances();
    assert!(cold.allows_unsafe_code_lint(site).is_err());
    let mut warm = MacroAllowanceSession {
        report: &report,
        memo,
    };
    // The borrowed public session cannot survive source mutation; this deliberate
    // private transferred-memo opponent must still reject when global work failed.
    report
        .package
        .source_core()
        .ok_or("source")?
        .macros
        .charge(1_000_001, 0)
        .err()
        .ok_or("failed work")?;
    assert!(warm.allows_unsafe_code_lint(site).is_err());
    Ok(())
}
#[test]
fn duplicate_unsafe_lint_declarations_and_empty_feature_rows_are_retained_once() -> TestResult {
    let report = unsafe_provider(
        "#[allow_internal_unsafe]#[allow_internal_unsafe]#[allow_internal_unstable()]macro_rules!route{()=>{probe()}}",
        "route!()",
    )?;
    let site = sites(&report, "probe")?[0];
    let mut session = report.macro_allowances();
    assert!(session.allows_unsafe_code_lint(site)?);
    let before = owner_usage(&report)?;
    for _ in 0..128 {
        assert!(session.allows_unsafe_code_lint(site)?);
    }
    let after = owner_usage(&report)?;
    assert_eq!(before.1, after.1);
    assert_eq!(session.memo.as_ref().ok_or("memo")?.headers.len(), 1);
    eprintln!(
        "unsafe metadata cold Declaration={} HeaderData={} SourceCore={} Session={}",
        size_of::<crate::resolve::bindings::panic_contracts::Declaration>(),
        size_of::<HeaderData>(),
        size_of::<SourceCore>(),
        size_of::<MacroAllowanceSession<'_>>()
    );
    Ok(())
}
