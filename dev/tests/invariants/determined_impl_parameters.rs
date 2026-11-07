//! Quantified clause outputs constrain impl parameters exactly when rustc accepts.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const TRAIT: &str = "impl type and const parameters must constrain Self or the trait application";
const TARGET: &str = "impl type and const parameters must constrain its target";
const CALLS: &str = "pa24/tests/spec/constrained-call-outputs";
const BINDERS: &str = "pa21/tests/spec/binders/constrained-results";

/// Independently checked rustc outcomes: `None` accepts, otherwise the expected
/// unconstrained-parameter rejection.
const CONTROLS: [(&str, &str, Option<&str>); 10] = [
    (CALLS, "trait-output", None),
    (CALLS, "inherent-output", None),
    (CALLS, "chained-outputs", None),
    (CALLS, "higher-ranked-output", None),
    (CALLS, "input-only", Some(TRAIT)),
    (CALLS, "unknown-input", Some(TRAIT)),
    (CALLS, "impl-lifetime-input", Some(TRAIT)),
    (CALLS, "inherent-input-only", Some(TARGET)),
    (BINDERS, "bound-result", None),
    (BINDERS, "bound-unknown-subject", Some(TRAIT)),
];

#[test]
fn quantified_outputs_constrain_impl_parameters_only_through_known_inputs() -> TestResult {
    for (directory, name, expected) in CONTROLS {
        let path = format!("{directory}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::resolve(&source, true)?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
