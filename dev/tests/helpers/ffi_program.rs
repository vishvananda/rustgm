//! Shared checked-LowIR execution with literal independently declared C observers.
use crate::provider_traits::{TestResult, native::run};
use std::{path::Path, process::Command};

/// One source-provider crate, its lints capped, lowered from its exports.
pub(crate) fn exported(
    source: &str,
    tag: &str,
) -> Result<rustgm::lowir::program::Program, Box<dyn std::error::Error>> {
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = crate::resolve::source_provider_config(source, tag, &config)?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    let (mut facts, prepared) = (analysis.facts, analysis.prepared);
    let roots = rustgm::lower::Roots {
        entry: None,
        exports: &[],
    };
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report, &mut facts, roots, prepared,
    )?)
}

pub(crate) fn check(
    root: &Path,
    folder: &Path,
    name: &str,
    program: &rustgm::lowir::program::Program,
    client_source: &str,
    expected: &str,
) -> TestResult {
    check_status(root, folder, name, program, client_source, expected, 0)
}

pub(crate) fn check_status(
    root: &Path,
    folder: &Path,
    name: &str,
    program: &rustgm::lowir::program::Program,
    client_source: &str,
    expected: &str,
    status: i32,
) -> TestResult {
    let mut text = Vec::new();
    rustgm::lowir::writer::write(program, &mut text)?;
    check_lowir(root, folder, name, &text, client_source, expected, status)
}

/// As `check_status`, for the `LowIR` text of a program that the compiler
/// command wrote.
pub(crate) fn check_lowir(
    root: &Path,
    folder: &Path,
    name: &str,
    text: &[u8],
    client_source: &str,
    expected: &str,
    status: i32,
) -> TestResult {
    let ir = folder.join(format!("{name}.lowir"));
    let object = folder.join(format!("{name}.o"));
    let binary = folder.join(format!("{name}.native"));
    let client = folder.join(format!("{name}.c"));
    std::fs::write(&ir, text)?;
    std::fs::write(&client, client_source)?;
    run(
        folder,
        &format!("{name}-backend"),
        Command::new("python3")
            .arg(root.join("scripts/backend.py"))
            .arg("--emit-object")
            .arg(&ir)
            .arg("-o")
            .arg(&object),
    )?;
    run(
        folder,
        &format!("{name}-link"),
        Command::new("cc")
            .arg(&client)
            .arg(&object)
            .arg("-o")
            .arg(&binary),
    )?;
    let output = Command::new(&binary).output()?;
    std::fs::write(
        folder.join(format!("{name}-execute.stdout")),
        &output.stdout,
    )?;
    std::fs::write(
        folder.join(format!("{name}-execute.stderr")),
        &output.stderr,
    )?;
    assert_eq!(output.status.code(), Some(status), "{name}: {output:?}");
    assert!(output.stderr.is_empty(), "{name}: {output:?}");
    assert_eq!(
        std::fs::read(folder.join(format!("{name}-execute.stdout")))?,
        expected.as_bytes(),
        "{name}"
    );
    Ok(())
}
