//! Real source wrapper outcomes use complete CFG proofs and the external native backend.
use crate::provider_traits::{TestResult, lower};
use std::path::{Path, PathBuf};
const CASES: &[(&str, bool)] = &[
    ("manual-header", true),
    ("maybe-header", true),
    ("manual-no-repr", true),
    ("manual-lifetime", true),
    ("manual-const", true),
    ("manual-two-fields", true),
    ("maybe-two-fields", true),
    ("manual-zero", false),
    ("manual-two-generics", false),
    ("maybe-zero", false),
    ("maybe-two-generics", false),
    ("manual-wrong-target", false),
    ("maybe-wrong-target", false),
    ("manual-duplicate", false),
    ("maybe-duplicate", false),
    ("maybe-valid-dangling", true),
    ("manual-nested-valid-dangling", true),
    ("maybe-null", false),
    ("maybe-unaligned", false),
    ("ordinary-dangling", false),
    ("manual-alone-dangling", false),
    ("lookalike-dangling", false),
    ("maybe-invalid-bool", false),
    ("manual-invalid-bool", false),
    ("maybe-extracted-dangling", false),
    ("maybe-sibling-memo", false),
    ("maybe-deref", false),
    ("maybe-borrow-escape", false),
    ("drop-pair", true),
    ("manual-own-destructor", true),
    ("const-layout", true),
    ("manual-move-out", true),
    ("combined-roles", false),
];
const NATIVE: &[(&str, &str)] = &[
    ("drop-pair", "23 7 0 1 1\n"),
    ("manual-own-destructor", "0 7\n"),
    ("manual-move-out", "4 7\n"),
    ("const-layout", "7 8 8\n"),
];
fn root() -> TestResult<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?
        .to_owned())
}
fn fixture(name: &str, extension: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(root()?.join(format!(
        "pa31/tests/spec/source-wrappers/{name}.{extension}"
    )))?)
}
#[test]
fn source_wrapper_outcomes_preserve_real_fields_const_validity_and_full_borrow_proofs() -> TestResult
{
    assert_eq!(CASES.len(), 33);
    for &(name, accepted) in CASES {
        let result = lower(&fixture(name, "rs")?);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {:?}",
            result.err().map(|error| error.to_string())
        );
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn source_wrappers_native_preserve_actual_destructors_layout_and_owned_field_moves() -> TestResult {
    assert_eq!(NATIVE.len(), 4);
    let root = root()?;
    let folder = root
        .join("obj/parallel/source-wrappers-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for &(name, expected) in NATIVE {
        crate::ffi_program::check(
            &root,
            &folder,
            name,
            &lower(&fixture(name, "rs")?)?,
            &fixture(name, "c")?,
            expected,
        )?;
    }
    Ok(())
}
