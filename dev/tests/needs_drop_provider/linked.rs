//! The unchanged independently settled source bytes supply each native bool result.
use super::{POSITIVE, SCALING, TestResult, lower, read_source};
use std::path::Path;
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
    std::fs::write(folder.join(format!("{name}.rs")), source)?;
    crate::ffi_program::check(
        root,
        folder,
        name,
        &program,
        "#include <stdio.h>\nextern unsigned char observe(void);int main(void){printf(\"%u\\n\",(unsigned)observe());return 0;}\n",
        &format!("{expected}\n"),
    )
}
