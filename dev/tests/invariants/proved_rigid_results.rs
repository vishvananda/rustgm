//! A proved projection equality whose normal form is a rigid projection fixes an
//! impl's result-only parameter, as rustc does, while rejections keep their causes.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const RESULTS: &str = "pa20/tests/spec/contracts/proved-results";
const REQUIREMENTS: &str = "trait implementation does not satisfy the trait's requirements";
const AMBIGUOUS: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the rejected control.
const CONTROLS: [(&str, Option<&str>); 13] = [
    ("rigid-result", None),
    ("flatten-shape", None),
    ("result-bound-holds", None),
    ("body-selection", None),
    ("nested-rigid-result", None),
    ("two-results-same", None),
    ("two-results-assumed", None),
    ("named-result", None),
    ("result-bound-fails", Some(REQUIREMENTS)),
    ("concrete-mismatch", Some(REQUIREMENTS)),
    ("two-results-differ", Some(REQUIREMENTS)),
    ("body-input", Some(AMBIGUOUS)),
    ("result-input-unresolved", Some(AMBIGUOUS)),
];

#[test]
fn proved_rigid_results_match_independent_outcomes() -> TestResult {
    for (name, expected) in CONTROLS {
        let path = format!("{RESULTS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::resolve(&source, true)?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
