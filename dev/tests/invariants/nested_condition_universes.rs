//! A generic body that selects an impl with a region-binding condition attaches
//! that selection's own binders below the universe of the query that made it,
//! as rustc proves the condition with its regions instantiated in a nested
//! universe of the caller's. Its trait queries see the caller's premises in
//! the body's alias form, so a premise that names a caller lifetime does not
//! compete with its declaration-form copy. A closure body has that form from
//! entry, before any of its expressions aliases a caller lifetime.
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/scoped-body-selections";

/// Independently checked rustc 1.98.0 outcomes: `None` accepts, otherwise the
/// first error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 39] = [
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
    ("caller-region", None),
    ("ordinary-region", None),
    ("let-annotation", None),
    ("annotation-bound", None),
    ("turbofish-region", None),
    ("qualified-region", None),
    ("qualified-self-region", None),
    ("closure-qualified", None),
    ("annotated-reference", None),
    ("by-value", None),
    ("bound-only", None),
    ("unsized-output", None),
    ("region-in-input", None),
    ("premise-static", None),
    ("premise-type", None),
    ("repeated-selections", None),
    (
        "ambiguous-premises",
        Some("associated type requires an unambiguous validated implementation"),
    ),
    ("closure-selection", None),
    ("closure-selection-before-alias", None),
    ("closure-annotation", None),
    ("closure-explicit-region", None),
    ("closure-premise-static", None),
    ("closure-no-caller-region", None),
    (
        "closure-undeclared-region",
        Some("file 0: 6629: unresolved lifetime name"),
    ),
];

fn analyze(name: &str) -> TestResult<Result<(), String>> {
    let path = format!("{CONTROLS}/{name}.rs");
    let source = crate::test_source::fixture(&path)?;
    // Resolution rejects an undeclared lifetime, with its location, before
    // semantic analysis.
    let report = match crate::test_source::source_provider(&source, "scoped_bodies") {
        Ok(report) => report,
        Err(error) => return Ok(Err(error)),
    };
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
    Ok(())
}

/// Ten selections through region-bearing caller premises, in two bodies with
/// lifetime binders, build the alias view of each body's premises once.
#[test]
fn body_trait_queries_share_one_alias_view_per_body() -> TestResult {
    #[derive(Debug, Default)]
    struct Body {
        queries: usize,
        views: usize,
        folded: bool,
    }
    let capture = crate::sema::copy_trace::Capture::matching(&[
        "body-trait-environment ",
        "body-environment-view ",
    ])?;
    let result = analyze("repeated-selections")?;
    let rows = capture.rows()?;
    drop(capture);
    assert_eq!(result, Ok(()));
    let mut bodies = std::collections::BTreeMap::<&str, Body>::new();
    for row in &rows {
        if let Some(body) = row.strip_prefix("body-trait-environment body=") {
            bodies.entry(body).or_default().queries += 1;
        } else if let Some(view) = row.strip_prefix("body-environment-view body=") {
            let (body, folded) = view
                .rsplit_once(" folded=")
                .ok_or("malformed alias view row")?;
            let entry = bodies.entry(body).or_default();
            entry.views += 1;
            entry.folded |= folded == "true";
        }
    }
    for (body, record) in &bodies {
        assert_eq!(record.views, 1, "{body}: {bodies:?}");
    }
    let mut selecting: Vec<_> = bodies
        .values()
        .filter(|record| record.folded)
        .map(|record| record.queries)
        .collect();
    selecting.sort_unstable();
    assert_eq!(selecting.len(), 2, "{bodies:?}");
    assert!(selecting[0] >= 2 && selecting[1] >= 8, "{bodies:?}");
    Ok(())
}
