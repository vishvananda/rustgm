//! Primitive value methods retain canonical Output proof and ordinary checked bodies.
use super::{mutation::frontend, native, rustc};
use std::path::{Path, PathBuf};
const ACCEPTED: &[&str] = &[
    "generic",
    "rhs-generic",
    "projected-output",
    "qualified",
    "dot",
    "reified",
    "alias",
    "float",
    "u128",
    "i128",
    "order",
    "overflow",
    "formation-reference",
];
const REJECTED: &[&str] = &[
    "mixed-width",
    "bool",
    "char",
    "wrong-output",
    "const-call",
    "unsafe-call",
    "mutable-rhs",
    "projection-not-injective",
    "projected-static-escape",
];
const BOUNDARIES: &[&str] = &["reference-boundary", "sub-boundary"];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa26/tests/spec/primitive-add/{name}.rs"))
}
#[test]
fn primitive_add_matches_declared_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for (name, independent, implemented) in ACCEPTED
        .iter()
        .map(|n| (*n, true, true))
        .chain(REJECTED.iter().map(|n| (*n, false, false)))
        .chain(BOUNDARIES.iter().map(|n| (*n, true, false)))
    {
        let source = std::fs::read_to_string(fixture(name))?;
        rustc::qualify(
            &format!("parallel/pa26/addition/independent/{name}"),
            &source,
            independent,
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
fn primitive_value_calls_execute_with_checked_arithmetic() -> Result<(), Box<dyn std::error::Error>>
{
    let sources = ACCEPTED
        .iter()
        .map(|n| std::fs::read_to_string(fixture(n)))
        .collect::<Result<Vec<_>, _>>()?;
    let cases = ACCEPTED
        .iter()
        .zip(&sources)
        .map(|(&name, source)| {
            Ok((
                name,
                source.strip_prefix(native::CHECK).ok_or("check prefix")?,
                name == "overflow",
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26/addition/native", &cases)
}
