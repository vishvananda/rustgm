//! A dynamic helper calls the table's complete concrete cleanup, including null.
use super::TestResult;
use crate::provider_traits::lower;
use std::path::Path;
const POSITIVE: &[(&str, &str)] = &[
    ("no-exposure", "0\n"),
    ("header-only-no-helper", "7\n"),
    ("generic-paid-parent", "1\n"),
    ("original-dyn-one", "1\n"),
    ("complete-empty-null-entry", "0\n"),
    ("nonempty-registry-null-entry", "0\n"),
    ("different-concrete-drop-entries", "23\n"),
    ("whole-cleanup-method-before-fields", "42\n"),
    ("actual-role-and-ordinary-lookalike", "0\n"),
    ("reified-static-object-bound", "1\n"),
];
fn fixture(name: &str, extension: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../pa31/tests/spec/drop-glue-dyn/{name}.{extension}"
        )),
    )?)
}
#[test]
fn actual_dyn_helpers_keep_null_entries_and_complete_concrete_cleanup() -> TestResult {
    let mut failures = Vec::new();
    for &(name, _) in POSITIVE {
        if let Err(error) = lower(&fixture(name, "rs")?) {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn actual_dyn_helpers_keep_borrow_trait_and_object_binder_opposites() -> TestResult {
    for name in [
        "shared-object-is-not-unique",
        "scoped-reference-cannot-escape",
        "unpaid-object-trait",
        "reified-wrong-object-generality",
    ] {
        assert!(lower(&fixture(name, "rs")?).is_err(), "{name}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn actual_dyn_helpers_native_dispatch_complete_cleanup_and_ordinary_lookalikes() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/dyn-glue-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for &(name, expected) in POSITIVE {
        let checked = || -> TestResult {
            let program = lower(&fixture(name, "rs")?).map_err(|e| format!("{name}: {e}"))?;
            crate::ffi_program::check(
                root,
                &folder,
                name,
                &program,
                &fixture(name, "c")?,
                expected,
            )?;
            assert!(std::fs::read(folder.join(format!("{name}-execute.stderr")))?.is_empty());
            Ok(())
        };
        if let Err(error) = checked() {
            failures.push(format!("{name}: {error}"));
        } else {
            println!("{name}: native observed {expected:?}");
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
