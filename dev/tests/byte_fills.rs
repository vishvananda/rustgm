//! Byte fills preserve raw access authority and defer typed scalar interpretation.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
#[path = "byte_fills/work.rs"]
mod work;
const CASES: &[(&str, i32, i32, &str)] = &[
    ("zero-shared", 0, 0, ""),
    ("zero-expired", 0, 0, ""),
    ("one-expired", 1, 1, "expired storage"),
    ("one-shared", 1, 1, "derived from a shared reference"),
    ("zero-static", 0, 0, ""),
    ("one-static", 1, 1, "cannot write static storage"),
    ("zst-max-count", 0, 1, "missing value resolution"),
    (
        "zst-null",
        0,
        1,
        "constant counted numeric pointer access is not implemented",
    ),
    (
        "null-zero-default",
        1,
        1,
        "constant counted numeric pointer access is not implemented",
    ),
    (
        "null-zero-allow",
        0,
        1,
        "constant counted numeric pointer access is not implemented",
    ),
    ("numeric-zst", 0, 1, "missing value resolution"),
    (
        "numeric-misaligned-zero",
        0,
        1,
        "constant counted numeric pointer access is not implemented",
    ),
    (
        "packed-field-zero",
        0,
        1,
        "constant counted pointer is unaligned",
    ),
    ("packed-base-zero", 0, 0, ""),
    ("zst-expired", 0, 1, "missing value resolution"),
    ("count-overflow", 1, 1, "missing value resolution"),
    ("out-of-bounds", 1, 1, "constant counted transfer exceeds"),
    ("byte-scalar", 0, 0, ""),
    ("invalid-bool-transport", 0, 0, ""),
    ("invalid-bool-demand", 1, 1, "invalid bool representation"),
    ("partial-initialization-repair", 0, 0, ""),
    (
        "unsafe-required",
        1,
        1,
        "call to unsafe function requires an unsafe context",
    ),
    ("wrong-fill-type", 1, 1, "incompatible types"),
    ("non-copy-generic-aggregate", 0, 0, ""),
    ("invalid-char-transport", 0, 0, ""),
    ("invalid-char-demand", 1, 1, "invalid char representation"),
    ("invalid-bool-repair", 0, 0, ""),
    (
        "partial-provenance-final",
        1,
        1,
        "partial or misordered pointer provenance",
    ),
    ("partial-provenance-restore", 0, 0, ""),
    ("whole-provenance-overwrite", 0, 0, ""),
    ("writable-after-shared-borrow", 0, 0, ""),
    ("reified-generic-runtime", 0, 0, ""),
    (
        "reified-const-call",
        1,
        1,
        "non-const calls are not allowed",
    ),
    ("destructor-not-overwritten", 0, 0, ""),
    ("argument-evaluation-once", 0, 0, ""),
    ("reference-bearing-repair", 0, 0, ""),
    (
        "packed-field-one",
        0,
        1,
        "constant counted pointer is unaligned",
    ),
    ("atomic-static-zero", 0, 0, ""),
    (
        "sized-required",
        1,
        1,
        "generic type argument must be sized",
    ),
    ("shadowed-nonconst", 1, 1, "non-const calls are not allowed"),
    ("canonical-core-alias", 0, 0, ""),
    ("zst-max-count-literal", 0, 0, ""),
    (
        "numeric-zst-literal",
        0,
        1,
        "constant counted numeric pointer access is not implemented",
    ),
    ("zst-expired-literal", 0, 0, ""),
    (
        "count-overflow-literal",
        1,
        1,
        "constant fill byte count exceeds allocation size",
    ),
    ("runtime-zero-zst", 0, 0, ""),
    ("packed-whole-runtime", 0, 0, ""),
];
#[test]
fn byte_fill_sources_match_declared_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let evidence = root.join(format!(
        "obj/parallel/pa28/byte-fill/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, front, reason) in CASES {
        checked::check(
            &root.join(format!("pa28/tests/spec/byte-fill/{name}.rs")),
            &evidence,
            name,
            (rust, front, reason),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn byte_fill_native_values_match_independent_execution() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let names = &[
        "byte-scalar",
        "invalid-bool-transport",
        "partial-initialization-repair",
        "non-copy-generic-aggregate",
        "invalid-char-transport-literal",
        "invalid-bool-repair",
        "partial-provenance-restore",
        "whole-provenance-overwrite",
        "writable-after-shared-borrow",
        "reified-generic-runtime",
        "destructor-not-overwritten",
        "argument-evaluation-once",
        "reference-bearing-repair",
        "runtime-zero-zst",
        "packed-whole-runtime",
    ];
    let mut inputs = Vec::new();
    for &name in names {
        inputs.push((
            name,
            std::fs::read_to_string(root.join(format!("pa28/tests/byte-fill-native/{name}.rs")))?,
        ));
    }
    let cases = inputs
        .iter()
        .map(|(name, body)| (*name, body.as_str(), false))
        .collect::<Vec<_>>();
    native::check_literal_cases("parallel/pa28/byte-fill/native", &cases)
}
