//! rustc 1.98's `specializes` registers every parent obligation under the
//! child's environment and solves them together. A parent parameter that only
//! a where clause names, as in core's `SpecExtend` pair, is inferred from the
//! child's where clauses before its `Sized` obligation is decided; ambiguity that
//! no proof resolves means the child does not specialize, never an error. Its
//! `min_specialization` check then omits every parent argument that the parent's
//! own associated equalities determine before looking for repeated parameters
//! and `'static` arguments.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const UNORDERED: &str = "overlapping implementation headers have no specialization ordering";
const EQUIVALENT: &str = "equivalent overlapping implementation headers cannot specialize";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects (E0119).
const FIXPOINT: [(&str, Option<&str>); 4] = [
    ("core-extend", None),
    ("concrete-item", None),
    ("other-item", Some(UNORDERED)),
    ("ambiguous-item", Some(UNORDERED)),
];

/// Independently checked rustc outcomes, as above. Both unmarked lifetime pairs
/// are E0119: `specializes` ignores region constraints, so each holds both ways.
const DETERMINED: [(&str, Option<&str>); 6] = [
    ("repeated-determined", None),
    (
        "repeated-undetermined",
        Some("minimal specializing implementation repeats a parameter"),
    ),
    ("static-determined", Some(EQUIVALENT)),
    ("static-undetermined", Some(EQUIVALENT)),
    ("static-marked-determined", None),
    (
        "static-marked-undetermined",
        Some("minimal specialization introduces a static lifetime"),
    ),
];

fn check(controls: &str, cases: &[(&str, Option<&str>)]) -> TestResult {
    for &(name, expected) in cases {
        let path = format!("{controls}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "parent_obligations")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}

#[test]
fn where_clause_parent_parameters_are_inferred_before_specialization_is_decided() -> TestResult {
    check("pa31/tests/spec/specialization-fixpoint", &FIXPOINT)
}

#[test]
fn parent_arguments_its_own_equalities_determine_are_not_checked_for_repeats_or_static()
-> TestResult {
    check("pa31/tests/spec/determined-parents", &DETERMINED)
}
