//! The unchanged independently settled source bytes supply each native bool result.
use super::{POSITIVE, SCALING, TestResult, lower, read_source};
use crate::provider_traits::native::run;
use std::{path::Path, process::Command};
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn source_needs_drop_native_wrappers_and_fallbacks() -> TestResult {
    check_cases(POSITIVE)
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn source_needs_drop_scaling_native_answers() -> TestResult {
    check_cases(SCALING)
}
fn check_cases(cases: &[(&str, &str, u8)]) -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/needs-drop-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for &(name, path, expected) in cases {
        let source = read_source(path)?;
        if let Err(error) = check_one(root, &folder, name, &source, expected) {
            failures.push(format!("{name}: {error}"));
        } else {
            println!("{name}: native observed {expected}");
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn check_one(root: &Path, folder: &Path, name: &str, source: &str, expected: u8) -> TestResult {
    let program = lower(source).map_err(|e| format!("{name}: {e}"))?;
    let ir = folder.join(format!("{name}.lowir"));
    let object = folder.join(format!("{name}.o"));
    let binary = folder.join(format!("{name}.native"));
    let client = folder.join(format!("{name}.c"));
    let mut text = Vec::new();
    rustgm::lowir::writer::write(&program, &mut text)?;
    std::fs::write(&ir, text)?;
    std::fs::write(folder.join(format!("{name}.rs")), source)?;
    std::fs::write(
        &client,
        "#include <stdio.h>\nextern unsigned char observe(void);int main(void){printf(\"%u\\n\",(unsigned)observe());return 0;}\n",
    )?;
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
        format!("{expected}\n").as_bytes(),
        "{name}"
    );
    Ok(())
}
