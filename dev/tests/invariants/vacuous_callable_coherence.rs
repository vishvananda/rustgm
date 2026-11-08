//! Parenthesized callable sugar writes its trait predicate under a binder that
//! binds no region. rustc 1.98 evaluates it in the overlap check like any where
//! clause, so core's `impl<F> MultiCharEq for F where F: FnMut(char) -> bool` is
//! disjoint from its array, array reference and slice reference impls: no impl
//! or builtin callable candidate applies to them. A second blanket over another
//! argument tuple, or a reference to a parameter, still overlaps.
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/callable-sugar-coherence";
const OVERLAP: &str = "overlapping trait implementation headers require coherence proof";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects (E0119).
/// Core's const callable reference impls (`const impl<A: Tuple, F: ?Sized>
/// Fn<A> for &F where F: [const] Fn<A>`) prove their `[const]` supertraits as
/// `[const]` goals from that premise, as rustc's wfcheck does.
const CASES: [(&str, Option<&str>); 9] = [
    ("array", None),
    ("primitive", None),
    ("nominal", None),
    ("explicit", None),
    ("core-pattern", None),
    ("array-reference", None),
    ("slice-reference", None),
    ("other-arguments", Some(OVERLAP)),
    ("generic-reference", Some(OVERLAP)),
];

fn analyze(name: &str) -> TestResult<Result<(), String>> {
    let path = format!("{CONTROLS}/{name}.rs");
    let source = crate::test_source::fixture(&path)?;
    let report = crate::test_source::source_provider(&source, "callable_sugar_coherence")?;
    Ok(crate::pipeline::analyze_source(&report)
        .map(|_| ())
        .map_err(|error| error.message.clone()))
}

#[test]
fn vacuous_callable_conditions_prove_disjoint_impls_like_rustc() -> TestResult {
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
