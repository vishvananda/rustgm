//! Symbolic byte transport retains provenance, checked shapes and independent validity.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
const CASES: &[(&str, i32, i32, &str)] = &[
    ("thin-split-same", 0, 0, ""),
    ("thin-reverse-same", 0, 0, ""),
    ("thin-via-two-snapshots", 0, 0, ""),
    ("thin-mixed-identities", 1, 1, "provenance word"),
    (
        "partial-numeric-overwrite",
        1,
        1,
        "incomplete or misordered provenance",
    ),
    (
        "partial-unused-image",
        1,
        1,
        "partial or misordered pointer provenance",
    ),
    ("numeric-pointer-read", 1, 1, "symbolic pointer bytes"),
    (
        "shared-derived-reassembly",
        1,
        1,
        "derived from a shared reference",
    ),
    ("writable-separate-shared", 0, 0, ""),
    ("expired-reassembly", 1, 1, "expired storage"),
    (
        "slice-split",
        0,
        1,
        "constant raw whole-range access is not implemented",
    ),
    (
        "slice-mixed-length",
        0,
        1,
        "constant raw whole-range access is not implemented",
    ),
    ("slice-mixed-length-bounds", 1, 1, "out of bounds"),
    ("function-split", 0, 0, ""),
    ("object-split", 0, 1, "operation requires a scalar type"),
    ("thin-duplicate-cells", 0, 0, ""),
    (
        "thin-same-allocation-different-offset",
        1,
        1,
        "provenance word",
    ),
    ("complete-image", 0, 0, ""),
    (
        "slice-modified-metadata",
        0,
        1,
        "constant raw whole-range access is not implemented",
    ),
    ("complete-slice-image", 0, 0, ""),
    ("shared-reference-split", 0, 0, ""),
    (
        "object-separate-vtable-word",
        0,
        1,
        "operation requires a scalar type",
    ),
    (
        "object-same-vtable-halves",
        0,
        1,
        "operation requires a scalar type",
    ),
    (
        "object-mixed-vtable-halves",
        1,
        1,
        "operation requires a scalar type",
    ),
    (
        "object-numeric-vtable-word",
        1,
        1,
        "operation requires a scalar type",
    ),
    ("thin-duplicate-field-views", 0, 0, ""),
    ("thin-root-first-element-views", 0, 0, ""),
    ("slice-prefix-data-views", 0, 0, ""),
    (
        "ordinary-raw-slice-index",
        0,
        1,
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    ("thin-duplicate-field-views-encoding-reversed", 0, 0, ""),
    ("thin-duplicate-field-views-copy-reversed", 0, 0, ""),
    ("thin-root-first-element-views-encoding-reversed", 0, 0, ""),
    ("thin-root-first-element-views-copy-reversed", 0, 0, ""),
    ("slice-prefix-data-views-encoding-reversed", 0, 0, ""),
    ("slice-prefix-data-views-copy-reversed", 0, 0, ""),
    ("multiple-static-reference-waits", 0, 0, ""),
    ("reference-read-after-intervening-write", 0, 0, ""),
    ("object-split-typed-let", 0, 0, ""),
    ("object-separate-vtable-word-typed-let", 0, 0, ""),
    ("object-same-vtable-halves-typed-let", 0, 0, ""),
    (
        "object-mixed-vtable-halves-typed-let",
        1,
        1,
        "provenance word",
    ),
    (
        "object-numeric-vtable-word-typed-let",
        1,
        1,
        "symbolic pointer bytes",
    ),
    (
        "vtable-borrowed-concrete-forward",
        1,
        1,
        "required lifetime does not outlive its target",
    ),
    ("vtable-borrowed-concrete-forward-scoped", 0, 0, ""),
    (
        "vtable-borrowed-concrete-reversed",
        1,
        1,
        "required lifetime does not outlive its target",
    ),
    ("vtable-borrowed-concrete-reversed-scoped", 0, 0, ""),
    ("vtable-trait-shape-mixed", 1, 1, "provenance word"),
    ("vtable-const-shape-mixed", 1, 1, "provenance word"),
    ("vtable-associated-shape-mixed", 1, 1, "provenance word"),
    (
        "vtable-scoped-borrowed-object-escape",
        1,
        1,
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    ("complete-function-image", 0, 0, ""),
    ("complete-object-image", 0, 0, ""),
    (
        "vtable-scoped-reference-cell-escape",
        1,
        1,
        "expired storage",
    ),
    ("union-owned-interior", 0, 0, ""),
    ("union-interior-reference", 1, 1, "expired storage"),
];
#[test]
fn relocation_sources_match_independent_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let evidence = root.join(format!(
        "obj/parallel/pa28/relocation-transport/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, front, reason) in CASES {
        checked::check(
            &root.join(format!("pa28/tests/spec/relocation-transport/{name}.rs")),
            &evidence,
            name,
            (rust, front, reason),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn relocation_native_values_match_independent_execution() -> Result<(), Box<dyn std::error::Error>>
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let names = &[
        "thin-split-same-literal-native",
        "thin-reverse-same-literal-native",
        "thin-via-two-snapshots-literal-native",
        "writable-separate-shared-literal-native",
        "function-split-literal-native",
        "thin-duplicate-cells-literal-native",
        "complete-image-literal-native",
        "complete-slice-image-literal-native",
        "shared-reference-split-literal-native",
        "thin-duplicate-field-views-literal-native",
        "thin-root-first-element-views-literal-native",
        "slice-prefix-data-views-literal-native",
        "thin-duplicate-field-views-encoding-reversed-literal-native",
        "thin-duplicate-field-views-copy-reversed-literal-native",
        "thin-root-first-element-views-encoding-reversed-literal-native",
        "thin-root-first-element-views-copy-reversed-literal-native",
        "slice-prefix-data-views-encoding-reversed-literal-native",
        "slice-prefix-data-views-copy-reversed-literal-native",
        "multiple-static-reference-waits-literal-native",
        "reference-read-after-intervening-write-literal-native",
        "object-split-typed-let-literal-native",
        "object-separate-vtable-word-typed-let-literal-native",
        "object-same-vtable-halves-typed-let-literal-native",
        "pointer-neighbor-literal-native",
        "complete-function-image-literal-native",
        "complete-object-image-literal-native",
        "vtable-borrowed-concrete-forward-scoped-literal-native",
        "vtable-borrowed-concrete-reversed-scoped-literal-native",
    ];
    let mut inputs = Vec::new();
    for &name in names {
        inputs.push((
            name,
            std::fs::read_to_string(root.join(format!("pa28/tests/relocation-native/{name}.rs")))?,
        ));
    }
    let cases = inputs
        .iter()
        .map(|(name, body)| (*name, body.as_str(), false))
        .collect::<Vec<_>>();
    native::check_literal_cases("parallel/pa28/relocation-transport/native", &cases)
}
