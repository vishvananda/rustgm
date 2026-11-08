//! A recursively const-checked body may call an intrinsic whose source
//! fallback is itself safe to expose; bodiless and const-unstable ones refuse.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/intrinsic-fallback-exposure";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 5] = [
    ("cold-hint-length", None),
    ("cold-hint-body", None),
    ("marked-bodiless-hint", None),
    (
        "bodiless-assume-exposed",
        Some("const-stable body cannot expose this unstable intrinsic"),
    ),
    (
        "const-unstable-cold-hint",
        Some("const-stable body cannot expose this unstable intrinsic"),
    ),
];

#[test]
fn recursively_checked_bodies_expose_only_safe_intrinsic_fallbacks() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "intrinsic_exposure")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
