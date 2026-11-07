//! Declaration-phase constants execute source `assume` itself: a true
//! condition returns unit, a false one fails, and no fallback body runs.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/assume-constant-execution";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 7] = [
    ("assume-true-length", None),
    ("assert-unchecked-length", None),
    ("associated-buffer-length", None),
    ("bodiless-assume-length", None),
    ("assume-length-mismatch", Some("incompatible types")),
    (
        "assume-false-length",
        Some("`assume` was executed with a false condition"),
    ),
    (
        "empty-fallback-false",
        Some("`assume` was executed with a false condition"),
    ),
];

#[test]
fn declaration_constants_execute_assume_without_its_fallback() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "assume_constant")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
