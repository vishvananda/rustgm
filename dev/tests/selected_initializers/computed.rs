//! Source computation preserves mandatory caller obligations and untyped values.
use super::native;
use std::path::{Path, PathBuf};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const CASES: &[(&str, bool)] = &[
    ("caller-lifetime", true),
    ("caller-static-bound", true),
    ("caller-cross-bound", true),
    ("repeated-callers", true),
    ("fresh-dispatcher", true),
    ("cached-unpaid-caller", false),
    ("missing-cross-bound", false),
];
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/lifetime-initializer-execution")
        .join(format!("{name}.rs"))
}
#[test]
fn source_values_keep_paid_and_unpaid_caller_outcomes() -> TestResult {
    let evidence = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-source-computation/source")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&evidence)?;
    for &(name, accepted) in CASES {
        let status = i32::from(!accepted);
        if accepted {
            super::checked_sources::check(&fixture(name), &evidence, name, (0, 0, ""))?;
            let view =
                std::fs::read_to_string(evidence.join(format!("{name}.--emit-constants.stdout")))?;
            assert!(view.contains("status=source-complete result=deferred"));
            continue;
        }
        super::checked_sources::check_modes(
            &fixture(name),
            &evidence,
            name,
            (status, [0, status, status], "lifetime"),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn source_values_execute_without_static_lifetime_equivalence() -> TestResult {
    let sources = CASES
        .iter()
        .filter(|case| case.1)
        .map(|(name, _)| Ok((*name, std::fs::read_to_string(fixture(name))?)))
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
    native::check_cases("parallel/pa25-source-computation/native", &cases)
}
