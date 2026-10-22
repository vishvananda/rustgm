//! Byte transport preserves initialization independently of typed scalar validity.
#[path = "harness/checked_sources.rs"]
mod checked;
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "helpers/resolve.rs"]
mod source;
#[path = "byte_transfers/work.rs"]
mod work;
use std::path::Path;
const CASES: &[(&str, i32, i32, &str)] = &[
    ("invalid-bool-transport", 0, 0, ""),
    ("invalid-bool-demand", 1, 1, "invalid bool representation"),
    ("invalid-char-transport", 0, 0, ""),
    ("invalid-char-demand", 1, 1, "invalid char representation"),
    ("uninitialized-transport", 0, 0, ""),
    ("uninitialized-demand", 1, 1, "uses uninitialized bytes"),
    ("uninitialized-identical", 0, 0, ""),
    ("invalid-restore", 0, 0, ""),
    ("projected-ordinary-writes", 0, 0, ""),
    ("overlapping-invalid-snapshot", 0, 0, ""),
    ("rest-alias-ordinary-read", 0, 0, ""),
    (
        "pointer-neighbor-profile",
        0,
        1,
        "constant byte backing with pointer or enum storage is not implemented",
    ),
    ("large-root-small-transfer", 0, 0, ""),
    ("ordinary-control", 0, 0, ""),
    ("length-with-invalid-element", 0, 0, ""),
    ("empty-rest-with-invalid-prefix", 0, 0, ""),
    ("range-assignment-after-conversion", 0, 0, ""),
    ("uninitialized-array-restore", 0, 0, ""),
    ("zero-sized-field", 0, 0, ""),
    ("uninitialized-bool-destination", 1, 1, "uninitialized"),
    ("uninitialized-word-destination", 1, 1, "uninitialized"),
    ("uninitialized-word-demand", 1, 1, "uninitialized"),
];
#[test]
fn byte_transport_matches_independent_source_validity() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let evidence = root.join(format!(
        "obj/parallel/pa28/byte-transport/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, front, reason) in CASES {
        checked::check(
            &root.join(format!("pa28/tests/spec/byte-transport/{name}.rs")),
            &evidence,
            name,
            (rust, front, reason),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn byte_transport_executes_constant_values() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let mut inputs = Vec::new();
    for &(name, rust, front, _) in CASES {
        if rust == 0 && front == 0 {
            let body = std::fs::read_to_string(
                root.join(format!("pa28/tests/byte-transport-native/{name}.rs")),
            )?;
            inputs.push((name, body));
        }
    }
    let cases = inputs
        .iter()
        .map(|(name, body)| (*name, body.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("parallel/pa28/byte-transport/native", &cases)
}
