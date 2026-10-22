//! Nested fields preserve their actual placement through imports, exports and callbacks.
use super::{frontend, objects};
use std::path::Path;
const ACCEPTED: &[&str] = &[
    "nested-integer",
    "nested-sse",
    "nested-integer-sse",
    "nested-sse-integer",
    "array-integer",
    "array-sse",
    "array-bytes",
    "array-nested-merge",
    "nested-memory",
    "shifted-nested-mixed",
    "shifted-array-mixed",
    "zero-array-field",
];
const REJECTED: &[&str] = &[
    "packed-inner",
    "rust-layout-inner",
    "union-inner",
    "wrong-nominal",
    "unsafe-call",
    "wrong-arity",
];
fn root() -> Result<&'static Path, Box<dyn std::error::Error>> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "root".into())
}
#[test]
fn composite_source_contracts_and_unsupported_profiles_are_distinct()
-> Result<(), Box<dyn std::error::Error>> {
    for (&name, accepted) in ACCEPTED
        .iter()
        .map(|name| (name, true))
        .chain(REJECTED.iter().map(|name| (name, false)))
    {
        let source = std::fs::read_to_string(root()?.join(format!(
            "pa30/tests/spec/aggregate-abi/composites/{name}.rs"
        )))?;
        let checked = frontend::checked(&source);
        assert_eq!(checked.is_ok(), accepted, "{name}: {:?}", checked.err());
    }
    Ok(())
}
fn native_case(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source = format!("pa30/tests/spec/aggregate-abi/composites/{name}.rs");
    let provider = format!("pa30/tests/spec/aggregate-abi/composites/{name}.c");
    objects::check_cases(
        &format!("pa30-composites/{name}"),
        &provider,
        &[(name, &source)],
    )
}
#[test]
#[ignore = "requires the optional pinned CPPGM object profile and host C compiler"]
fn composite_calls_match_rust_c_and_shifted_eightbyte_classes()
-> Result<(), Box<dyn std::error::Error>> {
    for &name in ACCEPTED.iter().filter(|&&name| name != "zero-array-field") {
        native_case(name)?;
    }
    objects::check_cases(
        "pa30-composites/legacy",
        "pa30/tests/spec/aggregate-abi/composites/empty.c",
        &[
            ("nested", "pa30/tests/spec/aggregate-abi/nested.rs"),
            ("array", "pa30/tests/spec/aggregate-abi/array.rs"),
        ],
    )
}
#[test]
#[ignore = "requires optional pinned object profile and GNU C zero-length-array extension"]
fn zero_array_field_retains_explicit_gnu_host_qualification()
-> Result<(), Box<dyn std::error::Error>> {
    native_case("zero-array-field")
}
