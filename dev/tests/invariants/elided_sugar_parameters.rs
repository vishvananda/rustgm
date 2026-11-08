//! Callable sugar with an elided input lifetime quantifies its impl condition
//! over that region. Rustc proves it with the region instantiated, so an impl
//! parameter that only the `Output` equality determines still resolves, and a
//! result naming the bound region determines no parameter.
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/region-sugar-impl-parameters";

/// Independently checked rustc 1.98.0 outcomes: `None` accepts, otherwise the
/// first error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 8] = [
    ("scan", None),
    ("map-windows", None),
    ("caller-signature", None),
    ("caller-region", None),
    // A generic body that selects such an impl keeps the selection's own
    // binders below the call's universe (`scoped-body-selections/`).
    ("body-caller", None),
    ("body-fixed-output", None),
    ("caller-mismatch", Some("incompatible types")),
    (
        "escaping-region",
        Some("selected type retains an unmapped lifetime"),
    ),
];

fn analyze(name: &str) -> TestResult<Result<(), String>> {
    let path = format!("{CONTROLS}/{name}.rs");
    let source = crate::test_source::fixture(&path)?;
    let report = crate::test_source::source_provider(&source, "region_sugar")?;
    Ok(crate::pipeline::analyze_source(&report)
        .map(|_| ())
        .map_err(|error| error.message.clone()))
}

#[test]
fn region_sugar_determined_impl_parameters_match_independent_outcomes() -> TestResult {
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
