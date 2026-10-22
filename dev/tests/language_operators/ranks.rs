//! Canonical comparison Ordering retains nominal identity through ordinary checked methods.
use super::{mutation::frontend, native, rustc};
use std::path::{Path, PathBuf};
const CASES: &[(&str, bool, bool, bool)] = &[
    ("entry-ord-first", true, true, false),
    ("entry-partial-first", true, true, false),
    ("all-nine-pairs", true, true, false),
    ("qualified-methods", true, true, false),
    ("dot-methods", true, true, false),
    ("reified-methods", true, true, false),
    ("generic-eq", true, true, false),
    ("generic-partial", true, true, false),
    ("generic-ord", true, true, false),
    ("generic-helpers", true, true, false),
    ("helper-panic", true, true, true),
    ("aliases", true, true, false),
    ("variant-patterns", true, true, false),
    ("signed-casts", true, true, false),
    ("shared-capture", true, true, false),
    ("local-rhs", true, true, false),
    ("local-rhs-partial", true, true, false),
    ("lookalike-manual", true, true, false),
    ("lookalike-trait", true, true, false),
    ("no-prelude", true, true, false),
    ("no-lookalike-impl", false, false, false),
    ("cross-atomic", false, false, false),
    ("atomic-not-ord", false, false, false),
    ("wrong-rhs", false, false, false),
    ("foreign-impl", false, false, false),
    ("wrong-reified", false, false, false),
    ("const-call", false, false, false),
    ("reference-boundary", true, true, false),
    ("array-boundary", true, false, false),
    ("constant-pattern", true, true, false),
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa26/tests/spec/ordering-impls/{name}.rs"))
}
#[test]
fn canonical_ordering_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted, implemented, _) in CASES {
        let source = std::fs::read_to_string(fixture(name))?;
        rustc::qualify(
            &format!("parallel/pa26/ordering-impls/independent/{name}"),
            &source,
            accepted,
        )?;
        let checked = frontend::checked(&source);
        if checked.is_ok() != implemented {
            failures.push(format!("{name}: {:?}", checked.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn canonical_ordering_executes() -> Result<(), Box<dyn std::error::Error>> {
    let names = CASES
        .iter()
        .filter(|case| case.1 && case.2 && case.0 != "no-prelude")
        .collect::<Vec<_>>();
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
                case.3,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26/ordering-impls/native", &cases)
}
