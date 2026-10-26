//! Retained closure environments use typed captures, never recovered callable identities.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
type Case = (
    &'static str,
    &'static str,
    i32,
    [i32; 3],
    bool,
    &'static str,
);
const CASES: &[Case] = &[
    (
        "original-explicit-closure",
        "pa27/tests/spec/closure-byte-backing/original-explicit-closure.rs",
        0,
        [0, 0, 0],
        true,
        "",
    ),
    (
        "owned-capture",
        "pa27/tests/spec/closure-byte-backing/owned-capture.rs",
        0,
        [0, 0, 0],
        true,
        "",
    ),
    (
        "shared-capture",
        "pa27/tests/spec/closure-byte-backing/shared-capture.rs",
        0,
        [0, 0, 0],
        true,
        "",
    ),
    (
        "mixed-captures",
        "pa27/tests/spec/closure-byte-backing/mixed-captures.rs",
        0,
        [0, 0, 0],
        true,
        "",
    ),
    (
        "zero-sized-capture",
        "pa27/tests/spec/closure-byte-backing/zero-sized-capture.rs",
        0,
        [0, 0, 0],
        true,
        "",
    ),
    (
        "mutable-transient",
        "pa27/tests/spec/closure-byte-backing/mutable-transient.rs",
        0,
        [0, 0, 0],
        true,
        "",
    ),
    (
        "copied-owned",
        "pa28/tests/spec/closure-byte-backing/copied-owned.rs",
        0,
        [0, 0, 0],
        true,
        "",
    ),
    (
        "copied-reference",
        "pa28/tests/spec/closure-byte-backing/copied-reference.rs",
        0,
        [0, 0, 0],
        true,
        "",
    ),
    (
        "invalid-bool",
        "pa28/tests/spec/closure-byte-backing/invalid-bool.rs",
        1,
        [0, 1, 1],
        false,
        "invalid bool",
    ),
    (
        "uninitialized-capture",
        "pa27/tests/spec/closure-byte-backing/uninitialized-capture.rs",
        1,
        [0, 1, 1],
        false,
        "uninitialized",
    ),
    (
        "null-reference",
        "pa28/tests/spec/closure-byte-backing/null-reference.rs",
        1,
        [0, 1, 1],
        false,
        "numeric references, function or wide-pointer words require supported validity proof",
    ),
    (
        "local-escape",
        "pa27/tests/spec/closure-byte-backing/local-escape.rs",
        1,
        [1, 1, 1],
        false,
        "required lifetime",
    ),
    (
        "owned-destructor-raw-transfer",
        "pa28/tests/spec/closure-byte-backing/owned-destructor-raw-transfer.rs",
        0,
        [0; 3],
        true,
        "",
    ),
    (
        "mutable-owned-state",
        "pa27/tests/spec/closure-byte-backing/mutable-owned-state.rs",
        0,
        [0; 3],
        true,
        "",
    ),
    (
        "invalid-bool-named",
        "pa28/tests/spec/closure-byte-backing/invalid-bool-named.rs",
        1,
        [1; 3],
        false,
        "invalid bool",
    ),
    (
        "distinct-closure-identity",
        "pa27/tests/spec/closure-byte-backing/distinct-closure-identity.rs",
        0,
        [0; 3],
        true,
        "",
    ),
    (
        "distinct-capture-order",
        "pa27/tests/spec/closure-byte-backing/distinct-capture-order.rs",
        0,
        [0; 3],
        true,
        "",
    ),
];
#[test]
fn closure_byte_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa27/closure-byte-backing/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for &(name, path, status, views, _, reason) in CASES {
        checked::check_views_with_rustc_args(
            &root.join(path),
            &out,
            name,
            status,
            &[
                ("--emit-types", views[0]),
                ("--emit-lowir", views[1]),
                ("--emit-constants", views[2]),
            ],
            reason,
            &["--cap-lints=allow"],
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned Rust and CPPGM backend; literal source/value comparisons"]
fn closure_byte_native_values() -> Result<(), Box<dyn std::error::Error>> {
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
    native::check_literal_cases("parallel/pa27/closure-byte-backing/native", &cases)
}
