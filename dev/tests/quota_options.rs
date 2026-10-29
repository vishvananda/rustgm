//! Explicit resource limits preserve default source output and hard language debt.
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const FLAGS: [&str; 3] = [
    "--source-contract-work-limit",
    "--source-contract-attempted-bytes",
    "--source-contract-retained-bytes",
];
fn fixture(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-contract-limits")
            .join(format!("{name}.rs")),
    )?)
}
fn run(source: &str, mode: &str, args: &[&str]) -> Result<Output> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(mode)
        .arg("-")
        .args(args)
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
fn ample() -> Vec<String> {
    FLAGS
        .iter()
        .map(|flag| format!("{flag}=4294967295"))
        .collect()
}
#[test]
fn finite_numeric_forms_and_defaults_preserve_actual_source_output() -> Result {
    for name in ["ordinary", "source"] {
        let source = fixture(name)?;
        let provider = [
            "--source-core",
            "minicore@limits_cli",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "limits_cli",
        ];
        let base = if name == "source" { &provider[..] } else { &[] };
        let default = run(&source, "--emit-names", base)?;
        assert!(
            default.status.success(),
            "{}",
            String::from_utf8_lossy(&default.stderr)
        );
        let values = ample();
        let mut equal = base.to_vec();
        equal.extend(values.iter().map(String::as_str));
        let explicit = run(&source, "--emit-names", &equal)?;
        assert!(
            explicit.status.success(),
            "{}",
            String::from_utf8_lossy(&explicit.stderr)
        );
        assert_eq!(default.stdout, explicit.stdout);
        assert_eq!(default.stderr, explicit.stderr);
        let mut spaced = base.to_vec();
        for flag in FLAGS {
            spaced.extend([flag, "4294967295"]);
        }
        let explicit = run(&source, "--emit-names", &spaced)?;
        assert!(explicit.status.success());
        assert_eq!(default.stdout, explicit.stdout);
        assert_eq!(default.stderr, explicit.stderr);
    }
    for flag in FLAGS {
        for value in [
            "",
            "-1",
            "+1",
            "1_000",
            "1.0",
            "0x10",
            " 1",
            "１",
            "4294967296",
            "unlimited",
        ] {
            let option = format!("{flag}={value}");
            let output = run("", "--emit-names", &[&option])?;
            assert_eq!(output.status.code(), Some(2), "{option}");
            assert!(String::from_utf8_lossy(&output.stderr).contains(flag));
        }
        let missing = run("", "--emit-names", &[flag])?;
        assert_eq!(missing.status.code(), Some(2));
        let option = format!("{flag}=1");
        let duplicate = run("", "--emit-names", &[&option, flag, "2"])?;
        assert_eq!(duplicate.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&duplicate.stderr).contains("duplicate"));
    }
    Ok(())
}
#[test]
fn zeros_reject_real_contract_owners_without_granting_language_permissions() -> Result {
    for name in ["ordinary", "source"] {
        let source = fixture(name)?;
        let provider = [
            "--source-core",
            "minicore@limits_cli",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "limits_cli",
        ];
        let base = if name == "source" { &provider[..] } else { &[] };
        for (flag, reason) in FLAGS.into_iter().zip([
            "work budget",
            "attempted storage budget",
            "retained storage budget",
        ]) {
            let option = format!("{flag}=0");
            let mut args = base.to_vec();
            args.push(&option);
            let output = run(&source, "--emit-names", &args)?;
            assert_eq!(output.status.code(), Some(1), "{name} {flag}");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(reason),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    let zero: Vec<_> = FLAGS.iter().map(|flag| format!("{flag}=0")).collect();
    let output = run(
        "pub fn value()->u8{7u8}",
        "--emit-names",
        &zero.iter().map(String::as_str).collect::<Vec<_>>(),
    )?;
    assert!(output.status.success());
    let values = ample();
    let args: Vec<_> = values.iter().map(String::as_str).collect();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let unsafe_source =
        std::fs::read_to_string(root.join("pa40/tests/spec/lint-cap-allow/hard-unsafe.rs"))?;
    for source in [fixture("hard-type")?, unsafe_source] {
        let default = run(&source, "--emit-lowir", &[])?;
        let explicit = run(&source, "--emit-lowir", &args)?;
        assert_eq!(default.status.code(), Some(1));
        assert_eq!(explicit.status.code(), Some(1));
        assert_eq!(default.stderr, explicit.stderr);
        assert_eq!(default.stdout, explicit.stdout);
    }
    let wrong_role = std::fs::read_to_string(
        root.join("pa31/tests/spec/source-builtin-declarations/unknown-role.rs"),
    )?;
    let provider = [
        "--source-core",
        "minicore@limits_cli",
        "--crate-name",
        "minicore",
        "--crate-disambiguator",
        "limits_cli",
    ];
    let default = run(&wrong_role, "--emit-names", &provider)?;
    let mut larger = provider.to_vec();
    larger.extend(args);
    let explicit = run(&wrong_role, "--emit-names", &larger)?;
    assert_eq!(default.status.code(), Some(1));
    assert_eq!(explicit.status.code(), Some(1));
    assert_eq!(default.stderr, explicit.stderr);
    Ok(())
}
