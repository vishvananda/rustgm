//! rustc 1.98's `check_param_wf` proves a const parameter's `Ty: ConstParamTy_`
//! under the param-env of the parameter's owner. Core's `TransmuteFrom<Src,
//! const ASSUME: Assume>` carries `#[unstable_feature_bound(transmutability)]`,
//! which proves the matching bound on `impl ConstParamTy_ for Assume`; a missing
//! or different bound does not, and in a staged-API crate neither does its own
//! `#![feature]`. Each parameter's proof is independent: an owner with
//! lifetimes and several const parameters, as core's `Arguments::new<const N,
//! const M>`, proves every one of them.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/feature-bound-const-parameters";
const UNPROVEN: &str = "const parameter type must implement actual ConstParamTy";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects. Like rustc's
/// `check_false_global_bounds`, the global where clause fails first, by itself.
const CASES: [(&str, Option<&str>); 5] = [
    ("bound-trait", None),
    ("unbound-trait", Some(UNPROVEN)),
    ("other-feature-trait", Some(UNPROVEN)),
    ("bound-function", None),
    (
        "where-clause-trait",
        Some("closed trait bounds require an implementation proof"),
    ),
];

const ENVIRONMENT_CONTROLS: &str = "pa31/tests/spec/const-parameter-environments";

/// Independently checked rustc outcomes for owners with lifetimes and several
/// const parameters.
const ENVIRONMENTS: [(&str, Option<&str>); 5] = [
    ("inherent-pair", None),
    ("inherent-three", None),
    ("function-pair", None),
    ("trait-pair", None),
    ("second-unproven", Some(UNPROVEN)),
];

#[test]
fn const_parameter_types_are_proven_under_their_owner_feature_bounds() -> TestResult {
    check(CONTROLS, &CASES)
}

#[test]
fn every_const_parameter_of_an_owner_with_lifetimes_is_proven_on_its_own() -> TestResult {
    check(ENVIRONMENT_CONTROLS, &ENVIRONMENTS)
}

fn check(controls: &str, cases: &[(&str, Option<&str>)]) -> TestResult {
    for &(name, expected) in cases {
        let path = format!("{controls}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "feature_bound_parameters")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
