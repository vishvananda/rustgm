//! Closed pattern continuations publish scalar ranges only after their dependencies.
use super::native;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ROOT: &str = "../pa25/tests/spec/constant-pattern-readiness";
const ACCEPTED: &[&str] = &[
    "callee-enum-cast",
    "suspended-body-pattern",
    "exterior-dispatch-owner",
    "skipped-associated-pattern",
    "discriminant-match",
    "forward-pattern",
    "nested-constant",
    "callee-pattern",
    "forward-callee",
    "forward-enum-cast",
    "ordinary-pattern",
    "associated-scalar",
    "associated-nested",
    "associated-specialized",
    "associated-range",
    "array-bound-pattern",
    "source-independent-instances",
    "source-self-pattern",
];
const REJECTED: &[&str] = &[
    "discriminant-cycle",
    "pattern-value-cycle",
    "cross-enum-cycle",
    "associated-cycle",
    "interleaved-lifetime-bound",
    "source-opaque-pattern",
];
const BOUNDARIES: &[(&str, &str)] = &[(
    "interleaved-lifetime",
    "selected constant arguments are not concrete",
)];
fn source(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(ROOT)
            .join(format!("{name}.rs")),
    )
}
fn output() -> Result<PathBuf, std::io::Error> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa25-pattern-readiness/source")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
fn object(folder: &Path, name: &str, source: &str, accepted: bool) -> TestResult {
    let path = folder.join(format!("{name}.rs"));
    std::fs::write(&path, source)?;
    let result = Command::new("rustc")
        .args([
            "+1.98.0",
            "--edition=2024",
            "--crate-name=pattern_readiness",
            "--emit=obj",
            "-o",
        ])
        .arg(folder.join(format!("{name}.o")))
        .arg(path)
        .output()?;
    std::fs::write(folder.join(format!("{name}.rustc.stderr")), &result.stderr)?;
    assert_eq!(
        result.status.code(),
        Some(i32::from(!accepted)),
        "{name}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
fn views(folder: &Path, name: &str, accepted: bool, diagnostic: &str) -> TestResult {
    for stage in ["types", "lowir", "constants"] {
        let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg(format!("--emit-{stage}"))
            .arg(folder.join(format!("{name}.rs")))
            .output()?;
        std::fs::write(
            folder.join(format!("{name}.{stage}.stderr")),
            &result.stderr,
        )?;
        assert_eq!(
            result.status.code(),
            Some(i32::from(!accepted)),
            "{name}/{stage}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        if accepted {
            assert!(!result.stdout.is_empty() && result.stderr.is_empty());
            if stage == "constants" {
                let repeated = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                    .arg("--emit-constants")
                    .arg(folder.join(format!("{name}.rs")))
                    .output()?;
                assert!(repeated.status.success());
                assert_eq!(result.stdout, repeated.stdout);
            }
        } else {
            assert!(result.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&result.stderr).contains(diagnostic),
                "{name}/{stage}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        std::fs::write(folder.join(format!("{name}.{stage}")), &result.stdout)?;
    }
    Ok(())
}
#[test]
fn completed_patterns_and_cycles_match_independent_objects() -> TestResult {
    let folder = output()?;
    for &name in ACCEPTED.iter().chain(REJECTED) {
        let accepted = ACCEPTED.contains(&name);
        object(&folder, name, &source(name)?, accepted)?;
        let diagnostic = if name.ends_with("cycle") {
            "cycle"
        } else if name == "source-opaque-pattern" {
            "symbolic assumption"
        } else {
            "selected constant arguments are not concrete"
        };
        views(&folder, name, accepted, diagnostic)?;
    }
    Ok(())
}
#[test]
fn remaining_readiness_boundaries_keep_rust_acceptance() -> TestResult {
    let folder = output()?;
    for &(name, diagnostic) in BOUNDARIES {
        object(&folder, name, &source(name)?, true)?;
        views(&folder, name, false, diagnostic)?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn completed_patterns_have_native_values() -> TestResult {
    let owned = ACCEPTED
        .iter()
        .map(|&name| Ok((name, source(name)?)))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    let cases = owned
        .iter()
        .map(|(name, source)| {
            Ok((
                *name,
                source
                    .strip_prefix(native::CHECK)
                    .ok_or("fixture lacks native check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa25-pattern-readiness/native", &cases)
}
