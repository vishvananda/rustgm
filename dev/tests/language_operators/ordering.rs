//! Ordering uses exact library contracts, ordinary provided bodies and shared operands.
use super::{mutation::frontend, native, rustc};
use std::path::{Path, PathBuf};
const CASES: &[(&str, bool, bool, bool)] = &[
    ("custom-relations", true, true, true),
    ("generic-relations", true, true, true),
    ("qualified-partial", true, true, true),
    ("dot-partial", true, true, true),
    ("reified-partial", true, true, true),
    ("custom-ord", true, true, true),
    ("generic-ord", true, true, true),
    ("reified-ord", true, true, true),
    ("override-lt", true, true, true),
    ("partial-unordered", true, true, true),
    ("primitive-signed-unsigned", true, true, true),
    ("primitive-nan", true, true, true),
    ("primitive-zero-infinity", true, true, true),
    ("heterogeneous", true, true, true),
    ("unsized-rhs", true, true, true),
    ("independent-borrows", true, true, true),
    ("ordering-identity", true, true, true),
    ("lookalike", false, false, false),
    ("missing-partial-eq", false, false, false),
    ("missing-eq", false, false, false),
    ("float-ord", false, false, false),
    ("wrong-partial-output", false, false, false),
    ("mutable-receiver", false, false, false),
    ("atomic-result", false, false, false),
    ("mismatched-primitives", false, false, false),
    ("ord-helpers", true, true, true),
    ("primitive-ord", true, true, true),
    ("primitive-relations-qualified", true, true, true),
    ("primitive-partial-reified", true, true, true),
    ("primitive-self-local-rhs", true, true, true),
    ("ordering-alias", true, true, true),
    ("ordering-shadow", true, true, true),
    ("ordering-type-mismatch", false, false, false),
    ("no-prelude-positive", true, true, false),
    ("no-prelude-dot", false, false, false),
    ("named-trait-dot", true, true, true),
    ("ambiguous-dot", false, false, false),
    ("override-relations", true, true, true),
    ("borrow-conflict", false, false, false),
    ("captured-shared", true, true, true),
    ("explicit-slots", true, true, true),
    ("extra-slot-bound", false, false, false),
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../pa26/tests/spec/ordering/{name}.rs"))
}
#[test]
fn ordering_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, accepted, implemented, _) in CASES {
        let source = std::fs::read_to_string(fixture(name))?;
        rustc::qualify(
            &format!("parallel/pa26/ordering/independent/{name}"),
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
fn ordering_execute() -> Result<(), Box<dyn std::error::Error>> {
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
    native::check_cases("parallel/pa26/ordering/native", &cases)
}
