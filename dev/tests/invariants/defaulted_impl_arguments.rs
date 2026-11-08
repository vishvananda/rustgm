//! An impl trait reference whose argument is left to a default that projects
//! on `Self`, `impl FromResidual for Short` with `R = <Self as Try>::Residual`,
//! normalizes that argument by the routine a written argument uses, as rustc
//! normalizes the impl's trait reference before matching it.
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/defaulted-impl-arguments";
const UNSATISFIED: &str = "trait implementation does not satisfy the trait's requirements";
const UNAMBIGUOUS: &str = "associated type requires an unambiguous implementation";

/// Independently checked rustc 1.98.0 outcomes: `None` accepts, otherwise the
/// first error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 7] = [
    ("default-omitted", None),
    ("default-written", None),
    ("default-other-member", None),
    ("default-self-bounds", None),
    ("core-never-short-circuit", None),
    ("default-bound-unsatisfied", Some(UNSATISFIED)),
    ("default-scaling", None),
];

/// A declared control whose rustc outcome this analysis does not reach yet,
/// with the outcome it has instead. The assertion fails when it changes.
/// `default-bound-satisfied`: rustc accepts `impl Fill for Short` whose
/// omitted `<Self as Source>::Item` default satisfies `R: Marker`. The default's
/// retained alias requirement is instantiated from the path's arguments
/// without the trait's `Self`, so it is rejected by name.
const GAPS: [(&str, &str); 1] = [("default-bound-satisfied", UNAMBIGUOUS)];

/// Accepted analysis reports its trait selections and canonical steps.
fn analyze(source: &str) -> TestResult<Result<[usize; 2], String>> {
    let report = crate::test_source::source_provider(source, "defaulted_impl_arguments")?;
    Ok(crate::pipeline::analyze_source(&report)
        .map(|analysis| {
            let work = &analysis.facts.body_work;
            [work.trait_selections, work.canonical_steps]
        })
        .map_err(|error| error.message.clone()))
}

#[test]
fn defaulted_impl_arguments_normalize_like_written_ones() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        match (expected, analyze(&source)?) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => {
                return Err(format!("{path}: accepted, not {message}").into());
            }
        }
    }
    for (name, message) in GAPS {
        let source = crate::test_source::fixture(&format!("{CONTROLS}/{name}.rs"))?;
        assert_eq!(analyze(&source)?, Err(message.to_owned()), "{name}");
    }
    Ok(())
}

/// The shape of `default-scaling.rs`: `types` impls with an omitted default,
/// each selected through `uses` traits whose supertraits are `Try + FromResidual`.
fn scaling(prefix: &str, types: usize, uses: usize) -> Result<String, std::fmt::Error> {
    use std::fmt::Write as _;
    let mut source = format!(
        "{prefix}pub trait Try: FromResidual {{\n    type Residual;\n}}\n\
         pub trait FromResidual<R = <Self as Try>::Residual> {{\n    \
         fn from_residual(residual: R) -> Self;\n}}\n"
    );
    for use_ in 0..uses {
        writeln!(source, "pub trait Use{use_}: Try + FromResidual {{}}")?;
    }
    for ty in 0..types {
        write!(
            source,
            "pub struct Short{ty};\npub enum Residual{ty} {{}}\n\
             impl Try for Short{ty} {{\n    type Residual = Residual{ty};\n}}\n\
             impl FromResidual for Short{ty} {{\n    \
             fn from_residual(never: Residual{ty}) -> Self {{\n        match never {{}}\n    }}\n}}\n"
        )?;
        for use_ in 0..uses {
            writeln!(source, "impl Use{use_} for Short{ty} {{}}")?;
        }
    }
    Ok(source)
}

/// Each omitted default is normalized once, when its impl's trait reference
/// forms; selecting the impl through more traits does not normalize it again,
/// and the analysis work grows no faster than the impls and their uses.
#[test]
fn defaulted_arguments_normalize_once_per_impl_not_per_selection() -> TestResult {
    let control = crate::test_source::fixture(&format!("{CONTROLS}/default-scaling.rs"))?;
    let prefix = control
        .split_once("pub trait Try")
        .ok_or("default-scaling.rs has no Try declaration")?
        .0;
    assert_eq!(
        scaling(prefix, 3, 2)?,
        control,
        "default-scaling.rs is scaling(3, 2)"
    );
    let mut work = Vec::new();
    for (types, uses) in [(4, 1), (16, 1), (4, 4), (16, 4)] {
        super::DEFAULTED.with(|count| count.set(0));
        let counts = analyze(&scaling(prefix, types, uses)?)?
            .map_err(|error| format!("{types}x{uses}: {error}"))?;
        assert_eq!(
            super::DEFAULTED.with(std::cell::Cell::take),
            types,
            "{types}x{uses}"
        );
        work.push(counts);
    }
    println!("defaulted-scaling [selections, canonical] for 4x1, 16x1, 4x4, 16x4: {work:?}");
    // Quadrupling either the impls or their uses at most sextuples the work.
    for (small, large) in [(0, 1), (2, 3), (0, 2), (1, 3)] {
        let bounded = work[large]
            .iter()
            .zip(work[small])
            .all(|(&large, small)| large <= small * 6);
        assert!(bounded, "{work:?}");
    }
    Ok(())
}
