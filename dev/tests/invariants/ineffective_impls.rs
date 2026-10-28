//! Pinned independent source outcomes and header demand boundaries.
use super::*;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa40/tests/spec/ineffective-unstable-impl/{name}.rs"
    ))?)
}
#[test]
fn direct_impl_events_follow_actual_type_paths_and_effective_expectations() -> TestResult {
    for name in [
        "expect",
        "allow",
        "primitive",
        "f16",
        "generic-self",
        "stable-argument",
        "bound-unrelated",
        "never-return",
    ] {
        let source = fixture(name)?;
        let report = crate::test_source::resolve(&source, true)?;
        let analysis =
            crate::pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
        assert!(analysis.facts.warnings().next().is_none(), "{name}");
    }
    for name in [
        "unstable-self",
        "unstable-trait",
        "unstable-argument",
        "unstable-alias",
        "stable-impl",
        "inherited-unstable",
        "never-self",
        "never-parameter",
        "bound-matching",
        "bound-combined",
        "wrong-owner",
    ] {
        let source = fixture(name)?;
        let report = crate::test_source::resolve(&source, true)?;
        let analysis =
            crate::pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
        assert_eq!(
            analysis
                .facts
                .warnings()
                .map(|warning| warning.code)
                .collect::<Vec<_>>(),
            vec![WarningCode::UnfulfilledLintExpectations],
            "{name}"
        );
    }
    let source = fixture("warn")?;
    let report = crate::test_source::resolve(&source, true)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let warnings = analysis.facts.warnings().collect::<Vec<_>>();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].code, WarningCode::IneffectiveUnstableTraitImpl);
    assert_eq!(warnings[0].diagnostic.span.start, 247);
    for name in ["default-deny", "deny", "forbid"] {
        let report = crate::test_source::resolve(&fixture(name)?, true)?;
        let error = crate::pipeline::analyze_source(&report)
            .err()
            .ok_or("accepted denied actual event")?;
        assert!(
            error
                .to_string()
                .contains("unstable annotation has no effect"),
            "{name}: {error}"
        );
    }
    let report = crate::test_source::resolve(&fixture("forbid-allow")?, true)?;
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("forbidden lint weakening accepted")?;
    assert!(
        error
            .to_string()
            .contains("forbidden ineffective_unstable_trait_impl lint cannot be weakened")
    );
    Ok(())
}
#[test]
fn completed_headers_close_only_this_lint_without_demanding_function_bodies() -> TestResult {
    for (name, expected) in [
        ("crate-header-event", vec![]),
        ("nested-header-event", vec![]),
        (
            "crate-header-empty",
            vec![WarningCode::UnfulfilledLintExpectations],
        ),
    ] {
        let report = crate::test_source::resolve(&fixture(name)?, true)?;
        let facts = super::super::declarations::analyze(&report)?;
        assert!(facts.typed_function_bodies.is_empty(), "{name}");
        assert_eq!(
            facts
                .warnings()
                .map(|warning| warning.code)
                .collect::<Vec<_>>(),
            expected,
            "{name}"
        );
    }
    Ok(())
}
#[test]
fn source_declared_unstable_function_abi_does_not_fulfill_the_expectation() -> TestResult {
    let source = fixture("source-rust-call")?;
    let report = crate::test_source::source_provider(&source, "ineffective_abi")?;
    let facts = super::super::declarations::analyze(&report)?;
    assert!(facts.typed_function_bodies.is_empty());
    assert_eq!(
        facts
            .warnings()
            .map(|warning| warning.code)
            .collect::<Vec<_>>(),
        vec![WarningCode::UnfulfilledLintExpectations]
    );
    Ok(())
}
