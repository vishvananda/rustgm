//! Ordinary enum backing uses retained discriminants, selected payloads and physical offsets.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
const CASES: &[(&str, &str, i32, i32, bool, &str)] = &[
    (
        "static-union-image",
        "pa27/tests/spec/default-tag-enum-bytes/static-union-image.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "ordinary-closure-after-image",
        "pa27/tests/spec/default-tag-enum-bytes/ordinary-closure-after-image.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "ordinary-closure-child",
        "pa27/tests/spec/default-tag-enum-bytes/ordinary-closure-child.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "borrowed-closure-child",
        "pa27/tests/spec/default-tag-enum-bytes/borrowed-closure-child.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "fieldless-roundtrip",
        "pa27/tests/spec/default-tag-enum-bytes/fieldless-roundtrip.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "signed-roundtrip",
        "pa27/tests/spec/default-tag-enum-bytes/signed-roundtrip.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "payload-variants",
        "pa27/tests/spec/default-tag-enum-bytes/payload-variants.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "record-fields",
        "pa27/tests/spec/default-tag-enum-bytes/record-fields.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "reference-payload",
        "pa27/tests/spec/default-tag-enum-bytes/reference-payload.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "generic-instances",
        "pa27/tests/spec/default-tag-enum-bytes/generic-instances.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "whole-array-copy",
        "pa28/tests/spec/default-tag-enum-bytes/whole-array-copy.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "field-pointer-fill",
        "pa28/tests/spec/default-tag-enum-bytes/field-pointer-fill.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "published-enum",
        "pa27/tests/spec/default-tag-enum-bytes/published-enum.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "invalid-tag",
        "pa27/tests/spec/default-tag-enum-bytes/invalid-tag.rs",
        1,
        1,
        false,
        "invalid discriminant",
    ),
    (
        "original-default-multiple",
        "pa27/tests/spec/default-enum-bytes/default-layout-still-required.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "empty-inactive-uninitialized",
        "pa27/tests/spec/default-tag-enum-bytes/empty-inactive-uninitialized.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "active-uninitialized",
        "pa27/tests/spec/default-tag-enum-bytes/active-uninitialized.rs",
        1,
        1,
        false,
        "uninitialized",
    ),
    (
        "field-before-tag-change",
        "pa28/tests/spec/default-tag-enum-bytes/field-before-tag-change.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "selected-invalid-bool",
        "pa28/tests/spec/default-tag-enum-bytes/selected-invalid-bool.rs",
        1,
        1,
        false,
        "invalid bool representation",
    ),
    (
        "selected-null-reference",
        "pa28/tests/spec/default-tag-enum-bytes/selected-null-reference.rs",
        1,
        1,
        false,
        "symbolic pointer read requires one complete provenance word",
    ),
    (
        "slice-payload",
        "pa27/tests/spec/default-tag-enum-bytes/slice-payload.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "function-payload",
        "pa27/tests/spec/default-tag-enum-bytes/function-payload.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "nested-variants",
        "pa27/tests/spec/default-tag-enum-bytes/nested-variants.rs",
        0,
        0,
        true,
        "",
    ),
];
#[test]
fn default_enum_byte_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa27/default-tag-enum-bytes/source-{}",
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
fn default_enum_layout_specific_bool_observation_is_not_a_rust_rejection_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa27/default-tag-enum-bytes/observation-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    checked::check_views_with_rustc_args(
        &root.join("pa28/tests/spec/default-tag-enum-bytes/active-invalid-bool.rs"),
        &out,
        "bool-layout",
        0,
        &[
            ("--emit-types", 1),
            ("--emit-lowir", 1),
            ("--emit-constants", 1),
        ],
        "invalid bool representation",
        &["--cap-lints=allow"],
    )
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend; own-encoding values, no repr(Rust) ABI equality claim"]
fn default_enum_byte_native_values() -> Result<(), Box<dyn std::error::Error>> {
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
    native::check_literal_cases("parallel/pa27/default-tag-enum-bytes/native", &cases)
}
