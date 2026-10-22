//! Canonical pointer arithmetic preserves zero-offset identity and access restrictions.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
#[path = "offset_intrinsics/measurements.rs"]
mod measurements;
const CASES: &[(&str, &str, i32)] = &[
    (
        "original-add",
        "pa28/tests/spec/byte-access/library-add-offset.rs",
        0,
    ),
    ("const-read", "pa28/tests/spec/pointer-add/const-read.rs", 0),
    (
        "mutable-store",
        "pa28/tests/spec/pointer-add/mutable-store.rs",
        0,
    ),
    (
        "field-relative",
        "pa28/tests/spec/pointer-add/field-relative.rs",
        0,
    ),
    (
        "const-generic",
        "pa28/tests/spec/pointer-add/const-generic.rs",
        0,
    ),
    (
        "mutable-generic",
        "pa28/tests/spec/pointer-add/mutable-generic.rs",
        0,
    ),
    ("ufcs-const", "pa28/tests/spec/pointer-add/ufcs-const.rs", 0),
    (
        "ufcs-mutable",
        "pa28/tests/spec/pointer-add/ufcs-mutable.rs",
        0,
    ),
    (
        "reified-const",
        "pa28/tests/spec/pointer-add/reified-const.rs",
        0,
    ),
    (
        "reified-mutable",
        "pa28/tests/spec/pointer-add/reified-mutable.rs",
        0,
    ),
    (
        "const-reified-forbidden",
        "pa28/tests/spec/pointer-add/const-reified-forbidden.rs",
        1,
    ),
    (
        "owned-runtime",
        "pa28/tests/spec/pointer-add/owned-runtime.rs",
        0,
    ),
    (
        "evaluate-once",
        "pa28/tests/spec/pointer-add/evaluate-once.rs",
        0,
    ),
    (
        "zero-static",
        "pa28/tests/spec/pointer-add/zero-static.rs",
        0,
    ),
    (
        "uninitialized-bytes-address-only",
        "pa28/tests/spec/pointer-add/uninitialized-bytes-address-only.rs",
        0,
    ),
    (
        "zero-numeric",
        "pa28/tests/spec/pointer-add/zero-numeric.rs",
        0,
    ),
    ("zero-null", "pa28/tests/spec/pointer-add/zero-null.rs", 0),
    ("zst-huge", "pa28/tests/spec/pointer-add/zst-huge.rs", 0),
    (
        "uninhabited-zst",
        "pa28/tests/spec/pointer-add/uninhabited-zst.rs",
        0,
    ),
    ("live-zst", "pa28/tests/spec/pointer-add/live-zst.rs", 0),
    (
        "expired-zero-transient",
        "pa28/tests/spec/pointer-add/expired-zero-transient.rs",
        0,
    ),
    (
        "expired-nonzero",
        "pa28/tests/spec/pointer-add/expired-nonzero.rs",
        1,
    ),
    (
        "zero-escape",
        "pa28/tests/spec/pointer-add/zero-escape.rs",
        1,
    ),
    ("one-past", "pa28/tests/spec/pointer-add/one-past.rs", 0),
    ("beyond-end", "pa28/tests/spec/pointer-add/beyond-end.rs", 1),
    (
        "one-past-dereference",
        "pa28/tests/spec/pointer-add/one-past-dereference.rs",
        1,
    ),
    (
        "multiply-overflow",
        "pa28/tests/spec/pointer-add/multiply-overflow.rs",
        1,
    ),
    (
        "isize-overflow",
        "pa28/tests/spec/pointer-add/isize-overflow.rs",
        1,
    ),
    (
        "packed-zero-address",
        "pa28/tests/spec/pointer-add/packed-zero-address.rs",
        0,
    ),
    (
        "packed-unaligned-read",
        "pa28/tests/spec/pointer-add/packed-unaligned-read.rs",
        0,
    ),
    (
        "packed-aligned-read-reject",
        "pa28/tests/spec/pointer-add/packed-aligned-read-reject.rs",
        1,
    ),
    (
        "shared-derived-store",
        "pa28/tests/spec/pointer-add/shared-derived-store.rs",
        1,
    ),
    (
        "unsafe-required",
        "pa28/tests/spec/pointer-add/unsafe-required.rs",
        1,
    ),
    (
        "wrong-count",
        "pa28/tests/spec/pointer-add/wrong-count.rs",
        1,
    ),
    (
        "method-turbofish-reject",
        "pa28/tests/spec/pointer-add/method-turbofish-reject.rs",
        1,
    ),
    (
        "unsized-receiver",
        "pa28/tests/spec/pointer-add/unsized-receiver.rs",
        1,
    ),
    (
        "ordinary-shadow",
        "pa28/tests/spec/pointer-add/ordinary-shadow.rs",
        0,
    ),
    (
        "trait-sized-shadow",
        "pa28/tests/spec/pointer-add/trait-sized-shadow.rs",
        0,
    ),
    (
        "trait-unsized-shadow",
        "pa28/tests/spec/pointer-add/trait-unsized-shadow.rs",
        1,
    ),
    (
        "trait-explicit-ufcs",
        "pa28/tests/spec/pointer-add/trait-explicit-ufcs.rs",
        0,
    ),
    ("alias-ufcs", "pa28/tests/spec/pointer-add/alias-ufcs.rs", 0),
    (
        "generic-ufcs",
        "pa28/tests/spec/pointer-add/generic-ufcs.rs",
        0,
    ),
    (
        "trait-unsized-unsafe",
        "pa28/tests/spec/pointer-add/trait-unsized-unsafe.rs",
        1,
    ),
];
#[test]
fn pointer_add_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa28/pointer-add-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for &(name, path, expected) in CASES {
        checked::check_views_with_rustc_args(
            &root.join(path),
            &out,
            name,
            expected,
            &[
                ("--emit-types", expected),
                ("--emit-lowir", expected),
                ("--emit-constants", expected),
            ],
            "",
            &["--cap-lints=allow"],
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn pointer_add_native_generic_reified_and_owned_values() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let names = [
        "original-add",
        "const-read",
        "mutable-store",
        "field-relative",
        "const-generic",
        "mutable-generic",
        "ufcs-const",
        "ufcs-mutable",
        "reified-const",
        "reified-mutable",
        "owned-runtime",
        "evaluate-once",
        "zero-static",
        "uninitialized-bytes-address-only",
        "live-zst",
        "packed-zero-address",
        "packed-unaligned-read",
        "ordinary-shadow",
        "trait-sized-shadow",
        "trait-explicit-ufcs",
        "alias-ufcs",
        "generic-ufcs",
    ];
    let mut owned = Vec::new();
    for name in names {
        let (_, path, _) = CASES
            .iter()
            .find(|case| case.0 == name)
            .ok_or("native case")?;
        owned.push((name, std::fs::read_to_string(root.join(path))?));
    }
    let cases: Vec<_> = owned
        .iter()
        .map(|(name, text)| (*name, text.as_str(), false))
        .collect();
    native::check_literal_cases("parallel/pa28/pointer-add-native", &cases)
}
