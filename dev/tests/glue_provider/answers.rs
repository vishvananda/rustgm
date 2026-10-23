//! C observers own the borrowed storage and never create a second Rust owner.
use super::{POSITIVE, TestResult, lower, source};
use std::path::Path;

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn actual_source_glue_native_values_order_and_ownership() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/glue-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for &(name, expected) in POSITIVE {
        if let Err(error) = check_one(root, &folder, name, expected) {
            failures.push(format!("{name}: {error}"));
        } else {
            println!("{name}: native observed {expected:?}");
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
fn check_one(root: &Path, folder: &Path, name: &str, expected: &str) -> TestResult {
    let program = lower(&source(name, "rs")?).map_err(|e| format!("{name}: {e}"))?;
    std::fs::write(folder.join(format!("{name}.rs")), source(name, "rs")?)?;
    crate::ffi_program::check(root, folder, name, &program, &source(name, "c")?, expected)
}
