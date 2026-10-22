//! Explicit payload tags preserve active field validity and physical pointer identity.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
const CASES: &[(&str, &str, i32, bool)] = &[
    (
        "tuple-roundtrip",
        "pa27/tests/spec/payload-enum-bytes/tuple-roundtrip.rs",
        0,
        true,
    ),
    (
        "tuple-from-bytes",
        "pa27/tests/spec/payload-enum-bytes/tuple-from-bytes.rs",
        0,
        true,
    ),
    (
        "inline-boolean-offset",
        "pa27/tests/spec/payload-enum-bytes/inline-boolean-offset.rs",
        0,
        true,
    ),
    (
        "c-boolean-offset",
        "pa27/tests/spec/payload-enum-bytes/c-boolean-offset.rs",
        0,
        true,
    ),
    (
        "active-invalid-boolean",
        "pa27/tests/spec/payload-enum-bytes/active-invalid-boolean.rs",
        1,
        false,
    ),
    (
        "inactive-invalid-boolean",
        "pa27/tests/spec/payload-enum-bytes/inactive-invalid-boolean.rs",
        0,
        true,
    ),
    (
        "active-uninitialized-payload",
        "pa27/tests/spec/payload-enum-bytes/active-uninitialized-payload.rs",
        1,
        false,
    ),
    (
        "invalid-tag",
        "pa27/tests/spec/payload-enum-bytes/invalid-tag.rs",
        1,
        false,
    ),
    (
        "empty-uninitialized-inactive",
        "pa27/tests/spec/payload-enum-bytes/empty-uninitialized-inactive.rs",
        0,
        true,
    ),
    (
        "record-variant",
        "pa27/tests/spec/payload-enum-bytes/record-variant.rs",
        0,
        true,
    ),
    (
        "wider-tag-fields",
        "pa27/tests/spec/payload-enum-bytes/wider-tag-fields.rs",
        0,
        true,
    ),
    (
        "nested-record",
        "pa27/tests/spec/payload-enum-bytes/nested-record.rs",
        0,
        true,
    ),
    (
        "reference-field",
        "pa27/tests/spec/payload-enum-bytes/reference-field.rs",
        0,
        true,
    ),
    (
        "inactive-reference",
        "pa27/tests/spec/payload-enum-bytes/inactive-reference.rs",
        0,
        true,
    ),
    (
        "function-field",
        "pa27/tests/spec/payload-enum-bytes/function-field.rs",
        0,
        true,
    ),
    (
        "fill-then-match",
        "pa28/tests/spec/payload-enum-bytes/fill-then-match.rs",
        0,
        true,
    ),
    (
        "field-write-after-conversion",
        "pa28/tests/spec/payload-enum-bytes/field-write-after-conversion.rs",
        0,
        true,
    ),
    (
        "tag-change-and-repair",
        "pa28/tests/spec/payload-enum-bytes/tag-change-and-repair.rs",
        0,
        true,
    ),
    (
        "inactive-padding-copy",
        "pa28/tests/spec/payload-enum-bytes/inactive-padding-copy.rs",
        0,
        true,
    ),
    (
        "noncopy-fill",
        "pa28/tests/spec/payload-enum-bytes/noncopy-fill.rs",
        0,
        true,
    ),
    (
        "old-field-pointer",
        "pa28/tests/spec/payload-enum-bytes/old-field-pointer.rs",
        0,
        true,
    ),
];
#[test]
fn payload_enum_byte_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa27/payload-enum-bytes/source-{}",
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
fn payload_enum_byte_native_values() -> Result<(), Box<dyn std::error::Error>> {
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
    native::check_literal_cases("parallel/pa27/payload-enum-bytes/native", &cases)
}
