//! Type privacy holds in bodies and the anonymous constants nested in them,
//! never in an anonymous constant of an item signature, as in rustc 1.98.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/signature-constant-privacy";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the rejected control.
const CASES: [(&str, Option<&str>); 11] = [
    ("signature-fn-result-length", None),
    ("signature-field-length", None),
    ("signature-const-argument", None),
    ("signature-discriminant", None),
    ("signature-leaked-value-length", None),
    ("signature-nested-length", None),
    ("signature-const-annotation-length", None),
    (
        "body-local-length",
        Some("semantic type or associated item is private"),
    ),
    (
        "body-repeat-count",
        Some("semantic type or associated item is private"),
    ),
    (
        "body-const-argument",
        Some("semantic type or associated item is private"),
    ),
    (
        "signature-private-method-length",
        Some("inherent method is private"),
    ),
];

#[test]
fn signature_constants_are_outside_type_privacy_while_body_constants_are_checked() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "signature_privacy")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
