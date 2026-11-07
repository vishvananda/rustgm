//! rustc's builtin binary rule unifies numeric operands before an impl is
//! selected; an operand outside it still selects its implementation by trait.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/builtin-binop-types";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the rejected control.
const CASES: [(&str, Option<&str>); 12] = [
    ("sub-literals", None),
    ("mul-const-argument", None),
    ("sub-impl-lengths", None),
    ("shift-result-lhs", None),
    (
        "shift-rhs-not-unified",
        Some("associated type requires an unambiguous validated implementation"),
    ),
    ("float-literals", None),
    ("reference-rhs", None),
    ("user-impl-by-trait", None),
    ("mixed-width", Some("incompatible types")),
    ("assign-literal", None),
    ("compare-literal", None),
    ("compare-mixed-width", Some("incompatible types")),
];

#[test]
fn builtin_binary_operands_match_independent_outcomes() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "builtin_binop")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
