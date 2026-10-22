//! Closed aggregate and static shared results retain original and caller proofs.
use super::native;
use std::path::{Path, PathBuf};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ACCEPTED: &[&str] = &[
    "tuple",
    "array",
    "record",
    "enum",
    "promoted-reference",
    "static-reference",
    "reference-product",
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/lifetime-initializer-results")
        .join(format!("{name}.rs"))
}
#[test]
fn closed_result_outcomes_preserve_escape_and_structural_checks() -> TestResult {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-closed-results/source")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for name in ACCEPTED {
        super::checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
        let view =
            std::fs::read_to_string(evidence.join(format!("{name}.--emit-constants.stdout")))?;
        assert!(view.contains("status=source-complete result=deferred"));
    }
    for (name, reason) in [
        ("dangling-reference", "lifetime"),
        ("nonstructural-record", "PartialEq"),
    ] {
        super::checked_sources::check(&fixture(name), &evidence, name, (1, 1, reason))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn closed_result_values_execute() -> TestResult {
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
    native::check_cases("parallel/pa25-closed-results/native", &cases)
}
