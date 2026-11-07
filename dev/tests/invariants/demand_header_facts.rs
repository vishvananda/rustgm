//! Header facts that a later fixed pass owns are prepared on demand exactly when
//! rustc accepts, and genuine header cycles are still diagnosed.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const LENGTHS: &str = "pa31/tests/spec/demanded-header-facts";
const PROJECTIONS: &str = "pa20/tests/spec/projections/header-demand";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the rejected control.
const CONTROLS: [(&str, &str, Option<&str>); 15] = [
    (LENGTHS, "impl-length-sub", None),
    (LENGTHS, "impl-length-chain", None),
    (
        LENGTHS,
        "impl-length-overlap",
        Some("overlapping trait implementation headers require coherence proof"),
    ),
    (
        LENGTHS,
        "impl-length-missing-sub",
        Some("associated type requires an unambiguous validated implementation"),
    ),
    (
        LENGTHS,
        "impl-length-cycle",
        Some("cycle in enum discriminant or constant dependencies"),
    ),
    (LENGTHS, "static-length-neg-sync", None),
    (
        LENGTHS,
        "static-length-missing-neg-sync",
        Some("associated type requires an unambiguous validated implementation"),
    ),
    (PROJECTIONS, "where-nested-premise", None),
    (PROJECTIONS, "where-bounded-premise", None),
    (
        PROJECTIONS,
        "where-missing-premise",
        Some("associated type requires an unambiguous implementation"),
    ),
    (PROJECTIONS, "where-result-equality", None),
    (PROJECTIONS, "self-target", None),
    (PROJECTIONS, "self-target-binding", None),
    (
        PROJECTIONS,
        "self-argument-cycle",
        Some("cycle in implementation header dependencies"),
    ),
    (
        PROJECTIONS,
        "self-target-unbound",
        Some("no associated type in the parameter's declared bounds"),
    ),
];

#[test]
fn demanded_header_facts_match_independent_outcomes() -> TestResult {
    for (directory, name, expected) in CONTROLS {
        let path = format!("{directory}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = if directory == LENGTHS {
            crate::test_source::source_provider(&source, "demanded_header")?
        } else {
            crate::test_source::resolve(&source, true)?
        };
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
