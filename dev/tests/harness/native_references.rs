//! Independent rustc/CPPGM execution checks for declared reference outcomes.
use std::{
    os::unix::process::ExitStatusExt,
    path::Path,
    process::{Command, Output},
};

#[path = "frontend_diagnostics.rs"]
pub(crate) mod frontend_diagnostics;

pub const CHECK: &str = "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\n";
fn compile(name: &str, output: Output) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "{name}: {:?}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}
/// A literal caller declaration is required; backend and runtime stderr stay strict.
#[allow(
    dead_code,
    reason = "only suites with reviewed successful warnings use this helper"
)]
pub(crate) fn compile_with_warnings(
    name: &str,
    output: Output,
    expected: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    frontend_diagnostics::accepted(&output, expected)
        .map_err(|error| format!("{name}: {error}"))?;
    Ok(output.stdout)
}

fn execute(
    name: &str,
    binary: &Path,
    panic: bool,
    independent: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(binary).output()?;
    assert!(output.stdout.is_empty(), "{name}");
    if panic && independent {
        assert_eq!(
            output.status.signal(),
            Some(6),
            "{name}: {:?}",
            output.status
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("panicked at"),
            "{name}"
        );
    } else if panic {
        assert_eq!(
            output.status.code(),
            Some(134),
            "{name}: {:?}",
            output.status
        );
        assert!(output.stderr.is_empty(), "{name}");
    } else {
        assert!(
            output.status.success() && output.stderr.is_empty(),
            "{name}: {:?}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let mut evidence = binary.as_os_str().to_os_string();
    evidence.push(".run.stderr");
    std::fs::write(evidence, output.stderr)?;
    Ok(())
}
pub fn check_cases(
    suite: &str,
    cases: &[(&str, &str, bool)],
) -> Result<(), Box<dyn std::error::Error>> {
    check_cases_with(suite, cases, |_, path| {
        compile(
            "frontend",
            Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg("--emit-lowir")
                .arg(path)
                .output()?,
        )
    })
}
pub fn check_cases_with(
    suite: &str,
    cases: &[(&str, &str, bool)],
    frontend: impl Fn(&str, &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    check_inputs(suite, cases, true, frontend)
}
/// Literal sources retain leading crate attributes and their independently frozen bytes.
#[allow(
    dead_code,
    reason = "the shared harness is also compiled in suite groups without literal-source inputs"
)]
pub fn check_literal_cases(
    suite: &str,
    cases: &[(&str, &str, bool)],
) -> Result<(), Box<dyn std::error::Error>> {
    check_inputs(suite, cases, false, |_, path| {
        compile(
            "frontend",
            Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg("--emit-lowir")
                .arg(path)
                .output()?,
        )
    })
}
/// Every source supplies its exact successful compiler stderr in the test contract.
#[allow(
    dead_code,
    reason = "only source-warning suites use literal diagnostic declarations"
)]
pub fn check_literal_cases_with_warnings(
    suite: &str,
    cases: &[(&str, &str, bool, &[u8])],
) -> Result<(), Box<dyn std::error::Error>> {
    let sources: Vec<_> = cases
        .iter()
        .map(|&(name, body, panic, _)| (name, body, panic))
        .collect();
    check_inputs(suite, &sources, false, |_, path| {
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or("missing warning source name")?;
        let expected = cases
            .iter()
            .find(|case| case.0 == name)
            .ok_or("missing literal warning declaration")?
            .3;
        compile_with_warnings(
            name,
            Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg("--emit-lowir")
                .arg(path)
                .output()?,
            expected,
        )
    })
}

fn check_inputs(
    suite: &str,
    cases: &[(&str, &str, bool)],
    prefix: bool,
    frontend: impl Fn(&str, &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!("obj/{suite}-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for &(name, body, panic) in cases {
        let path = folder.join(format!("{name}.rs"));
        let source = if prefix {
            format!("{CHECK}{body}")
        } else {
            body.to_owned()
        };
        std::fs::write(&path, &source)?;
        let independent = folder.join(format!("{name}.rustc"));
        compile(
            name,
            Command::new("rustc")
                .args([
                    "+1.98.0",
                    "--edition=2024",
                    "--crate-name=probe",
                    "--cap-lints=allow",
                    "-C",
                    "overflow-checks=yes",
                    "-C",
                    "panic=abort",
                ])
                .arg(&path)
                .arg("-o")
                .arg(&independent)
                .output()?,
        )?;
        execute(name, &independent, panic, true)?;
        let bytes = frontend(&source, &path)?;
        let again = frontend(&source, &path)?;
        assert_eq!(bytes, again, "{name}");
        let lowir = folder.join(format!("{name}.lowir"));
        std::fs::write(&lowir, bytes)?;
        let native = folder.join(format!("{name}.native"));
        compile(
            name,
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg(&lowir)
                .arg("-o")
                .arg(&native)
                .output()?,
        )?;
        execute(name, &native, panic, false)?;
    }
    Ok(())
}
