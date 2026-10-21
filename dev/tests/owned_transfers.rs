//! Sized pointer transfers use ordinary ownership and preserve raw storage authority.
#[path = "harness/checked_sources.rs"]
mod checked;
#[path = "harness/native_references.rs"]
mod native;
use std::path::Path;
const CASES: &[(&str, i32, [i32; 3], &str)] = &[
    ("plain-runtime", 0, [0, 0, 0], ""),
    ("drop-write", 0, [0, 0, 0], ""),
    ("ordinary-assignment", 0, [0, 0, 0], ""),
    ("drop-read", 0, [0, 0, 0], ""),
    ("drop-discarded-read", 0, [0, 0, 0], ""),
    ("move-then-reinitialize", 0, [0, 0, 0], ""),
    ("generic", 0, [0, 0, 0], ""),
    ("reified", 0, [0, 0, 0], ""),
    (
        "write-moves-source",
        1,
        [0, 1, 1],
        "local may be uninitialized or moved at this read",
    ),
    ("read-preserves-source", 0, [0, 0, 0], ""),
    (
        "uninitialized-address",
        1,
        [0, 1, 1],
        "borrowed place may be uninitialized or moved",
    ),
    ("const-plain", 0, [0, 0, 0], ""),
    ("const-move-reinitialize", 0, [0, 0, 0], ""),
    ("const-moved-raw-field", 0, [0, 0, 0], ""),
    ("const-moved-ptr-read", 0, [0, 0, 0], ""),
    (
        "const-reference-transfer",
        0,
        [1, 1, 1],
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    (
        "const-reference-alias-use",
        0,
        [1, 1, 1],
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    ("const-raw-field-transfer", 0, [0, 0, 0], ""),
    (
        "const-drop-forbidden",
        1,
        [1, 1, 1],
        "constant evaluation cannot run destructors",
    ),
    ("const-static-drop-result", 0, [0, 0, 0], ""),
    (
        "const-shared-write",
        1,
        [1, 1, 1],
        "constant raw pointer write is derived from a shared reference",
    ),
    (
        "const-stale-read",
        1,
        [1, 1, 1],
        "constant pointer refers to expired storage",
    ),
    (
        "const-interior-static",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
    (
        "const-interior-whole",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
    (
        "const-interior-ordinary-field",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
    (
        "const-interior-inactive",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
    ("const-interior-zero-array", 0, [0, 0, 0], ""),
    ("const-interior-local", 0, [0, 0, 0], ""),
    ("const-interior-reference-slot", 0, [0, 0, 0], ""),
    (
        "const-union-mask",
        1,
        [1, 1, 1],
        "constant union scalar read uses uninitialized bytes",
    ),
    ("const-union-preserve", 0, [0, 0, 0], ""),
    ("static-interior-read", 0, [0, 0, 0], ""),
    ("static-interior-field-read", 0, [0, 0, 0], ""),
    (
        "static-mutable-read",
        0,
        [1, 1, 1],
        "constant execution cannot read mutable static storage",
    ),
    ("const-interior-reference", 0, [0, 0, 0], ""),
    ("const-interior-zst-field", 0, [0, 0, 0], ""),
    ("static-interior-function", 0, [0, 0, 0], ""),
    (
        "const-interior-function",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
    (
        "static-nested-named-const",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
    (
        "static-nested-inline-const",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
    ("static-nested-const-function", 0, [0, 0, 0], ""),
    ("const-direct-reference-cell", 0, [0, 0, 0], ""),
    (
        "mixed-static-first",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
    (
        "mixed-const-first",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
    (
        "const-direct-inline",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
    (
        "static-grouped-inline",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
    ("static-grouped-plain", 0, [0, 0, 0], ""),
    (
        "static-nested-block-inline",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
    ("static-inline-scalar", 0, [0, 0, 0], ""),
    ("const-inline-scalar", 0, [0, 0, 0], ""),
    ("static-inline-reference", 0, [0, 0, 0], ""),
    (
        "static-associated-const",
        1,
        [1, 1, 1],
        "constant execution cannot read interior-mutable static storage",
    ),
];
#[test]
fn owned_pointer_transfers_match_independent_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa28/owned-pointer-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for &(name, rust, statuses, reason) in CASES {
        let lane = if ["static-inline-scalar", "const-inline-scalar"].contains(&name) {
            "pa25/tests/spec/inline-owner-context"
        } else {
            "pa28/tests/spec/owned-pointer-memory"
        };
        let path = root.join(format!("{lane}/{name}.rs"));
        if statuses == [statuses[0]; 3] {
            checked::check(&path, &out, name, (rust, statuses[0], reason))?;
        } else {
            checked::check_modes(&path, &out, name, (rust, statuses, reason))?;
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn owned_pointer_destructors_and_reified_transfers() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let names = [
        "const-plain",
        "generic",
        "ignored-read",
        "move-reinitialize",
        "nested-fields",
        "ordinary-assignment",
        "overwrite-no-drop",
        "read-forget",
        "read-replace",
        "reified",
        "direct-reference-cell",
        "static-context",
        "static-reference",
        "zero-sized-drop",
    ];
    let cases = names
        .into_iter()
        .map(|name| {
            std::fs::read_to_string(root.join(format!("pa28/tests/owned-memory-native/{name}.rs")))
                .map(|body| (name, body))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let borrowed = cases
        .iter()
        .map(|(name, body)| (*name, body.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("parallel/pa28/owned-pointer-native", &borrowed)
}
