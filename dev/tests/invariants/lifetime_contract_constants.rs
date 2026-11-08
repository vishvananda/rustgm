//! Declaration-phase constants relate lifetime nominals by their declared
//! variance and outlives contracts before the crate-wide inference runs.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/declaration-lifetime-contracts";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 8] = [
    ("field-length", None),
    ("signature-length", None),
    ("later-nominal-length", None),
    ("covariant-shrink-length", None),
    ("implied-outlives-length", None),
    (
        "covariant-widen-length",
        Some("required lifetime does not outlive its target"),
    ),
    (
        "invariant-shrink-length",
        Some("required lifetime does not outlive its target"),
    ),
    (
        "outlives-escape-length",
        Some("required lifetime does not outlive its target"),
    ),
];

#[test]
fn declaration_constants_demand_reached_lifetime_contracts() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "lifetime_contract")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
