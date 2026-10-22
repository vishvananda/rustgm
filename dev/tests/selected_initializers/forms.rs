//! Projection shape completion keeps selected alias ownership and normalization debts.
use super::native;
use std::path::{Path, PathBuf};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ACCEPTED: &[&str] = &[
    "reference",
    "product",
    "paid-normalization",
    "selected-tuple",
    "selected-record",
    "nested-reference",
    "nested-tuple",
    "constant-function",
    "selected-reference",
    "selected-paid",
    "selected-forward-pattern",
    "selected-early-pattern",
    "selected-nested-pattern",
    "literal-reference",
    "named-scalar-value",
];
fn fixture(name: &str) -> PathBuf {
    let (assignment, cluster) = match name {
        "literal-reference" => ("pa16", "named-pattern-rules"),
        "named-scalar-value" => ("pa13", "named-pattern-rules"),
        _ => ("pa25", "projected-initializer-patterns"),
    };
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../{assignment}/tests/spec/{cluster}/{name}.rs"))
}
#[test]
fn projected_patterns_keep_shape_and_normalization_debts() -> TestResult {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-projected-patterns/source")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for name in ACCEPTED {
        super::checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    super::checked_sources::check_modes(
        &fixture("unpaid-normalization"),
        &evidence,
        "unpaid-normalization",
        (1, [0, 1, 1], "lifetime"),
    )?;
    for (name, diagnostic) in [
        ("unpaid-source-relation", "lifetime"),
        ("unused-runtime-call", "non-const"),
    ] {
        super::checked_sources::check_modes(
            &fixture(name),
            &evidence,
            name,
            (1, [0, 1, 1], diagnostic),
        )?;
    }
    super::checked_sources::check_modes(
        &fixture("selected-unpaid"),
        &evidence,
        "selected-unpaid",
        (1, [0, 1, 1], "required lifetime"),
    )?;
    // PA13 explicitly follows the Reference's non-reference constant rule,
    // while pinned rustc requires an explicit reference pattern.
    super::checked_sources::check(
        &fixture("selected-scalar"),
        &evidence,
        "selected-scalar",
        (1, 0, ""),
    )?;
    for name in [
        "wrong-referent",
        "wrong-mutability",
        "selected-wrong-referent",
        "selected-wrong-mutability",
    ] {
        super::checked_sources::check(
            &fixture(name),
            &evidence,
            name,
            (1, 1, "incompatible types"),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn projected_pattern_values_execute() -> TestResult {
    let sources = ACCEPTED
        .iter()
        .map(|name| Ok((*name, std::fs::read_to_string(fixture(name))?)))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let cases = sources
        .iter()
        .map(|(name, text)| {
            Ok((
                *name,
                text.strip_prefix(native::CHECK).ok_or("check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa25-projected-patterns/native", &cases)
}
