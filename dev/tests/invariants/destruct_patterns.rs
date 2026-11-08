//! A pattern type is const-destructible exactly when its base type is.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/pattern-const-destruct";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 6] = [
    ("pattern-field", None),
    ("nonnull-field", None),
    ("drain-fields", None),
    ("pointer-base-unbounded", None),
    ("parameter-base-bounded", None),
    (
        "parameter-base-unbounded",
        Some("const Drop declaration field requires const Destruct in its actual impl environment"),
    ),
];

#[test]
fn const_drop_fields_destroy_pattern_types_as_their_base() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "pattern_destruct")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
