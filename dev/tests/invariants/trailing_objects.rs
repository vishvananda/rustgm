//! A trait object, like a slice or str, has no fixed layout. A declaration
//! whose last field is or ends in one has no static layout to demand, as in
//! rustc 1.98; a trait object anywhere else is still an unsized field.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/dynamic-tail-layouts";
const UNSIZED: &str = "only a struct's final field may be unsized";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects (E0277).
const CASES: [(&str, Option<&str>); 8] = [
    ("request", None),
    ("plain-request", None),
    ("tuple-tail", None),
    ("lifetime-tail", None),
    ("transparent-tail", None),
    ("slice-tail", None),
    ("leading-object", Some(UNSIZED)),
    ("enum-object", Some(UNSIZED)),
];

#[test]
fn trailing_trait_objects_leave_declarations_without_a_static_layout() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "dynamic_tail_layout")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
