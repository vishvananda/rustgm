//! A `[const]` or `const` trait bound implies its ordinary trait bound, as rustc
//! 1.98 elaborates it. The projection it names is a rigid, well-formed argument
//! of a nominal type and keeps the caller's reference lifetimes there.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/const-bound-projections";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects (E0277).
const CASES: [(&str, Option<&str>); 14] = [
    ("as-deref", None),
    ("named-lifetime", None),
    ("argument", None),
    ("as-deref-mut", None),
    ("result-as-deref", None),
    ("pin-as-ref", None),
    ("slice-index", None),
    ("always-const", None),
    ("static-ref", None),
    ("raw-pointer", None),
    ("plain-bound", None),
    ("direct-ref", None),
    (
        "unsized-target",
        Some("nominal type argument must be sized"),
    ),
    (
        "missing-bound",
        Some("associated type requires an unambiguous implementation"),
    ),
];

#[test]
fn effect_qualified_bounds_form_their_projections_inside_nominal_arguments() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "const_bound_projection")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
