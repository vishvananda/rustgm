//! Canonical static arguments retain ordinary result typing and lifetime proof.
use super::native;
use std::path::{Path, PathBuf};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ACCEPTED: &[&str] = &["reference", "product", "record", "array", "local-borrow"];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/canonical-initializer-results")
        .join(format!("{name}.rs"))
}
#[test]
fn canonical_results_preserve_storage_and_escape_checks() -> TestResult {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-canonical-results/source")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for name in ACCEPTED {
        super::checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    super::checked_sources::check(
        &fixture("dangling"),
        &evidence,
        "dangling",
        (1, 1, "lifetime"),
    )?;
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn canonical_result_values_execute() -> TestResult {
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
    native::check_cases("parallel/pa25-canonical-results/native", &cases)
}
