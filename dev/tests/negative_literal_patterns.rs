//! A `no_core` negative literal pattern requires the `neg` language item and
//! `T: Neg`, as pinned rustc does; positive ranges need neither.
use std::{path::Path, process::Command};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Declared and frozen in `contract.json` before any candidate run.
const CASES: &[(&str, Option<&str>)] = &[
    (
        "missing-range",
        Some("source - requires its actual Neg language declaration"),
    ),
    (
        "missing-literal",
        Some("source - requires its actual Neg language declaration"),
    ),
    ("positive-range", None),
    ("ordinary-impl", None),
    ("unimplemented", Some("trait obligation requires")),
];

fn types(name: &str) -> Result<std::process::Output> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/source-negative-patterns")
        .join(format!("{name}.rs"));
    Ok(Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            "--emit-types",
            "--source-core",
            "core@h00000000000000000001",
            "--crate-name",
            "core",
            "--crate-disambiguator",
            "h00000000000000000001",
        ])
        .arg(path)
        .output()?)
}

#[test]
fn negative_literal_patterns_require_the_source_neg_item_and_bound() -> Result {
    for &(name, cause) in CASES {
        let output = types(name)?;
        let stderr = String::from_utf8(output.stderr)?;
        match cause {
            None => assert!(output.status.success(), "{name}: {stderr}"),
            Some(cause) => {
                assert_eq!(output.status.code(), Some(1), "{name}: {stderr}");
                assert!(output.stdout.is_empty(), "{name}");
                assert!(stderr.contains(cause), "{name}: {stderr}");
            }
        }
    }
    Ok(())
}
