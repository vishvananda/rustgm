//! Projection shape completion keeps selected alias ownership and normalization debts.
use super::native;
use std::path::{Path, PathBuf};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ACCEPTED: &[&str] = &["reference", "product", "paid-normalization"];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/projected-initializer-patterns")
        .join(format!("{name}.rs"))
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
    for name in ["wrong-referent", "wrong-mutability"] {
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
