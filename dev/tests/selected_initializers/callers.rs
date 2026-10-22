//! Ordinary and nested callers retain source computation debts and exact context.
use super::{checked_sources, native};
use std::path::{Path, PathBuf};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const POSITIVES: &[&str] = &[
    "generic-body",
    "nested-initializer",
    "inline-caller",
    "generic-reference",
    "nested-reference",
    "paid-relation",
];
const LEGACY: &[(&str, &str)] = &[
    (
        "callable-early",
        "projections/definition-readiness/lowir/generic-callable-early.rs",
    ),
    (
        "callable-late",
        "projections/definition-readiness/lowir/generic-callable-late.rs",
    ),
    (
        "interleaved-lookup",
        "selection/lookup-readiness/lowir/interleaved-trait-lookup.rs",
    ),
];
fn legacy_fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa20/tests/spec")
        .join(path)
}
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/associated-callers")
        .join(format!("{name}.rs"))
}
#[test]
fn caller_outcomes_preserve_paid_relations_and_escape_failures() -> TestResult {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-caller-contexts/source")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for name in POSITIVES {
        checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    for name in ["unpaid-relation", "dangling"] {
        checked_sources::check_modes(&fixture(name), &evidence, name, (1, [0, 1, 1], "lifetime"))?;
    }
    checked_sources::check(
        &fixture("inline-local-capture"),
        &evidence,
        "inline-local-capture",
        (1, 1, ""),
    )?;
    Ok(())
}
#[test]
fn selected_implementation_can_erase_the_open_trait_lifetime() -> TestResult {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-caller-contexts/erased-selection")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for path in [
        "selection/lookup-readiness/semantics/trait-caller-lifetime.rs",
        "constants/pattern/semantics/erased-type-called.rs",
        "projections/definition-readiness/lowir/generic-callable-early.rs",
        "projections/definition-readiness/lowir/generic-callable-late.rs",
        "selection/lookup-readiness/lowir/interleaved-trait-lookup.rs",
        "constants/value/lowir/qualified_lifetime.rs",
    ] {
        let input = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa20/tests/spec")
            .join(path);
        let name = input
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or("fixture name")?;
        checked_sources::check(&input, &evidence, name, (0, 0, ""))?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn caller_values_execute_after_their_own_contracts() -> TestResult {
    let mut sources = POSITIVES
        .iter()
        .map(|name| Ok((*name, std::fs::read_to_string(fixture(name))?)))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    for &(name, path) in LEGACY {
        sources.push((name, std::fs::read_to_string(legacy_fixture(path))?));
    }
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
    native::check_cases("parallel/pa25-caller-contexts/native", &cases)
}
