//! Overlapping transfers snapshot typed storage and retain explicit byte-profile boundaries.
#[path = "overlapping_transfers/backend.rs"]
mod backend;
use crate::checked_sources as checked;
use crate::counted_cases as declared;
use crate::native_references as native;
use crate::qualify as rustc;
use crate::resolve as source;
#[path = "overlapping_transfers/work.rs"]
mod work;
use std::path::Path;
const ADDITIONAL: &[(&str, i32, i32, &str)] = &[
    ("overlap-left", 0, 0, ""),
    ("overlap-right", 0, 0, ""),
    ("identical", 0, 0, ""),
    ("aggregate-overlap", 0, 0, ""),
    ("nested-rest-overlap", 0, 0, ""),
    (
        "intra-object-profile",
        0,
        1,
        "constant counted pointer reinterpretation is not implemented",
    ),
    (
        "invalid-byte-profile",
        0,
        1,
        "constant counted union-field byte transport is not implemented",
    ),
    (
        "invalid-byte-demand",
        1,
        1,
        "constant counted union-field byte transport is not implemented",
    ),
    ("local-copy-shadow", 0, 0, ""),
    ("local-core-shadow", 0, 0, ""),
];
#[test]
fn overlapping_memory_matches_predeclared_source_outcomes() -> Result<(), Box<dyn std::error::Error>>
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa28/overlapping-memory/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for &(name, rust, front, reason) in declared::CASES.iter().chain(ADDITIONAL) {
        let (rust, front, reason) = match name {
            "shared-one" => (
                1,
                1,
                "constant raw pointer write is derived from a shared reference",
            ),
            "same-root-overlap" => (0, 0, ""),
            _ => (rust, front, reason),
        };
        checked::check(
            &root.join(format!("pa28/tests/spec/overlapping-memory/{name}.rs")),
            &out,
            name,
            (rust, front, reason),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn overlapping_memory_executes_owned_and_reified_moves() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let names = &[
        "backend-loop",
        "scalars",
        "disjoint",
        "generic",
        "reified",
        "argument-once",
        "reference-elements",
        "rest-destination",
        "owned-drops",
        "zst-large",
        "shared-zero",
        "overlap-left",
        "overlap-right",
        "identical",
        "intra-object-left",
        "intra-object-right",
        "overlap-owned-drops",
        "rest-overlap",
        "invalid-bool-byte",
        "invalid-char-byte",
        "nonoverlap-invalid-bool-control",
        "nonoverlap-nan-bits-control",
        "nonoverlap-reference-bits-control",
    ];
    let inputs = names
        .iter()
        .map(|&name| {
            std::fs::read_to_string(
                root.join(format!("pa28/tests/overlapping-memory-native/{name}.rs")),
            )
            .map(|source| (name, source))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let cases = inputs
        .iter()
        .map(|(name, source)| (*name, source.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("parallel/pa28/overlapping-memory/native", &cases)
}
