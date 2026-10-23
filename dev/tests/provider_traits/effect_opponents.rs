//! Structural signatures do not grant source callable const eligibility.
use super::{TestResult, lower, report};
#[test]
fn source_callable_effect_opponents_match_their_actual_targets() -> TestResult {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/source-const-payment/callable");
    let mut mismatches = Vec::new();
    for (name, expected) in [
        ("unused-const-function-pointer", false),
        ("ordinary-function-pointer", true),
        ("const-function-item", true),
        ("nonconst-function-item", false),
        ("nonconst-invocation", false),
    ] {
        let source = std::fs::read_to_string(folder.join(format!("{name}.rs")))?;
        let analysis = rustgm::pipeline::analyze_source(&report(&source)?);
        println!(
            "{name}: analysis={} diagnostic={}",
            analysis.is_ok(),
            analysis
                .as_ref()
                .err()
                .map_or(String::new(), ToString::to_string)
        );
        let result = lower(&source);
        println!(
            "{name}: complete_lowering={} diagnostic={}",
            result.is_ok(),
            result
                .as_ref()
                .err()
                .map_or(String::new(), ToString::to_string)
        );
        if result.is_ok() != expected {
            mismatches.push(name);
        }
    }
    assert!(
        mismatches.is_empty(),
        "callable effect mismatches: {mismatches:?}"
    );
    Ok(())
}
