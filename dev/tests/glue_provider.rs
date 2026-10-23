//! Actual source helper bodies retain their independently settled execution contracts.
use crate::provider_traits::{TestResult, lower};
use std::path::Path;
#[path = "glue_provider/answers.rs"]
mod answers;
#[path = "glue_provider/unsized_elements.rs"]
mod unsized_elements;

const POSITIVE: &[(&str, &str)] = &[
    ("original-cleanup-count", "7\n"),
    ("direct-helper", "7\n"),
    ("generic-helper", "7\n"),
    ("reified-helper", "7\n"),
    ("pinned-wrapper-body", "7\n"),
    ("ordinary-lookalike", "0\n"),
    ("owner-fields-order", "3\n"),
    ("enum-live-payload", "2\n"),
    ("zst-array0", "0\n"),
    ("primitive-dropless", "7\n"),
    ("reference-does-not-own", "0\n"),
];
const NEGATIVE: &[&str] = &[
    "const-nonconst-drop",
    "const-unpaid-destruct",
    "const-shared-restriction",
    "reified-wrong-generality",
];
fn source(name: &str, extension: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa31/tests/spec/drop-glue/{name}.{extension}")),
    )?)
}
#[test]
fn actual_source_glue_and_scope_cleanup_lower_without_placeholder_execution() -> TestResult {
    let mut failures = Vec::new();
    for &(name, _) in POSITIVE {
        match lower(&source(name, "rs")?) {
            Ok(_) => println!("{name}: accepted"),
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn actual_source_glue_preserves_const_borrow_and_binder_opposites() -> TestResult {
    for &name in NEGATIVE {
        assert!(lower(&source(name, "rs")?).is_err(), "{name}");
    }
    Ok(())
}
