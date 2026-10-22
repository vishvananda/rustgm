//! Explicit fieldless enum tags are validated only at typed demand.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
const CASES: &[(&str, &str, i32, bool)] = &[
    (
        "u8-tag-read",
        "pa27/tests/spec/enum-bytes/u8-tag-read.rs",
        0,
        true,
    ),
    (
        "u8-tag-write",
        "pa27/tests/spec/enum-bytes/u8-tag-write.rs",
        0,
        true,
    ),
    (
        "invalid-tag",
        "pa27/tests/spec/enum-bytes/invalid-tag.rs",
        1,
        false,
    ),
    (
        "unused-invalid-tag",
        "pa27/tests/spec/enum-bytes/unused-invalid-tag.rs",
        1,
        false,
    ),
    (
        "uninitialized-tag",
        "pa27/tests/spec/enum-bytes/uninitialized-tag.rs",
        1,
        false,
    ),
    (
        "negative-tag-read",
        "pa27/tests/spec/enum-bytes/negative-tag-read.rs",
        0,
        true,
    ),
    (
        "negative-tag-write",
        "pa27/tests/spec/enum-bytes/negative-tag-write.rs",
        0,
        true,
    ),
    (
        "negative-invalid",
        "pa27/tests/spec/enum-bytes/negative-invalid.rs",
        1,
        false,
    ),
    (
        "single-variant",
        "pa27/tests/spec/enum-bytes/single-variant.rs",
        0,
        true,
    ),
    (
        "sparse-valid",
        "pa27/tests/spec/enum-bytes/sparse-valid.rs",
        0,
        true,
    ),
    (
        "sparse-hole",
        "pa27/tests/spec/enum-bytes/sparse-hole.rs",
        1,
        false,
    ),
    (
        "computed-discriminant",
        "pa27/tests/spec/enum-bytes/computed-discriminant.rs",
        0,
        true,
    ),
    (
        "ordinary-enum",
        "pa27/tests/spec/enum-bytes/ordinary-enum.rs",
        0,
        true,
    ),
    (
        "signed-boundaries",
        "pa27/tests/spec/enum-bytes/signed-boundaries.rs",
        0,
        true,
    ),
    (
        "tag-padding",
        "pa27/tests/spec/enum-bytes/tag-padding.rs",
        0,
        true,
    ),
    (
        "pointer-copy",
        "pa28/tests/spec/enum-bytes/pointer-copy.rs",
        0,
        true,
    ),
    (
        "fill-valid",
        "pa28/tests/spec/enum-bytes/fill-valid.rs",
        0,
        true,
    ),
    (
        "fill-invalid",
        "pa28/tests/spec/enum-bytes/fill-invalid.rs",
        1,
        false,
    ),
    (
        "zero-effective-copy",
        "pa28/tests/spec/enum-bytes/zero-effective-copy.rs",
        0,
        true,
    ),
    (
        "invalid-copy-repair",
        "pa28/tests/spec/enum-bytes/invalid-copy-repair.rs",
        0,
        true,
    ),
    (
        "uninitialized-copy-repair",
        "pa28/tests/spec/enum-bytes/uninitialized-copy-repair.rs",
        0,
        true,
    ),
    (
        "invalid-copy-demand",
        "pa28/tests/spec/enum-bytes/invalid-copy-demand.rs",
        1,
        false,
    ),
    (
        "uninitialized-copy-demand",
        "pa28/tests/spec/enum-bytes/uninitialized-copy-demand.rs",
        1,
        false,
    ),
    (
        "nested-family",
        "pa28/tests/spec/enum-bytes/nested-family.rs",
        0,
        true,
    ),
    (
        "distinct-family",
        "pa28/tests/spec/enum-bytes/distinct-family.rs",
        0,
        true,
    ),
    (
        "distinct-invalid-family",
        "pa28/tests/spec/enum-bytes/distinct-invalid-family.rs",
        1,
        false,
    ),
];
#[test]
fn fieldless_enum_byte_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa27/enum-bytes/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for &(name, path, status, _) in CASES {
        checked::check_views_with_rustc_args(
            &root.join(path),
            &out,
            name,
            status,
            &[
                ("--emit-types", status),
                ("--emit-lowir", status),
                ("--emit-constants", status),
            ],
            "",
            &["--cap-lints=allow"],
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn fieldless_enum_byte_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let mut owned = Vec::new();
    for &(name, path, _, valid) in CASES {
        if valid {
            owned.push((name, std::fs::read_to_string(root.join(path))?));
        }
    }
    let cases: Vec<_> = owned
        .iter()
        .map(|(name, text)| (*name, text.as_str(), false))
        .collect();
    native::check_literal_cases("parallel/pa27/enum-bytes/native", &cases)
}
