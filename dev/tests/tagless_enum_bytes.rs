//! Unique no-tag enums retain exact variant and physical field identities.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
const CASES: &[(&str, &str, i32, i32, bool, &str)] = &[
    (
        "union-roundtrip",
        "pa27/tests/spec/tagless-enum-bytes/union-roundtrip.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "from-bytes",
        "pa27/tests/spec/tagless-enum-bytes/from-bytes.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "partial-initialization",
        "pa27/tests/spec/tagless-enum-bytes/partial-initialization.rs",
        1,
        1,
        false,
        "uninitialized bytes",
    ),
    (
        "record-fields",
        "pa27/tests/spec/tagless-enum-bytes/record-fields.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "nested-enums",
        "pa27/tests/spec/tagless-enum-bytes/nested-enums.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "reference-field",
        "pa27/tests/spec/tagless-enum-bytes/reference-field.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "slice-field",
        "pa27/tests/spec/tagless-enum-bytes/slice-field.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "wrong-reference-referent",
        "pa27/tests/spec/tagless-enum-bytes/wrong-reference-referent.rs",
        1,
        1,
        false,
        "outside allocation bounds",
    ),
    (
        "escaping-reference",
        "pa28/tests/spec/tagless-enum-bytes/escaping-reference.rs",
        1,
        1,
        false,
        "required lifetime does not outlive its target",
    ),
    (
        "noncopy-fill",
        "pa28/tests/spec/tagless-enum-bytes/noncopy-fill.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "generic-instances",
        "pa28/tests/spec/tagless-enum-bytes/generic-instances.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "projection-before",
        "pa28/tests/spec/tagless-enum-bytes/projection-before.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "projection-after",
        "pa28/tests/spec/tagless-enum-bytes/projection-after.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "zero-sized",
        "pa28/tests/spec/tagless-enum-bytes/zero-sized.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "zero-array-field",
        "pa27/tests/spec/tagless-enum-bytes/zero-array-field.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "padding-transport",
        "pa28/tests/spec/tagless-enum-bytes/padding-transport.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "padding-read",
        "pa28/tests/spec/tagless-enum-bytes/padding-read.rs",
        1,
        1,
        false,
        "uninitialized bytes",
    ),
    (
        "original-const-return",
        "pa28/tests/spec/default-enum-bytes/tagless-still-required.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "default-multiple-still-required",
        "pa27/tests/spec/default-enum-bytes/default-layout-still-required.rs",
        0,
        0,
        true,
        "",
    ),
];
#[test]
fn tagless_enum_byte_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa27/tagless-enum-bytes/source-{}",
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
fn tagless_enum_byte_native_values() -> Result<(), Box<dyn std::error::Error>> {
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
    native::check_literal_cases("parallel/pa27/tagless-enum-bytes/native", &cases)
}
