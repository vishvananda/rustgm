//! A generic body that selects an impl with a region-binding condition attaches
//! that selection's own binders below the universe of the query that made it,
//! as rustc proves the condition with its regions instantiated in a nested
//! universe of the caller's.
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/scoped-body-selections";

/// Independently checked rustc 1.98.0 outcomes: `None` accepts, otherwise the
/// first error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 15] = [
    ("path-call", None),
    ("qualified-call", None),
    ("matched", None),
    ("constructed", None),
    ("annotated", None),
    ("generic-state", None),
    ("generic-output", None),
    ("static-output", None),
    ("fixed-output-path", None),
    ("adapter-caller", None),
    ("windows-caller", None),
    ("mismatch", Some("incompatible types")),
    (
        "unsatisfied",
        Some("no applicable method for this receiver"),
    ),
    (
        "escaping",
        Some("selected type retains an unmapped lifetime"),
    ),
    ("adapter-mismatch", Some("incompatible types")),
];

/// Declared controls whose rustc outcome (acceptance) this analysis does not
/// reach yet, each with the outcome it has instead. The assertion fails when
/// one changes. A body-selected impl parameter that a caller premise
/// determines to name a caller lifetime stays an unresolved trait goal, with
/// or without a region binder (`ordinary-region` fails the same way before
/// this change); it is a separate cause.
const GAPS: [(&str, &str); 2] = [
    (
        "caller-region",
        "trait obligation requires an unambiguous matching assumption or implementation",
    ),
    (
        "ordinary-region",
        "trait obligation requires an unambiguous matching assumption or implementation",
    ),
];

fn analyze(name: &str) -> TestResult<Result<(), String>> {
    let path = format!("{CONTROLS}/{name}.rs");
    let source = crate::test_source::fixture(&path)?;
    let report = crate::test_source::source_provider(&source, "scoped_bodies")?;
    Ok(crate::pipeline::analyze_source(&report)
        .map(|_| ())
        .map_err(|error| error.message.clone()))
}

#[test]
fn body_selections_keep_scoped_impl_conditions_and_match_independent_outcomes() -> TestResult {
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
    for (name, message) in GAPS {
        assert_eq!(analyze(name)?, Err(message.to_owned()), "{name}");
    }
    Ok(())
}
