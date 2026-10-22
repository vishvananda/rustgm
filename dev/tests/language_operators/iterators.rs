//! Canonical iteration uses selected calls and normal loop ownership cleanup.
use super::{mutation::frontend, native, rustc};
use std::path::{Path, PathBuf};
const CASES: &[(&str, bool)] = &[
    ("owned-iterator", true),
    ("explicit-into-iterator", true),
    ("generic-into-iterator", true),
    ("generic-iterator", true),
    ("zero-iterations", true),
    ("evaluate-once", true),
    ("break", true),
    ("continue", true),
    ("nested-label", true),
    ("mutable-reference", true),
    ("moved-mutable-reference", true),
    ("nested-mutable-reference", true),
    ("shared-reference-reject", false),
    ("immutable-binding-reborrow-reject", false),
    ("moved-iterator-reject", false),
    ("break-value-reject", false),
    ("wrong-next-receiver", false),
    ("missing-iterator-prerequisite", false),
    ("crossed-associated-item", false),
    ("wrong-item-bound", false),
    ("inherent-next-shadow", true),
    ("inherent-into-iter-shadow", true),
    ("qualified-next", true),
    ("reified-next", true),
    ("source-lookalike-reject", false),
    ("ordinary-lookalike-method", true),
    ("iterator-orphan-reject", false),
    ("overlap-source-first", false),
    ("overlap-iterator-last", false),
    ("item-reference-escape-reject", false),
    ("drop-complete", true),
    ("drop-break", true),
    ("drop-continue", true),
    ("source-reference-impl", true),
    ("source-reference-conflict", false),
    ("source-reference-conflict-reverse", false),
    ("source-reference-open", false),
    ("source-reference-disjoint-reverse", true),
    ("aliased-iterator", true),
    ("prelude-iterator", true),
    ("no-prelude-iterator", true),
    ("drop-return", true),
    ("irrefutable-pattern-reject", false),
    ("unsized-generic-reference", true),
    ("borrowed-item", true),
    ("iterator-loan-conflict", false),
    ("temporary-into-iterator", true),
    ("const-for-runtime-reject", false),
];
fn fixture(name: &str) -> PathBuf {
    let assignment = if name.starts_with("drop-") {
        "pa27"
    } else {
        "pa26"
    };
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../{assignment}/tests/spec/iteration/{name}.rs"))
}
#[test]
fn custom_iteration_matches_declared_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let source = std::fs::read_to_string(fixture(name))?;
        rustc::qualify(
            &format!("parallel/pa26/iteration/source/{name}"),
            &source,
            accepted,
        )?;
        let result = frontend::checked(&source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn custom_iteration_preserves_calls_loans_and_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let check = "fn check(b:bool){if !b{let z=0;let _=1/z;}}";
    let sources = CASES
        .iter()
        .filter(|case| case.1 && case.0 != "no-prelude-iterator")
        .map(|case| Ok((case.0, std::fs::read_to_string(fixture(case.0))?)))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let cases = sources
        .iter()
        .map(|(name, source)| (*name, source.strip_prefix(check).unwrap_or(source), false))
        .collect::<Vec<_>>();
    native::check_cases("parallel/pa26/iteration/native", &cases)
}
