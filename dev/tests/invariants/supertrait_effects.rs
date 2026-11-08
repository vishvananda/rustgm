//! rustc 1.98's wfcheck proves a const impl's trait const conditions, its
//! `[const]` supertraits and where clauses, as `[const]` goals under the impl's
//! own `[const]` assumptions. A const impl of `const trait Shared: [const] Once`
//! therefore holds through a const `Once` impl whose `[const]` premises follow
//! from those assumptions, as core's const callable reference impls do. An
//! ordinary `Once` impl, an ordinary premise or a `const` premise does not
//! satisfy the `[const]` supertrait, and a plain supertrait needs no const impl.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/const-supertrait-requirements";
const UNSATISFIED: &str = "trait implementation does not satisfy the trait's requirements";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects (E0277).
const CASES: [(&str, Option<&str>); 7] = [
    ("shared-premise", None),
    ("once-premise", None),
    ("plain-supertrait", None),
    ("ordinary-impls", None),
    ("ordinary-supertrait-impl", Some(UNSATISFIED)),
    ("ordinary-premise", Some(UNSATISFIED)),
    ("const-premise-required", Some(UNSATISFIED)),
];

#[test]
fn const_impls_prove_their_const_supertraits_as_conditional_goals() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "const_supertrait_requirements")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
