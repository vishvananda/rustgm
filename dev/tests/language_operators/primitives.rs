//! Canonical primitive evidence materializes checked ordinary method bodies.
use super::{mutation::frontend, native};
use std::path::{Path, PathBuf};
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa26/tests/spec/primitive-assignment/{name}.rs"))
}
const ACCEPTED: &[&str] = &[
    "explicit-forwarding",
    "named-lifetime",
    "static-rhs",
    "add",
    "add-ref",
    "sub",
    "sub-ref",
    "mul",
    "mul-ref",
    "div",
    "div-ref",
    "rem",
    "rem-ref",
    "and",
    "and-ref",
    "or",
    "or-ref",
    "xor",
    "xor-ref",
    "shl",
    "shl-ref",
    "shr",
    "shr-ref",
    "float-add",
    "float-rem",
    "bool-and",
    "bool-xor",
    "u128",
    "i128",
    "generic-order",
    "inferred",
    "reified",
    "ufcs",
    "local-reference",
    "local-first-generic",
    "local-trait",
    "alias-local",
    "tuple-covered",
    "array-covered",
    "nominal-covered",
    "empty-local-enum",
    "local-object",
    "projected-local",
];
const REJECTED: &[&str] = &[
    "implicit-forwarding-boundary",
    "alias-conflict",
    "bool-add",
    "float-bit",
    "char-add",
    "mixed-width",
    "mutable-rhs",
    "orphan",
    "orphan-ref",
    "uncovered",
    "uncovered-reference",
    "library-wrapper",
    "alias-library",
    "tuple-local",
    "array-local",
    "library-object",
];
#[test]
fn primitive_contracts_match_independent_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for (&name, accepted) in ACCEPTED
        .iter()
        .map(|n| (n, true))
        .chain(REJECTED.iter().map(|n| (n, false)))
    {
        let source = std::fs::read_to_string(fixture(name))?;
        super::rustc::qualify(
            &format!("assignment-{name}"),
            &source,
            accepted || name == "implicit-forwarding-boundary",
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
fn primitive_traits_and_source_order_execute() -> Result<(), Box<dyn std::error::Error>> {
    let sources = ACCEPTED
        .iter()
        .map(|n| std::fs::read_to_string(fixture(n)))
        .collect::<Result<Vec<_>, _>>()?;
    let cases = ACCEPTED
        .iter()
        .zip(&sources)
        .map(|(&n, s)| {
            Ok((
                n,
                s.strip_prefix(native::CHECK).ok_or("check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa26/primitive-assignment/native", &cases)
}
