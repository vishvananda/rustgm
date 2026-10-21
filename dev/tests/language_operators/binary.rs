//! Canonical binary families share exact primitive evidence and checked arithmetic.
use super::{mutation::frontend, native, rustc};
use std::path::{Path, PathBuf};
const CASES: &[(&str, bool, bool, bool)] = &[
    ("sub", true, true, false),
    ("mul", true, true, false),
    ("div", true, true, false),
    ("rem", true, true, false),
    ("bitand", true, true, false),
    ("bitor", true, true, false),
    ("bitxor", true, true, false),
    ("shl", true, true, false),
    ("shr", true, true, false),
    ("bool-bitand", true, true, false),
    ("bool-bitor", true, true, false),
    ("bool-bitxor", true, true, false),
    ("float-sub", true, true, false),
    ("float-mul", true, true, false),
    ("float-div", true, true, false),
    ("float-rem", true, true, false),
    ("mixed-shl", true, true, false),
    ("mixed-shr", true, true, false),
    ("signed-shr", true, true, false),
    ("wide-shl", true, true, false),
    ("wide-integer", true, true, false),
    ("wide-signed", true, true, false),
    ("user-trait", true, true, false),
    ("ambiguous", false, false, false),
    ("mixed-arithmetic", false, false, false),
    ("char", false, false, false),
    ("bool-mul", false, false, false),
    ("float-shift", false, false, false),
    ("mutable-rhs", false, false, false),
    ("wrong-output", false, false, false),
    ("const-call", false, false, false),
    ("projected-static-escape", false, false, false),
    ("reference-boundary", true, false, false),
    ("float-bitand", false, false, false),
    ("float-bitor", false, false, false),
    ("float-bitxor", false, false, false),
    ("div-zero", true, true, true),
    ("rem-zero", true, true, true),
    ("div-min", true, true, true),
    ("rem-min", true, true, true),
    ("shl-width", true, true, true),
    ("shr-width", true, true, true),
    ("sub-overflow", true, true, true),
    ("mul-overflow", true, true, true),
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa26/tests/spec/primitive-binary/{name}.rs"))
}
#[test]
fn primitive_binary_families_match_declared_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, independent, implemented, _) in CASES {
        let source = std::fs::read_to_string(fixture(name))?;
        rustc::qualify(
            &format!("parallel/pa26/binary/independent/{name}"),
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
fn primitive_binary_families_execute_with_checked_domains() -> Result<(), Box<dyn std::error::Error>>
{
    let names = CASES.iter().filter(|case| case.2).collect::<Vec<_>>();
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
    native::check_cases("parallel/pa26/binary/native", &cases)
}
