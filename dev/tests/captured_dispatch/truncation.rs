//! Checked source-root conditions survive shared-reference capture truncation.
use super::{checked, native};
use std::path::PathBuf;
const ACCEPTED: &[&str] = &[
    "after-scope",
    "generic-owner",
    "unused",
    "unused-valid-child",
    "nested-truncation",
];
fn fixture(name: &str) -> PathBuf {
    let path = if name == "original" {
        "captures/shared-reference-truncation.rs".to_string()
    } else {
        format!("formation-truncation/{name}.rs")
    };
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../pa24/tests/spec")
        .join(path)
}
#[test]
fn source_root_conditions_preserve_independent_outcomes() -> Result<(), Box<dyn std::error::Error>>
{
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/formation-truncation/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for name in std::iter::once("original").chain(ACCEPTED.iter().copied()) {
        checked::check(&fixture(name), &evidence, name, (0, 0, ""))?;
    }
    for name in [
        "reference-field",
        "local-field",
        "whole-root",
        "mutable-root",
    ] {
        checked::check_modes(
            &fixture(name),
            &evidence,
            name,
            (
                1,
                [0, 1, 1],
                "required lifetime does not outlive its target",
            ),
        )?;
    }
    checked::check_modes(
        &fixture("unused-invalid-child"),
        &evidence,
        "unused-invalid-child",
        (1, [0, 1, 1], "access conflicts with a live reference loan"),
    )?;
    // This independently accepted source remains an explicit checked boundary.
    checked::check_modes(
        &fixture("reference-result"),
        &evidence,
        "reference-result",
        (
            0,
            [0, 1, 1],
            "required lifetime does not outlive its target",
        ),
    )?;
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM backend; run make test-maintainer-native"]
fn source_root_conditions_execute_after_parent_storage_expires()
-> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "original",
        "after-scope",
        "generic-owner",
        "nested-truncation",
    ] {
        let source = std::fs::read_to_string(fixture(name))?;
        let body = source
            .strip_prefix(native::CHECK)
            .ok_or("missing check prefix")?;
        native::check_cases(
            "parallel/pa24/formation-truncation/native",
            &[(name, body, false)],
        )?;
    }
    Ok(())
}
