//! C tags use retained ABI facts; default multi-variant positives remain required gaps.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
const CASES: &[(&str, &str, i32, i32, bool, &str)] = &[
    (
        "fieldless-u32",
        "pa27/tests/spec/c-enum-bytes/fieldless-u32.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "fieldless-i32",
        "pa27/tests/spec/c-enum-bytes/fieldless-i32.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "fieldless-u64",
        "pa27/tests/spec/c-enum-bytes/fieldless-u64.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "fieldless-i64",
        "pa27/tests/spec/c-enum-bytes/fieldless-i64.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "tuple-from-bytes",
        "pa27/tests/spec/c-enum-bytes/tuple-from-bytes.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "inactive-invalid-bool",
        "pa27/tests/spec/c-enum-bytes/inactive-invalid-bool.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "active-invalid-bool",
        "pa27/tests/spec/c-enum-bytes/active-invalid-bool.rs",
        1,
        1,
        false,
        "invalid bool representation",
    ),
    (
        "invalid-tag",
        "pa27/tests/spec/c-enum-bytes/invalid-tag.rs",
        1,
        1,
        false,
        "invalid discriminant",
    ),
    (
        "partial-tag",
        "pa27/tests/spec/c-enum-bytes/partial-tag.rs",
        1,
        1,
        false,
        "uninitialized bytes",
    ),
    (
        "active-uninitialized",
        "pa27/tests/spec/c-enum-bytes/active-uninitialized.rs",
        1,
        1,
        false,
        "uninitialized bytes",
    ),
    (
        "empty-uninitialized-inactive",
        "pa27/tests/spec/c-enum-bytes/empty-uninitialized-inactive.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "record-variant",
        "pa27/tests/spec/c-enum-bytes/record-variant.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "reference-field",
        "pa27/tests/spec/c-enum-bytes/reference-field.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "slice-field",
        "pa27/tests/spec/c-enum-bytes/slice-field.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "nested-array",
        "pa27/tests/spec/c-enum-bytes/nested-array.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "generic-closed-fields",
        "pa28/tests/spec/c-enum-bytes/generic-closed-fields.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "fill-then-match",
        "pa28/tests/spec/c-enum-bytes/fill-then-match.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "old-field-pointer",
        "pa28/tests/spec/c-enum-bytes/old-field-pointer.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "explicit-alignment",
        "pa27/tests/spec/c-enum-bytes/explicit-alignment.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "copy-repair",
        "pa28/tests/spec/c-enum-bytes/copy-repair.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "copy-invalid",
        "pa28/tests/spec/c-enum-bytes/copy-invalid.rs",
        1,
        1,
        false,
        "invalid discriminant",
    ),
    (
        "default-layout-still-required",
        "pa27/tests/spec/default-enum-bytes/default-layout-still-required.rs",
        0,
        1,
        false,
        "requires a supported closed byte family",
    ),
    (
        "tagless-still-required",
        "pa28/tests/spec/default-enum-bytes/tagless-still-required.rs",
        0,
        0,
        true,
        "",
    ),
];
#[test]
fn c_enum_byte_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa27/c-enum-bytes/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for &(name, path, rust, frontend, _, reason) in CASES {
        checked::check_views_with_rustc_args(
            &root.join(path),
            &out,
            name,
            rust,
            &[
                ("--emit-types", frontend),
                ("--emit-lowir", frontend),
                ("--emit-constants", frontend),
            ],
            reason,
            &["--cap-lints=allow"],
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn c_enum_byte_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let mut owned = Vec::new();
    for &(name, path, _, _, valid, _) in CASES {
        if valid {
            owned.push((name, std::fs::read_to_string(root.join(path))?));
        }
    }
    let cases: Vec<_> = owned
        .iter()
        .map(|(name, text)| (*name, text.as_str(), false))
        .collect();
    native::check_literal_cases("parallel/pa27/c-enum-bytes/native", &cases)
}
