//! Declaration-phase constants execute source bit counts, read implicit enum
//! discriminants and wait for nominals declared later in source order.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/declaration-constant-execution";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the rejected control.
const CASES: [(&str, Option<&str>); 12] = [
    ("bit-count-lengths", None),
    ("bit-count-mismatch", Some("incompatible types")),
    (
        "bit-count-nonzero-zero",
        Some("a nonzero bit count was executed on zero"),
    ),
    ("implicit-discriminant-length", None),
    ("implicit-discriminant-mismatch", Some("incompatible types")),
    ("implicit-discriminant-layout", None),
    (
        "implicit-discriminant-layout-mismatch",
        Some("incompatible types"),
    ),
    ("projected-choice-length", None),
    ("later-field-template", None),
    ("later-field-template-mismatch", Some("incompatible types")),
    ("later-associated-value", None),
    (
        "later-associated-value-not-copy",
        Some("associated type definition does not satisfy its declared bounds"),
    ),
];

#[test]
fn declaration_constants_execute_bit_counts_discriminants_and_later_nominals() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "declaration_constant")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
