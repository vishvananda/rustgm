//! A non-null pattern type over a type parameter stays generic, as core's
//! pattern-type `DispatchFromDyn` impl needs; every instance supplies a raw
//! pointer, and other bases are rejected by name.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/generic-non-null-patterns";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 6] = [
    ("core-pattern-dispatch", None),
    ("unbounded-pattern-dispatch", None),
    ("generic-pattern-field", None),
    ("non-null-pointer-dispatch", None),
    (
        "missing-pattern-dispatch",
        Some("DispatchFromDyn changed field does not prove its actual recursive relation"),
    ),
    (
        "pattern-to-pointer-dispatch",
        Some(
            "DispatchFromDyn impl must preserve its reference lifetime, pointer mutability or nominal owner",
        ),
    ),
];

/// Declared controls rustc 1.98.0 accepts and this analysis rejects by name:
/// a non-null pattern over a base that is neither a raw pointer nor a type
/// parameter, written directly or reached by instantiation. The assertion
/// fails when one changes.
const STRICTER: [(&str, &str); 3] = [
    (
        "integer-base",
        "non-null pattern base must be a raw pointer or type parameter",
    ),
    (
        "reference-base",
        "non-null pattern base must be a raw pointer or type parameter",
    ),
    (
        "integer-instance",
        "non-null pattern instance requires a raw pointer base",
    ),
];

#[test]
fn generic_non_null_patterns_keep_pointer_instances() -> TestResult {
    let stricter = STRICTER.map(|(name, message)| (name, Some(message)));
    for (name, expected) in CASES.into_iter().chain(stricter) {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "non_null_pattern")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
