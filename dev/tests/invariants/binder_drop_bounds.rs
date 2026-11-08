//! Drop coverage proves binder-scoped bounds, such as paren-sugar `FnOnce(T)`,
//! inside their scope instead of refusing them as flat predicates.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/quantified-drop-bounds";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 7] = [
    ("plain-guard", None),
    ("core-const-guard", None),
    ("higher-ranked-guard", None),
    (
        "added-bound",
        Some("Drop impl adds type requirements to its nominal type"),
    ),
    (
        "stronger-output",
        Some("Drop impl adds type requirements to its nominal type"),
    ),
    (
        "added-higher-ranked",
        Some("Drop impl adds type requirements to its nominal type"),
    ),
    (
        "const-field-destruct",
        Some("const Drop declaration field requires const Destruct in its actual impl environment"),
    ),
];

#[test]
fn drop_coverage_proves_quantified_bounds_in_scope() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "drop_bound")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
