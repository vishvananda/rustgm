//! A closed universal call must pay its scoped proof before semantic acceptance.
use super::{TestResult, cli};
const CASES: &[(&str, i32)] = &[
    ("static-unique", 1),
    ("static-forward", 1),
    ("static-with-caller", 1),
    ("outlives-caller", 1),
    ("universal-unique", 0),
    ("assumed-concrete", 0),
    ("assumed-through-capture", 0),
    ("capture-outlives-valid", 0),
    ("capture-outlives-invalid", 1),
];
#[test]
fn closed_universal_calls_distinguish_static_implementations_from_actual_caller_premises()
-> TestResult {
    for &(name, expected) in CASES {
        let text = crate::resolve::fixture(&format!(
            "pa21/tests/spec/binders/universal-inherent/{name}.rs"
        ))?;
        let result = cli("--emit-generic-semantics", &text)?;
        assert_eq!(
            result.status.code(),
            Some(expected),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    Ok(())
}
