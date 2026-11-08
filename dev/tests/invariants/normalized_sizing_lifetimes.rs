//! A selection query that normalized a builtin goal's inputs keeps their rigid
//! projections: neither the builtin shape proof of a source sizing, `Freeze` or
//! `Destruct` goal nor the builtin `Sized` proof normalizes them again, where it
//! would have to restore the query's caller lifetimes without owning them.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/rigid-projection-lifetimes";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects (E0277).
/// `destruct-unsized-pair` reports the folded selection label; its cause is
/// the unsatisfied formation prerequisite, not an unmapped caller lifetime.
const CASES: [(&str, Option<&str>); 21] = [
    ("alias-bound", None),
    ("elided-lifetime", None),
    ("lang-deref", None),
    ("sized-target", None),
    ("nested-holder", None),
    ("supertrait-bound", None),
    ("method", None),
    ("sized-tuple", None),
    ("where-premise", None),
    ("raw-pointer", None),
    ("concrete-result", None),
    ("unsized-tuple", Some("nominal type argument must be sized")),
    (
        "missing-bound",
        Some("associated type requires an unambiguous implementation"),
    ),
    ("freeze-nested", None),
    ("freeze-premise", None),
    ("destruct-nested", None),
    ("destruct-premise", None),
    ("freeze-plain-reference", None),
    ("destruct-generic-reference", None),
    ("destruct-raw-projection", None),
    (
        "destruct-unsized-pair",
        Some("invalid canonical trait selection"),
    ),
];

#[test]
fn normalized_builtin_goals_keep_rigid_projections_and_caller_lifetimes() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "rigid_projection")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
