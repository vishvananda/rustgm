//! A declaration-phase constant whose Copy decision selects a generic nominal
//! impl waits for that impl's on-demand validation instead of failing.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/generic-copy-waits";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 7] = [
    ("option-copy-wait", None),
    ("root-copy-wait", None),
    ("nonzero-copy-wait", None),
    ("copy-length-mismatch", Some("incompatible types")),
    (
        "not-copy-move",
        Some("local may be uninitialized or moved at this read"),
    ),
    (
        "field-not-copy",
        Some("source Copy implementation requires every field to implement actual Copy"),
    ),
    (
        "validation-cycle",
        Some("cycle in implementation header dependencies"),
    ),
];

/// A declared control whose rustc outcome this analysis does not reach yet,
/// with the outcome it has instead. The assertion fails when it changes.
/// - `later-projected-copy-wait`: the waiting const fn destructures a field
///   whose type projects through an impl declared after it; that body cannot
///   suspend for the associated type definition.
const GAPS: [(&str, Option<&str>); 1] = [(
    "later-projected-copy-wait",
    Some("missing completed associated type definition"),
)];

#[test]
fn declaration_constants_wait_for_generic_copy_validation() -> TestResult {
    for (name, expected) in CASES.into_iter().chain(GAPS) {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "generic_copy")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
