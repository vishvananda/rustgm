//! Exact raw slice selections retain logical metadata and independent physical access duties.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
#[path = "raw_slice_indexing/work.rs"]
mod work;
const CASES: &[(&str, &str, i32, [i32; 3], &str)] = &[
    (
        "retained-slice-split",
        "pa28/tests/spec/relocation-transport/slice-split.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "retained-slice-mixed-length",
        "pa28/tests/spec/relocation-transport/slice-mixed-length.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "retained-slice-modified-metadata",
        "pa28/tests/spec/relocation-transport/slice-modified-metadata.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "retained-ordinary-raw-slice-index",
        "pa28/tests/spec/relocation-transport/ordinary-raw-slice-index.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "retained-slice-mixed-length-bounds",
        "pa28/tests/spec/relocation-transport/slice-mixed-length-bounds.rs",
        1,
        [1, 1, 1],
        "constant index is out of bounds",
    ),
    (
        "ordinary-read",
        "pa27/tests/spec/raw-slice-index/ordinary-read.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "ordinary-write",
        "pa27/tests/spec/raw-slice-index/ordinary-write.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "subrange-write",
        "pa27/tests/spec/raw-slice-index/subrange-write.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "shared-derived-write",
        "pa27/tests/spec/raw-slice-index/shared-derived-write.rs",
        1,
        [1, 1, 1],
        "constant raw pointer write is derived from a shared reference",
    ),
    (
        "empty-index",
        "pa27/tests/spec/raw-slice-index/empty-index.rs",
        1,
        [1, 1, 1],
        "constant index is out of bounds",
    ),
    (
        "metadata-end-index",
        "pa27/tests/spec/raw-slice-index/metadata-end-index.rs",
        1,
        [1, 1, 1],
        "constant index is out of bounds",
    ),
    (
        "expired-read",
        "pa27/tests/spec/raw-slice-index/expired-read.rs",
        1,
        [1, 1, 1],
        "constant pointer refers to expired storage",
    ),
    (
        "shared-element-reference",
        "pa27/tests/spec/raw-slice-index/shared-element-reference.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "mutable-element-reference",
        "pa27/tests/spec/raw-slice-index/mutable-element-reference.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "invalid-unselected",
        "pa28/tests/spec/raw-slice-index/invalid-unselected.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "invalid-selected",
        "pa28/tests/spec/raw-slice-index/invalid-selected.rs",
        1,
        [1, 1, 1],
        "constant union scalar read has invalid bool representation",
    ),
    (
        "zst-index",
        "pa27/tests/spec/raw-slice-index/zst-index.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "suffix-pattern",
        "pa27/tests/spec/raw-slice-index/suffix-pattern.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "empty-pattern",
        "pa27/tests/spec/raw-slice-index/empty-pattern.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "packed-element-field",
        "pa27/tests/spec/raw-slice-index/packed-element-field.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "packed-data-read",
        "pa27/tests/spec/raw-slice-index/packed-data-read.rs",
        1,
        [1, 1, 1],
        "constant raw pointer read is unaligned",
    ),
    (
        "packed-data-address-unaligned-read",
        "pa28/tests/spec/raw-slice-index/packed-data-address-unaligned-read.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "byte-literal",
        "pa27/tests/spec/raw-slice-index/byte-literal.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "literal-write",
        "pa27/tests/spec/raw-slice-index/literal-write.rs",
        1,
        [1, 1, 1],
        "constant raw pointer write is derived from a shared reference",
    ),
    (
        "oversized-metadata-selected-in-backing",
        "pa27/tests/spec/raw-slice-index/oversized-metadata-selected-in-backing.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "oversized-metadata-selected-outside-backing",
        "pa27/tests/spec/raw-slice-index/oversized-metadata-selected-outside-backing.rs",
        1,
        [1, 1, 1],
        "constant raw slice element is outside backing bounds",
    ),
    (
        "generic-dynamic-index",
        "pa27/tests/spec/raw-slice-index/generic-dynamic-index.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "expired-length-pattern",
        "pa27/tests/spec/raw-slice-index/expired-length-pattern.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "expired-element-address-zero",
        "pa27/tests/spec/raw-slice-index/expired-element-address-zero.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "uninitialized-length-pattern",
        "pa27/tests/spec/raw-slice-index/uninitialized-length-pattern.rs",
        1,
        [1, 1, 1],
        "borrowed place may be uninitialized or moved",
    ),
    (
        "uninitialized-selected-read",
        "pa27/tests/spec/raw-slice-index/uninitialized-selected-read.rs",
        1,
        [1, 1, 1],
        "borrowed place may be uninitialized or moved",
    ),
    (
        "oversized-length-pattern",
        "pa27/tests/spec/raw-slice-index/oversized-length-pattern.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "packed-length-pattern",
        "pa27/tests/spec/raw-slice-index/packed-length-pattern.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "expired-element-address-nonzero",
        "pa27/tests/spec/raw-slice-index/expired-element-address-nonzero.rs",
        1,
        [1, 1, 1],
        "constant pointer refers to expired storage",
    ),
    (
        "expired-zst-element-address",
        "pa27/tests/spec/raw-slice-index/expired-zst-element-address.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "expired-element-address-outside-metadata",
        "pa27/tests/spec/raw-slice-index/expired-element-address-outside-metadata.rs",
        1,
        [1, 1, 1],
        "constant index is out of bounds",
    ),
    (
        "expired-binding-required",
        "pa27/tests/spec/raw-slice-index/expired-binding-required.rs",
        1,
        [1, 1, 1],
        "constant pointer refers to expired storage",
    ),
    (
        "expired-literal-required",
        "pa27/tests/spec/raw-slice-index/expired-literal-required.rs",
        1,
        [1, 1, 1],
        "constant pointer refers to expired storage",
    ),
    (
        "oversized-binding-required",
        "pa27/tests/spec/raw-slice-index/oversized-binding-required.rs",
        1,
        [1, 1, 1],
        "constant raw slice element is outside backing bounds",
    ),
    (
        "oversized-literal-required",
        "pa27/tests/spec/raw-slice-index/oversized-literal-required.rs",
        1,
        [1, 1, 1],
        "constant raw slice element is outside backing bounds",
    ),
    (
        "reference-metadata-full-range-required",
        "pa27/tests/spec/raw-slice-index/reference-metadata-full-range-required.rs",
        0,
        [1, 1, 1],
        "constant range is out of bounds",
    ),
    (
        "reference-word-roundtrip",
        "pa28/tests/spec/raw-slice-index/reference-word-roundtrip.rs",
        0,
        [0, 0, 0],
        "",
    ),
    (
        "reference-metadata-published",
        "pa27/tests/spec/raw-slice-index/reference-metadata-published.rs",
        1,
        [1, 1, 1],
        "constant range is out of bounds",
    ),
];
#[test]
fn raw_slice_sources_match_independently_declared_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let evidence = root.join(format!(
        "obj/parallel/pa27/raw-slice-index/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, path, rust, frontend, reason) in CASES {
        checked::check_modes(&root.join(path), &evidence, name, (rust, frontend, reason))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn raw_slice_native_values_and_pattern_drop_match_independent_execution()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let paths = &[
        (
            "retained-slice-split",
            "pa28/tests/raw-slice-native/retained-slice-split.rs",
        ),
        (
            "retained-slice-mixed-length",
            "pa28/tests/raw-slice-native/retained-slice-mixed-length.rs",
        ),
        (
            "retained-slice-modified-metadata",
            "pa28/tests/raw-slice-native/retained-slice-modified-metadata.rs",
        ),
        (
            "retained-ordinary-raw-slice-index",
            "pa27/tests/raw-slice-native/retained-ordinary-raw-slice-index.rs",
        ),
        (
            "ordinary-read",
            "pa27/tests/raw-slice-native/ordinary-read.rs",
        ),
        (
            "ordinary-write",
            "pa27/tests/raw-slice-native/ordinary-write.rs",
        ),
        (
            "subrange-write",
            "pa27/tests/raw-slice-native/subrange-write.rs",
        ),
        (
            "shared-element-reference",
            "pa27/tests/raw-slice-native/shared-element-reference.rs",
        ),
        (
            "mutable-element-reference",
            "pa27/tests/raw-slice-native/mutable-element-reference.rs",
        ),
        (
            "invalid-unselected",
            "pa28/tests/raw-slice-native/invalid-unselected.rs",
        ),
        (
            "suffix-pattern",
            "pa27/tests/raw-slice-native/suffix-pattern.rs",
        ),
        (
            "empty-pattern",
            "pa27/tests/raw-slice-native/empty-pattern.rs",
        ),
        (
            "packed-element-field",
            "pa27/tests/raw-slice-native/packed-element-field.rs",
        ),
        (
            "packed-data-address-unaligned-read",
            "pa28/tests/raw-slice-native/packed-data-address-unaligned-read.rs",
        ),
        (
            "byte-literal",
            "pa27/tests/raw-slice-native/byte-literal.rs",
        ),
        (
            "oversized-metadata-selected-in-backing",
            "pa27/tests/raw-slice-native/oversized-metadata-selected-in-backing.rs",
        ),
        (
            "generic-dynamic-index",
            "pa27/tests/raw-slice-native/generic-dynamic-index.rs",
        ),
        (
            "reference-word-roundtrip",
            "pa28/tests/raw-slice-native/reference-word-roundtrip.rs",
        ),
        (
            "wildcard-scrutinee-once",
            "pa27/tests/raw-slice-native/wildcard-scrutinee-once.rs",
        ),
    ];
    let mut inputs = Vec::new();
    for &(name, path) in paths {
        inputs.push((name, std::fs::read_to_string(root.join(path))?));
    }
    inputs.push(("ordinary-pattern-move-drop", r"#![allow(dead_code,unused_mut,invalid_reference_casting)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static mut COUNT:u32=0;struct D{n:u32}impl Drop for D{fn drop(&mut self){unsafe{COUNT+=self.n;}}}fn main(){{let (_,selected)=(D{n:3},D{n:7});check(unsafe{COUNT}==3);check(selected.n==7);}check(unsafe{COUNT}==10);}
".to_owned()));
    let cases = inputs
        .iter()
        .map(|(name, body)| (*name, body.as_str(), false))
        .collect::<Vec<_>>();
    native::check_literal_cases("parallel/pa27/raw-slice-index/native", &cases)
}
