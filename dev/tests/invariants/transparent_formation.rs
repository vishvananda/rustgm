//! Unused written aliases must still validate real nominal and region obligations.
use super::*;

#[test]
fn unused_alias_declarations_check_argument_equality_and_region_formation() -> TestResult {
    for (name, expected) in [
        (
            "invalid-trait-argument",
            "nominal type argument must implement Copy",
        ),
        (
            "invalid-equality-result",
            "nominal type argument must implement Copy",
        ),
        (
            "invalid-inherited-argument",
            "nominal type argument must implement Copy",
        ),
        (
            "closed-missing-trait",
            "closed trait bounds require an implementation proof",
        ),
        (
            "missing-lifetime-bound",
            "trait argument lifetime requirement is not satisfied",
        ),
        (
            "valid-lifetime-bound",
            "trait alias contract expansion is not implemented for all consumers and export",
        ),
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/trait-aliases/declaration-formation/{name}.rs"
        ))?;
        let report = report(&source)?;
        let error = crate::sema::declarations::analyze(&report)
            .err()
            .ok_or("alias validation unexpectedly passed its retained execution fence")?;
        assert!(error.message.contains(expected), "{name}: {error}");
    }
    for name in ["valid-trait-argument", "local-copy-consumer"] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/trait-aliases/declaration-formation/{name}.rs"
        ))?;
        crate::pipeline::analyze_source(&report(&source)?)?;
    }
    Ok(())
}
