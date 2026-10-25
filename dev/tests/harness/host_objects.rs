//! Pinned `LowIR` objects and independent rustc executables link with the same real C provider.
use crate::native_references::frontend_diagnostics;
use std::{
    path::Path,
    process::{Command, Output},
};

fn successful(name: &str, output: Output) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "{name}: {}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}
pub fn check_cases(
    suite: &str,
    provider: &str,
    cases: &[(&str, &str)],
) -> Result<(), Box<dyn std::error::Error>> {
    let declared: Vec<_> = cases
        .iter()
        .map(|&(name, path)| (name, path, b"".as_slice()))
        .collect();
    check_cases_with_warnings(suite, provider, &declared)
}
/// Literal reviewed diagnostics apply only to the frontend; all other tools remain strict.
#[allow(
    dead_code,
    reason = "only suites with reviewed compiler warnings call this entrypoint"
)]
pub fn check_cases_with_warnings(
    suite: &str,
    provider: &str,
    cases: &[(&str, &str, &[u8])],
) -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!("obj/parallel/{suite}/cases/{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    let object = folder.join("provider.o");
    successful(
        "C provider",
        Command::new("cc")
            .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-c"])
            .arg(root.join(provider))
            .arg("-o")
            .arg(&object)
            .output()?,
    )?;
    for &(name, path, expected_diagnostics) in cases {
        let input = folder.join(format!("{name}.native.rs"));
        std::fs::copy(root.join(path), &input)?;
        let independent = folder.join(format!("{name}.rustc"));
        successful(
            name,
            Command::new("rustc")
                .args([
                    "+1.98.0",
                    "--edition=2024",
                    "--crate-name=object_native",
                    "--cap-lints=allow",
                    "-C",
                    "overflow-checks=yes",
                    "-C",
                    "panic=abort",
                ])
                .arg("-C")
                .arg(format!("link-arg={}", object.display()))
                .arg(&input)
                .arg("-o")
                .arg(&independent)
                .output()?,
        )?;
        let expected = successful(name, Command::new(&independent).output()?)?;
        let compiler = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-lowir", "--crate-name", "object_native"])
            .arg(&input)
            .output()?;
        frontend_diagnostics::accepted(&compiler, expected_diagnostics)
            .map_err(|error| format!("{name}: {error}"))?;
        let lowir = compiler.stdout;
        let lowir_path = folder.join(format!("{name}.lowir"));
        std::fs::write(&lowir_path, &lowir)?;
        let native = folder.join(format!("{name}.rustgm.o"));
        successful(
            name,
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(&lowir_path)
                .arg("-o")
                .arg(&native)
                .output()?,
        )?;
        let executable = folder.join(format!("{name}.rustgm"));
        successful(
            name,
            Command::new("cc")
                .arg(&native)
                .arg(&object)
                .arg("-o")
                .arg(&executable)
                .output()?,
        )?;
        let actual = successful(name, Command::new(&executable).output()?)?;
        assert_eq!(actual, expected, "{name}");
        assert!(actual.is_empty());
    }
    Ok(())
}
