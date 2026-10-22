//! Variadic C calls preserve fixed inputs, explicit promotions and actual tail types.
use crate::{foreign_frontend as frontend, host_objects, qualify};
use std::path::{Path, PathBuf};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const CASES: &[(&str, bool, bool)] = &[
    ("integers", true, true),
    ("zero-tail", true, true),
    ("float-register-spill", true, true),
    ("integer-register-spill", true, true),
    ("indirect", true, true),
    ("constant-pointer", true, true),
    ("safe-ignores-tail", true, true),
    ("named-tail", true, true),
    ("explicit-promotions", true, true),
    ("callback", true, true),
    ("evaluation-order", true, true),
    ("mixed-scalars", true, true),
    ("record-tail", true, true),
    ("record-prefix-result", true, true),
    ("nested-record-tail", true, true),
    ("bare-signature", true, false),
    ("generic-caller", true, true),
    ("unpromoted-u8", false, false),
    ("unpromoted-i8", false, false),
    ("unpromoted-u16", false, false),
    ("unpromoted-i16", false, false),
    ("unpromoted-bool", false, false),
    ("unpromoted-f32", false, false),
    ("function-item", false, false),
    ("unsafe-call", false, false),
    ("missing-fixed", false, false),
    ("wrong-fixed", false, false),
    ("pointer-safety", false, false),
    ("rust-abi", false, false),
    ("variadic-body", false, false),
    ("fixed-extra", false, false),
];
const TAIL_CASES: &[(&str, bool, bool)] = &[
    ("multiple-records", true, true),
    ("multiple-records-indirect", true, true),
    ("inferred-pointer", true, true),
    ("shared-tail", true, true),
    ("generic-small-tail", true, true),
    ("zero-sized-tail", true, true),
    ("moved-tail", true, true),
    ("moved-tail-read", false, false),
    ("mutable-alias-tail", false, false),
    ("expired-tail", false, false),
    ("constant-call", false, false),
];
fn root() -> TestResult<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?
        .to_path_buf())
}
#[test]
fn variadic_source_outcomes_match_independent_rust() -> TestResult {
    source_cases("variadics", CASES)
}
#[test]
fn variadic_tail_outcomes_preserve_inference_and_ownership() -> TestResult {
    source_cases("variadic-tails", TAIL_CASES)
}
fn source_cases(folder: &str, cases: &[(&str, bool, bool)]) -> TestResult {
    let root = root()?;
    let mut failures = Vec::new();
    for &(name, accepted, _) in cases {
        let source =
            std::fs::read_to_string(root.join(format!("pa30/tests/spec/{folder}/{name}.rs")))?;
        qualify::qualify(&format!("{folder}-{name}"), &source, accepted)?;
        let actual = frontend::checked(&source).and_then(|program| {
            let mut text = Vec::new();
            rustgm::lowir::writer::write(&program, &mut text).map_err(|e| e.to_string())?;
            Ok(())
        });
        if actual.is_ok() != accepted {
            failures.push(format!("{name}: {actual:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the optional pinned CPPGM object profile and host C compiler"]
fn variadic_values_match_independent_c_and_rust() -> TestResult {
    native_cases("variadics", CASES)
}
#[test]
#[ignore = "requires the optional pinned CPPGM object profile and host C compiler"]
fn variadic_tail_values_match_independent_c_and_rust() -> TestResult {
    native_cases("variadic-tails", TAIL_CASES)
}
fn native_cases(folder: &str, cases: &[(&str, bool, bool)]) -> TestResult {
    let paths: Vec<_> = cases
        .iter()
        .filter(|(_, _, native)| *native)
        .map(|(name, _, _)| (*name, format!("pa30/tests/spec/{folder}/{name}.rs")))
        .collect();
    let cases: Vec<_> = paths
        .iter()
        .map(|(name, path)| (*name, path.as_str()))
        .collect();
    host_objects::check_cases(
        folder,
        &format!("pa30/tests/spec/{folder}/provider.c"),
        &cases,
    )
}
