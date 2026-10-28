//! Replacement hints retain their source identity and require the actual feature.
use super::*;
use crate::{pipeline, support::diagnostics::WarningCode};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa40/tests/spec/deprecation-suggestion/{name}.rs"
    ))?)
}

#[test]
fn actual_replacement_metadata_keeps_deprecation_events_and_hard_feature_gate() -> TestResult {
    for (name, expected) in [
        ("accepted", vec![]),
        ("used", vec![WarningCode::Deprecated]),
    ] {
        let report = crate::test_source::resolve(&fixture(name)?, true)?;
        let analysis = pipeline::analyze_source(&report)?;
        assert_eq!(
            analysis
                .facts
                .warnings()
                .map(|warning| warning.code)
                .collect::<Vec<_>>(),
            expected,
            "{name}"
        );
    }
    for name in [
        "missing-feature",
        "wrong-value",
        "duplicate",
        "macro-allowance",
    ] {
        let result = crate::test_source::resolve(&fixture(name)?, true).and_then(|report| {
            pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        let error = result.err().ok_or("invalid replacement hint accepted")?;
        assert!(
            error.contains(if name == "wrong-value" {
                "string"
            } else if name == "duplicate" {
                "unique"
            } else {
                "requires deprecated_suggestion"
            }),
            "{name}: {error}"
        );
    }
    Ok(())
}

#[test]
fn decoded_hint_and_exact_argument_span_stay_in_existing_cold_owner() -> TestResult {
    let source = fixture("accepted")?;
    let report = crate::test_source::resolve(&source, true)?;
    let producer = report.lint_producer().ok_or("actual source producer")?;
    let values = producer
        .owners
        .iter()
        .filter_map(|owner| {
            owner
                .parsed
                .deprecated
                .as_ref()
                .and_then(|deprecation| deprecation.suggestion.as_ref())
        })
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 1);
    let (replacement, span) = values[0];
    assert_eq!(replacement.as_ref(), "as_nonzero_usize");
    assert_eq!(
        &source[span.start as usize..span.end as usize],
        "suggestion = \"as_nonzero_usize\""
    );
    assert_eq!(size_of::<super::super::expectations::Deprecation>(), 72);
    assert_eq!(size_of::<super::super::producer::Owner>(), 160);
    assert_eq!(size_of::<Producer>(), 368);
    Ok(())
}
