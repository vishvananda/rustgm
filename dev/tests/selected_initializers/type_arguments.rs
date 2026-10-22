//! Open type arguments retain recursive caller ownership and ordinary proof gates.
use super::native;
use std::path::{Path, PathBuf};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ACCEPTED: &[&str] = &[
    "paid",
    "ordinary",
    "pattern",
    "inline",
    "nested",
    "function-binder",
    "repeated",
    "projected",
    "mutable-type",
    "closed-control",
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/open-type-initializer-context")
        .join(format!("{name}.rs"))
}
#[test]
fn open_type_arguments_retain_caller_lifetimes_and_debts() -> TestResult {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-open-type-contexts/source")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for name in ACCEPTED {
        super::checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    for (name, modes, diagnostic) in [
        ("unpaid", [0, 1, 1], "lifetime"),
        ("wrong-result", [1, 1, 1], "incompatible types"),
    ] {
        super::checked_sources::check_modes(
            &fixture(name),
            &evidence,
            name,
            (1, modes, diagnostic),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn open_type_values_execute() -> TestResult {
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
    native::check_cases("parallel/pa25-open-type-contexts/native", &cases)
}
