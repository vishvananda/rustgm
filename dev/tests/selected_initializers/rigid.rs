//! Unused rigid owner parameters never become value or lifetime proof authority.
use super::{checked_sources, native};
use std::path::{Path, PathBuf};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ACCEPTED: &[&str] = &["ordinary", "bound", "multiple", "lifetime", "type-outlives"];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/rigid-pattern-context")
        .join(format!("{name}.rs"))
}
#[test]
fn rigid_original_patterns_preserve_owner_and_value_independence() -> Result {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-generic-pattern/source")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for name in ACCEPTED {
        checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    let original = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/open-type-initializer-context/generic-pattern.rs");
    checked_sources::check(&original, &evidence, "original", (0, 0, ""))?;
    for (name, diagnostic) in [
        ("dependent-associated", "symbolic assumption"),
        ("skipped-dependency", "symbolic assumption"),
        ("unpaid-bound", "implement Copy"),
        ("faulting", "overflows or divides by zero"),
        ("invalid-unused-body", "incompatible types"),
    ] {
        checked_sources::check_modes(&fixture(name), &evidence, name, (1, [1; 3], diagnostic))?;
    }
    checked_sources::check_modes(
        &fixture("unpaid-outlives"),
        &evidence,
        "unpaid-outlives",
        (1, [0, 1, 1], "outlive"),
    )?;
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn rigid_pattern_instances_execute() -> Result {
    let sources = ACCEPTED
        .iter()
        .map(|name| Ok((*name, std::fs::read_to_string(fixture(name))?)))
        .collect::<std::result::Result<Vec<_>, std::io::Error>>()?;
    let cases = sources
        .iter()
        .map(|(name, text)| {
            Ok((
                *name,
                text.strip_prefix(native::CHECK).ok_or("check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    native::check_cases("parallel/pa25-generic-pattern/native", &cases)
}
