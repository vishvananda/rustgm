//! An opaque application interned before its declaration waits for its own
//! identity during storage preparation and never fails another item.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/undeclared-opaque-storage";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the rejected control.
const CASES: [(&str, Option<&str>); 4] = [
    ("declared-before-length", None),
    (
        "failed-before-length",
        Some("cannot infer an output lifetime from these parameters"),
    ),
    ("waiting-gat-argument", None),
    ("waiting-gat-mismatch", Some("incompatible types")),
];

#[test]
fn undeclared_opaque_applications_wait_during_storage_preparation() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "opaque_storage")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
