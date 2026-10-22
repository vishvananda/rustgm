//! Null-pointer Option carriers keep their pointer and C composite call ABI.
use crate::{host_objects, native_references as native};
use std::path::Path;
const CASES: &[(&str, &str, bool)] = &[
    (
        "c-reference-call",
        "pa30/tests/spec/option-niche/c-reference-call.rs",
        false,
    ),
    (
        "c-function-call",
        "pa30/tests/spec/option-niche/c-function-call.rs",
        false,
    ),
    (
        "c-record-carrier",
        "pa30/tests/spec/option-niche/c-record-carrier.rs",
        true,
    ),
    (
        "c-reified-carrier",
        "pa30/tests/spec/option-niche/c-reified-carrier.rs",
        false,
    ),
    (
        "host-reference",
        "pa30/tests/spec/option-niche/host-reference.rs",
        true,
    ),
    (
        "host-function",
        "pa30/tests/spec/option-niche/host-function.rs",
        true,
    ),
    (
        "host-record",
        "pa30/tests/spec/option-niche/host-record.rs",
        true,
    ),
];
#[test]
#[ignore = "requires pinned rustc, CPPGM object bridge and C host toolchain"]
fn nullable_c_call_abi() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let mut direct = Vec::new();
    let mut objects = Vec::new();
    for &(name, path, object) in CASES {
        if object {
            objects.push((name, path));
        } else {
            direct.push((name, std::fs::read_to_string(root.join(path))?));
        }
    }
    let cases: Vec<_> = direct
        .iter()
        .map(|(name, text)| (*name, text.as_str(), false))
        .collect();
    native::check_literal_cases("parallel/pa27/default-enum-bytes/c-native", &cases)?;
    host_objects::check_cases(
        "pa27/default-enum-bytes/host",
        "pa30/tests/spec/option-niche/provider.c",
        &objects,
    )
}
