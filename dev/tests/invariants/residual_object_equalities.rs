//! A rigid projection subject selects its declaration's associated equalities
//! even when the crate has formed trait objects and so has dynamic shapes.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/projection-object-equalities";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the rejected control.
const CASES: [(&str, Option<&str>); 4] = [
    ("residual-with-object", None),
    ("residual-output-equation", None),
    ("residual-output-mismatch", Some("incompatible types")),
    (
        "residual-try-type-mismatch",
        Some("associated type projection requires normalization"),
    ),
];

#[test]
fn rigid_projection_equalities_hold_beside_trait_objects() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "projection_equation")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
