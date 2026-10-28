//! Driver caps are explicit inputs; language errors and successful IR stay stable.
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa40/tests/spec/lint-cap-allow")
            .join(format!("{name}.rs")),
    )?)
}
fn run(mode: &str, source: &str, args: &[&str]) -> Result<Output> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(mode)
        .args(args)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let write = child
        .stdin
        .take()
        .ok_or("stdin")?
        .write_all(source.as_bytes());
    let output = child.wait_with_output()?;
    if let Err(error) = write
        && (error.kind() != std::io::ErrorKind::BrokenPipe || output.status.success())
    {
        return Err(error.into());
    }
    Ok(output)
}
#[test]
fn standard_cap_forms_preserve_default_strict_and_hard_driver_boundaries() -> Result {
    let source = fixture("unknown-expect")?;
    let strict = run("--emit-lowir", &source, &[])?;
    assert_eq!(strict.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&strict.stderr).contains("unsupported lint expectation"));
    for args in [
        &["--cap-lints=allow"][..],
        &["--cap-lints", "allow"][..],
        &["--cap-lints=allow", "--edition=2021"][..],
    ] {
        let capped = run("--emit-lowir", &source, args)?;
        assert!(
            capped.status.success(),
            "{}",
            String::from_utf8_lossy(&capped.stderr)
        );
        assert!(!capped.stdout.is_empty() && capped.stderr.is_empty());
    }
    for value in ["warn", "deny", "forbid", "force-warn", "unknown"] {
        let output = run("--emit-types", &source, &["--cap-lints", value])?;
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported lint cap"));
    }
    for name in [
        "hard-type",
        "hard-unsafe",
        "hard-ctfe",
        "hard-feature",
        "hard-feature-name",
        "hard-syntax",
        "unknown-tool",
        "malformed-reason",
    ] {
        let output = run("--emit-lowir", &fixture(name)?, &["--cap-lints=allow"])?;
        assert_eq!(output.status.code(), Some(1), "{name}");
    }
    Ok(())
}
#[test]
fn capped_source_headers_and_warning_only_lowir_keep_actual_semantic_inputs() -> Result {
    let args = [
        "--source-core",
        "minicore@cap_cli",
        "--crate-name",
        "minicore",
        "--crate-disambiguator",
        "cap_cli",
        "--cap-lints=allow",
    ];
    let source = run("--emit-types", &fixture("source-unknown-expect")?, &args)?;
    assert!(
        source.status.success(),
        "{}",
        String::from_utf8_lossy(&source.stderr)
    );
    assert!(!source.stdout.is_empty() && source.stderr.is_empty());
    let warning = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa40/tests/spec/lint-expectations/unreachable-warning.rs"),
    )?;
    let strict = run("--emit-lowir", &warning, &[])?;
    let capped = run("--emit-lowir", &warning, &["--cap-lints=allow"])?;
    assert!(strict.status.success() && capped.status.success());
    assert!(String::from_utf8_lossy(&strict.stderr).contains("warning[unreachable_patterns]"));
    assert!(capped.stderr.is_empty());
    assert_eq!(strict.stdout, capped.stdout);
    Ok(())
}
