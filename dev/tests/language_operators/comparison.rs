//! Canonical equality uses ordinary selected methods and independently shared operands.
use super::{mutation::frontend, native, rustc};
use std::path::{Path, PathBuf};
const CASES: &[(&str, bool, bool, bool)] = &[
    ("manual-borrow", true, true, true),
    ("default-ne", true, true, true),
    ("override-ne", true, true, true),
    ("heterogeneous", true, true, true),
    ("primitive-local-rhs", true, true, true),
    ("generic", true, true, true),
    ("generic-rhs", true, true, true),
    ("eq-supertrait", true, true, true),
    ("qualified", true, true, true),
    ("dot", true, true, true),
    ("reified", true, true, true),
    ("signed", true, true, true),
    ("unsigned", true, true, true),
    ("float-nan", true, true, true),
    ("bool-char", true, true, true),
    ("order", true, true, true),
    ("captured-shared", true, true, true),
    ("unsized-rhs", true, true, true),
    ("canonical-alias", true, true, true),
    ("shadow", true, true, true),
    ("nonreflexive-eq", true, true, true),
    ("explicit-borrow-slots", true, true, true),
    ("local-trait", true, true, true),
    ("missing-eq", false, false, false),
    ("wrong-receiver", false, false, false),
    ("wrong-result", false, false, false),
    ("eq-without-partial", false, false, false),
    ("float-eq", false, false, false),
    ("mixed-domain", false, false, false),
    ("moved", false, false, false),
    ("right-mutation", false, false, false),
    ("const-call", false, false, false),
    ("orphan", false, false, false),
    ("lookalike-operator", false, false, false),
    ("pattern-structural", false, false, false),
    ("reference-boundary", true, false, false),
    ("derive-boundary", true, false, false),
    ("no-prelude-operator", true, true, false),
    ("no-prelude-dot", false, false, false),
    ("shadow-dot", false, false, false),
    ("ambiguous-dot", false, false, false),
    ("projected-rhs", true, true, true),
    ("independent-input-regions", true, true, true),
    ("numeric-competing-nominal", true, true, true),
    ("numeric-suffixed-mismatch", false, false, false),
    ("unknown-competing-nominal", false, false, false),
    ("override-no-eq", true, true, true),
    ("expired-heterogeneous", false, false, false),
    ("implicit-qualified", true, true, true),
    ("implicit-reified", true, true, true),
    ("qualified-lookalike", true, true, true),
    ("independent-default-ne", true, true, true),
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../pa26/tests/spec/comparison/{name}.rs"))
}
#[test]
fn comparison_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted, implemented, _) in CASES {
        let source = std::fs::read_to_string(fixture(name))?;
        rustc::qualify(
            &format!("parallel/pa26/comparison/independent/{name}"),
            &source,
            accepted,
        )?;
        let result = frontend::checked(&source);
        if result.is_ok() != implemented {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn comparisons_execute() -> Result<(), Box<dyn std::error::Error>> {
    let names = CASES.iter().filter(|case| case.3).collect::<Vec<_>>();
    let sources = names
        .iter()
        .map(|case| std::fs::read_to_string(fixture(case.0)))
        .collect::<Result<Vec<_>, _>>()?;
    let cases = names
        .iter()
        .zip(&sources)
        .map(|(case, source)| {
            Ok((
                case.0,
                source.strip_prefix(native::CHECK).ok_or("check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26/comparison/native", &cases)
}
