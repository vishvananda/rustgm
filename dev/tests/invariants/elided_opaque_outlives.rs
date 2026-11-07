//! `'_` in a result `impl Trait + '_` is the elided output lifetime; a
//! where-clause `T: '_` is never an elision site.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/elided-opaque-outlives";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the rejected control.
const CASES: [(&str, Option<&str>); 6] = [
    ("reference-parameter", None),
    ("argument-position-parameter", None),
    ("receiver", None),
    (
        "no-input-lifetime",
        Some("cannot infer an output lifetime from these parameters"),
    ),
    (
        "two-input-lifetimes",
        Some("cannot infer an output lifetime from these parameters"),
    ),
    (
        "where-clause",
        Some("outlives bound requires a named lifetime"),
    ),
];

#[test]
fn opaque_result_outlives_bounds_elide_like_their_signature_output() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "opaque_outlives")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
