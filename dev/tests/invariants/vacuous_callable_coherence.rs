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
const CASES: [(&str, Option<&str>); 6] = [
    ("array", None),
    ("primitive", None),
    ("nominal", None),
    ("explicit", None),
    ("other-arguments", Some(OVERLAP)),
    ("generic-reference", Some(OVERLAP)),
];

/// Declared controls whose rustc outcome this analysis does not reach yet,
/// each with the outcome it has instead. The assertion fails when one changes.
/// Core's const callable reference impls (`const impl<A: Tuple, F: ?Sized>
/// Fn<A> for &F where F: [const] Fn<A>`) do not yet prove their `[const]`
/// supertrait requirement from a `[const]` premise, which rustc accepts. That
/// requirement is checked after coherence, so these controls report it only
/// once their `MultiCharEq` impls are disjoint; `generic-reference` shows the
/// overlap is still reported first.
const GAPS: [(&str, &str); 3] = [
    ("core-pattern", REQUIREMENTS),
    ("array-reference", REQUIREMENTS),
    ("slice-reference", REQUIREMENTS),
];
const REQUIREMENTS: &str = "trait implementation does not satisfy the trait's requirements";

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
    for (name, message) in GAPS {
        assert_eq!(analyze(name)?, Err(message.to_owned()), "{name}");
    }
    Ok(())
}
