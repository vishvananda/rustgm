//! Implicit assignment syntax demands canonical calls without importing dot methods.
use super::{native, rustc};
use std::{path::PathBuf, process::Command};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const POSITIVE: &[&str] = &[
    "bound-reference", "returned-reference", "lexical-shadow", "module-shadow",
    "aliased-scalars", "call-order", "scalar-control", "no-demand", "disabled-demand",
    "boolean-forward", "all-families", "imported-dot",
];
const NEGATIVE: &[&str] = &[
    "mutable-rhs", "mixed-width", "boolean-add", "dot-before-demand", "dot-after-demand",
];
fn path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa26/tests/spec/implicit-assignment-demand/{name}.rs"))
}
#[test]
fn implicit_assignment_cli_outcomes_match_independent_rust() -> TestResult {
    for (&name, accepted) in POSITIVE.iter().map(|name| (name, true))
        .chain(NEGATIVE.iter().map(|name| (name, false))) {
        let source = std::fs::read_to_string(path(name))?;
        rustc::qualify(&format!("implicit-assignment-{name}"), &source, accepted)?;
        let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-lowir", "--crate-name", "compound"])
            .arg(path(name)).output()?;
        assert_eq!(result.status.code(), Some(i32::from(!accepted)), "{name}: {}",
            String::from_utf8_lossy(&result.stderr));
        if accepted {
            assert!(!result.stdout.is_empty() && result.stderr.is_empty(), "{name}");
        } else {
            assert!(String::from_utf8_lossy(&result.stderr).lines()
                .any(|line| line.starts_with("ERROR: ")), "{name}");
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned Rust and the qualified CPPGM backend"]
fn implicit_assignment_reference_calls_and_original_order_controls_execute() -> TestResult {
    for &name in POSITIVE {
        let source = std::fs::read_to_string(path(name))?;
        native::check_literal_cases("parallel/pa26/implicit-assignment-demand-v1/native", &[(name, &source, false)])?;
    }
    for (name, relative) in [
        ("implicit-original", "primitive-assignment/implicit-forwarding-boundary"),
        ("explicit-original", "primitive-assignment/explicit-forwarding"),
        ("primitive-order", "assignment/primitive-order"),
        ("trait-order", "assignment/trait-order"),
    ] {
        let source = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa26/tests/spec/{relative}.rs")))?;
        native::check_literal_cases("parallel/pa26/implicit-assignment-demand-v1/native", &[(name, &source, false)])?;
    }
    Ok(())
}
