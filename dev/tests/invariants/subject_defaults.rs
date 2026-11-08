//! A trait path's omitted default is instantiated with its trait's `Self` as
//! the path's subject, together with the default's retained requirements, as
//! rustc does; a `Self` written among a trait path's arguments keeps meaning
//! the enclosing item's `Self`.
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const UNAMBIGUOUS: &str = "associated type requires an unambiguous implementation";
const VALIDATED: &str = "associated type requires an unambiguous validated implementation";
const OBJECT: &str = "object arguments cannot depend on an erased Self default";
const BOUNDS: &str = "associated type definition does not satisfy its declared bounds";
const OBLIGATION: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";

/// Independently checked rustc 1.98.0 outcomes: `None` accepts, otherwise the
/// first error this compiler reports for the control rustc also rejects.
const DEFAULTS: [(&str, Option<&str>); 16] = [
    ("impl-omitted", None),
    ("impl-generic", None),
    ("impl-header-phase", None),
    ("impl-missing", Some(UNAMBIGUOUS)),
    ("impl-generic-missing", Some(UNAMBIGUOUS)),
    ("bound-omitted", None),
    ("where-omitted", None),
    ("bound-header-phase", None),
    ("bound-missing", Some(VALIDATED)),
    ("qualified-concrete", None),
    ("qualified-generic", None),
    ("qualified-missing", Some(UNAMBIGUOUS)),
    ("shorthand", None),
    ("nested-default", None),
    ("nested-default-missing", Some(UNAMBIGUOUS)),
    ("object", Some(OBJECT)),
];
const WRITTEN_SELF: [(&str, Option<&str>); 8] = [
    (
        "pa19/tests/spec/trait-bounds/written-self/method-bound",
        None,
    ),
    (
        "pa19/tests/spec/trait-bounds/written-self/method-call",
        None,
    ),
    (
        "pa19/tests/spec/trait-bounds/written-self/method-call-opp",
        Some(OBLIGATION),
    ),
    (
        "pa20/tests/spec/projections/written-self/associated-bound",
        None,
    ),
    (
        "pa20/tests/spec/projections/written-self/associated-bound-opp",
        Some(BOUNDS),
    ),
    (
        "pa19/tests/spec/trait-bounds/written-self/supertrait-self",
        None,
    ),
    ("pa19/tests/spec/trait-bounds/written-self/where-self", None),
    (
        "pa19/tests/spec/trait-bounds/written-self/impl-parameter",
        None,
    ),
];

fn analyze(source: &str) -> TestResult<Result<crate::pipeline::Analysis, String>> {
    let report = crate::test_source::resolve(source, true)?;
    Ok(crate::pipeline::analyze_source(&report).map_err(|error| error.message.clone()))
}

fn check(path: &str, expected: Option<&str>) -> TestResult {
    let source = crate::test_source::fixture(&format!("{path}.rs"))?;
    match (expected, analyze(&source)?) {
        (None, Ok(_)) => Ok(()),
        (Some(message), Err(error)) => {
            assert_eq!(error, message, "{path}");
            Ok(())
        }
        (None, Err(error)) => Err(format!("{path}: rejected: {error}").into()),
        (Some(message), Ok(_)) => Err(format!("{path}: accepted, not {message}").into()),
    }
}

#[test]
fn omitted_defaults_take_their_paths_subject_for_self() -> TestResult {
    for (name, expected) in DEFAULTS {
        check(
            &format!("pa20/tests/spec/projections/self-defaults/{name}"),
            expected,
        )?;
    }
    Ok(())
}

#[test]
fn written_self_arguments_keep_the_enclosing_items_self() -> TestResult {
    for (path, expected) in WRITTEN_SELF {
        check(path, expected)?;
    }
    Ok(())
}

/// `uses` impls, bounds and qualified paths, each leaving `Fill`'s argument to
/// its default `<Self as Source>::Item`, and a body that calls through each.
fn scaling(uses: usize) -> Result<String, std::fmt::Error> {
    use std::fmt::Write as _;
    let mut source = String::from(
        "trait Source {\n    type Item;\n}\n\
         trait Fill<R = <Self as Source>::Item> {\n    type Out;\n}\n",
    );
    for index in 0..uses {
        write!(
            source,
            "struct Short{index};\nstruct Marked{index};\n\
             impl Source for Short{index} {{\n    type Item = Marked{index};\n}}\n\
             impl Fill for Short{index} {{\n    type Out = u8;\n}}\n\
             fn bound{index}<T: Source + Fill>(value: T) -> T {{\n    value\n}}\n\
             fn qualified{index}(value: <Short{index} as Fill>::Out) -> u8 {{\n    value\n}}\n\
             fn call{index}() -> u8 {{\n    bound{index}(Short{index});\n    qualified{index}(0)\n}}\n"
        )?;
    }
    source.push_str("fn main() {}\n");
    Ok(source)
}

/// A subject-bearing default fold is recomputed for each use instead of being
/// cached under its arguments, and each use pays the substitution account; the
/// work grows linearly with the uses.
#[test]
fn omitted_self_defaults_cost_linear_substitution_work() -> TestResult {
    let mut work = Vec::new();
    for uses in [8, 32] {
        let analysis = analyze(&scaling(uses)?)?.map_err(|error| format!("{uses}: {error}"))?;
        let generics = &analysis.facts.generics;
        let body = &analysis.facts.body_work;
        work.push([
            generics.unretained_steps(),
            generics.substitution_steps(),
            body.trait_selections,
            body.inference_steps,
            body.canonical_steps,
        ]);
    }
    println!(
        "self-default scaling [unretained, substitution, selections, inference, canonical] for 8 and 32 uses: {work:?}"
    );
    assert!(work[1][0] > work[0][0], "each use pays its fold: {work:?}");
    assert!(
        work[0][2] > 0,
        "each call selects through the default: {work:?}"
    );
    // Quadrupling the uses at most quintuples each measure.
    let bounded = work[1]
        .iter()
        .zip(work[0])
        .all(|(&large, small)| large <= small.max(1) * 5);
    assert!(bounded, "{work:?}");
    Ok(())
}
