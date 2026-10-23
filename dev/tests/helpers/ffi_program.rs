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
    run(
        folder,
        &format!("{name}-execute"),
        &mut Command::new(&binary),
    )?;
    assert_eq!(
        std::fs::read(folder.join(format!("{name}-execute.stdout")))?,
        expected.as_bytes(),
        "{name}"
    );
    Ok(())
}
