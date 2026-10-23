//! Opaque byte storage consumes checked physical heads, never source type equality.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
type Case = (&'static str, &'static str, i32, i32, bool, &'static str);
const CASES: &[Case] = &[
    (
        "opaque-scalar-byte-check",
        "pa27/tests/spec/opaque-byte-backing/opaque-scalar-byte-check.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "opaque-record-offset-check",
        "pa28/tests/spec/opaque-byte-backing/opaque-record-offset-check.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "opaque-generic-heads-check",
        "pa27/tests/spec/opaque-byte-backing/opaque-generic-heads-check.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "opaque-union-roundtrip-check",
        "pa27/tests/spec/opaque-byte-backing/opaque-union-roundtrip-check.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "opaque-closure-roundtrip-check",
        "pa27/tests/spec/opaque-byte-backing/opaque-closure-roundtrip-check.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "opaque-invalid-bool",
        "pa27/tests/spec/opaque-byte-backing/opaque-invalid-bool.rs",
        1,
        1,
        false,
        "invalid bool",
    ),
    (
        "opaque-reference-word-check",
        "pa27/tests/spec/opaque-byte-backing/opaque-reference-word-check.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "opaque-reference-direct",
        "pa27/tests/spec/opaque-byte-backing/opaque-reference-direct.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "opaque-shared-restriction",
        "pa27/tests/spec/opaque-byte-backing/opaque-shared-restriction.rs",
        1,
        1,
        false,
        "constant raw pointer write is derived from a shared reference",
    ),
    (
        "opaque-owned-transfer-concrete",
        "pa28/tests/spec/opaque-byte-backing/opaque-owned-transfer-concrete.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "opaque-bounds",
        "pa28/tests/spec/opaque-byte-backing/opaque-bounds.rs",
        1,
        1,
        false,
        "bounds",
    ),
    (
        "opaque-expired-pointer",
        "pa27/tests/spec/opaque-byte-backing/opaque-expired-pointer.rs",
        1,
        1,
        false,
        "expired",
    ),
    (
        "opaque-uninitialized-read",
        "pa27/tests/spec/opaque-byte-backing/opaque-uninitialized-read.rs",
        1,
        1,
        false,
        "bounds",
    ),
];
const SCALING: &[Case] = &[
    (
        "opaque-wide-8-repeat-8",
        "pa27/tests/spec/opaque-byte-backing/opaque-wide-8-repeat-8.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "opaque-wide-32-repeat-32",
        "pa27/tests/spec/opaque-byte-backing/opaque-wide-32-repeat-32.rs",
        0,
        0,
        true,
        "",
    ),
    (
        "opaque-wide-128-repeat-128",
        "pa27/tests/spec/opaque-byte-backing/opaque-wide-128-repeat-128.rs",
        0,
        0,
        true,
        "",
    ),
];
fn source_cases(cases: &[Case], label: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa27/opaque-string-backing/{label}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for &(name, path, rust, expected, _, reason) in cases {
        checked::check_views_with_rustc_args(
            &root.join(path),
            &out,
            name,
            rust,
            &[
                ("--emit-types", expected),
                ("--emit-constants", expected),
                ("--emit-lowir", expected),
            ],
            reason,
            &["--cap-lints=allow"],
        )?;
    }
    Ok(())
}
fn native_cases(cases: &[Case], label: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let mut owned = Vec::new();
    for &(name, path, _, _, valid, _) in cases {
        if valid {
            owned.push((name, std::fs::read_to_string(root.join(path))?));
        }
    }
    let cases: Vec<_> = owned
        .iter()
        .map(|(name, text)| (*name, text.as_str(), false))
        .collect();
    native::check_literal_cases(label, &cases)
}
#[test]
fn opaque_storage_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    source_cases(CASES, "source")
}
#[test]
#[ignore = "requires pinned Rust and CPPGM backend; literal source/value comparisons"]
fn opaque_storage_native_values() -> Result<(), Box<dyn std::error::Error>> {
    native_cases(CASES, "parallel/pa27/opaque-string-backing/native")
}
#[test]
fn opaque_storage_scaling_sources() -> Result<(), Box<dyn std::error::Error>> {
    source_cases(SCALING, "scaling-source")
}
#[test]
#[ignore = "requires pinned Rust and CPPGM backend; literal source/value comparisons"]
fn opaque_storage_scaling_native() -> Result<(), Box<dyn std::error::Error>> {
    native_cases(
        SCALING,
        "parallel/pa27/opaque-string-backing/scaling-native",
    )
}
