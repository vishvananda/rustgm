//! Shared checked-LowIR execution with literal independently declared C observers.
use crate::provider_traits::{TestResult, native::run};
use std::{path::Path, process::Command};

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
    let ir = folder.join(format!("{name}.lowir"));
    let object = folder.join(format!("{name}.o"));
    let binary = folder.join(format!("{name}.native"));
    let client = folder.join(format!("{name}.c"));
    let mut text = Vec::new();
    rustgm::lowir::writer::write(program, &mut text)?;
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
