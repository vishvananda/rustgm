//! rustc 1.98's `specializes` equates the specializing impl's trait reference
//! with the parent's fresh instantiation, lifetimes included, so each parent
//! header lifetime is the child's corresponding one. Core's `&mut I` iterator
//! specializations, which only add `I: Sized`, therefore specialize; a `'static`
//! argument where the parent is generic, or a repeated lifetime, does not under
//! `min_specialization`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/lifetime-specialization";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 9] = [
    ("double-ended-ref", None),
    ("iterator-ref", None),
    ("shared-reference", None),
    ("named-lifetimes", None),
    ("concrete-referent", None),
    ("nominal-lifetime", None),
    ("static-reference", None),
    (
        "static-specialization",
        Some("equivalent overlapping implementation headers cannot specialize"),
    ),
    (
        "repeated-lifetime",
        Some("minimal specializing implementation repeats a parameter"),
    ),
];

#[test]
fn specialization_names_parent_header_lifetimes_by_the_child_like_rustc() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "header_lifetimes")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
