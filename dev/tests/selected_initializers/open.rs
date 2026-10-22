//! Open result conversion retains body slots, caller debts and escape validity.
use super::native;
use std::path::{Path, PathBuf};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ACCEPTED: &[&str] = &["reference", "product", "paid-relation", "paid-static"];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/lifetime-initializer-open")
        .join(format!("{name}.rs"))
}
#[test]
fn open_results_preserve_actual_lifetime_rejections() -> TestResult {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-open-results/source")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for name in ACCEPTED {
        super::checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
        let view =
            std::fs::read_to_string(evidence.join(format!("{name}.--emit-constants.stdout")))?;
        assert!(view.contains("status=source-complete result=deferred"));
    }
    for (name, modes) in [
        ("dangling", [1, 1, 1]),
        ("unpaid-relation", [0, 1, 1]),
        ("unpaid-static", [0, 1, 1]),
    ] {
        super::checked_sources::check_modes(
            &fixture(name),
            &evidence,
            name,
            (1, modes, "lifetime"),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn open_result_values_execute() -> TestResult {
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
    native::check_cases("parallel/pa25-open-results/native", &cases)
}
