//! Rustc normalizes a trait method's bounds for the impl before comparing an
//! override's bounds with them. An override that repeats a `Self::Item` bound,
//! or writes the impl's own definition of it, therefore states the same
//! requirement, and a different or added bound is still stronger.
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/override-projection-bounds";

/// Independently checked rustc 1.98.0 outcomes: `None` accepts, otherwise the
/// first error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 8] = [
    ("own-item", None),
    ("supertrait", None),
    ("supertrait-spelled", None),
    ("supertrait-nested", None),
    ("supertrait-sugar", None),
    (
        "own-item-wrong",
        Some("implementation method imposes a stronger generic requirement"),
    ),
    (
        "supertrait-stronger",
        Some("implementation method imposes a stronger generic requirement"),
    ),
    (
        "supertrait-wrong",
        Some("implementation method imposes a stronger generic requirement"),
    ),
];

fn analyze(name: &str) -> TestResult<Result<(), String>> {
    let path = format!("{CONTROLS}/{name}.rs");
    let source = crate::test_source::fixture(&path)?;
    let report = crate::test_source::source_provider(&source, "override_bounds")?;
    Ok(crate::pipeline::analyze_source(&report)
        .map(|_| ())
        .map_err(|error| error.message.clone()))
}

#[test]
fn override_bounds_compare_in_the_impls_normal_forms_like_rustc() -> TestResult {
    for (name, expected) in CASES {
        match (expected, analyze(name)?) {
            (None, Ok(())) => {}
            (Some(message), Err(error)) => assert_eq!(error, message, "{name}"),
            (None, Err(error)) => return Err(format!("{name}: rejected: {error}").into()),
            (Some(message), Ok(())) => {
                return Err(format!("{name}: accepted, not {message}").into());
            }
        }
    }
    Ok(())
}
