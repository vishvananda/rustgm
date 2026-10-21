//! Public checked views are compared against explicitly declared pinned object outcomes.
use std::{path::Path, process::Command};
pub fn check(
    source: &Path,
    evidence: &Path,
    name: &str,
    (rust, frontend, reason): (i32, i32, &str),
) -> Result<(), Box<dyn std::error::Error>> {
    check_modes(source, evidence, name, (rust, [frontend; 3], reason))
}
pub fn check_modes(
    source: &Path,
    evidence: &Path,
    name: &str,
    (rust, frontend, reason): (i32, [i32; 3], &str),
) -> Result<(), Box<dyn std::error::Error>> {
    check_views(
        source,
        evidence,
        name,
        rust,
        &[
            ("--emit-types", frontend[0]),
            ("--emit-lowir", frontend[1]),
            ("--emit-constants", frontend[2]),
        ],
        reason,
    )
}
pub fn check_views(
    source: &Path,
    evidence: &Path,
    name: &str,
    rust: i32,
    modes: &[(&str, i32)],
    reason: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(
        !modes.is_empty(),
        "source qualification requires at least one view"
    );
    let independent = Command::new("rustc")
        .args([
            "+1.98.0",
            "--edition=2024",
            "--crate-name=probe",
            "--emit=obj",
            "-C",
            "panic=abort",
            "-o",
        ])
        .arg(evidence.join(format!("{name}.o")))
        .arg(source)
        .output()?;
    std::fs::write(
        evidence.join(format!("{name}.rustc.stderr")),
        &independent.stderr,
    )?;
    assert_eq!(
        independent.status.code(),
        Some(rust),
        "{name}: {}",
        String::from_utf8_lossy(&independent.stderr)
    );
    for &(mode, frontend) in modes {
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg(mode)
            .arg(source)
            .output()?;
        std::fs::write(
            evidence.join(format!("{name}.{mode}.stdout")),
            &output.stdout,
        )?;
        std::fs::write(
            evidence.join(format!("{name}.{mode}.stderr")),
            &output.stderr,
        )?;
        assert_eq!(
            output.status.code(),
            Some(frontend),
            "{name}/{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if frontend == 0 {
            assert!(output.stderr.is_empty(), "{name}/{mode}");
        } else {
            assert!(output.stdout.is_empty(), "{name}/{mode}");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(reason),
                "{name}/{mode}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    Ok(())
}
