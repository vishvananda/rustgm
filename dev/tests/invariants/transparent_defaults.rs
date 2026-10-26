//! Alias defaults are declaration obligations, even without an instantiated caller.
use super::*;

#[test]
fn unused_alias_defaults_check_written_clauses_without_expanding_dependencies() -> TestResult {
    for (case, expected) in [
        (
            "valid-direct",
            "trait alias contract expansion is not implemented for all consumers and export",
        ),
        (
            "valid-transitive",
            "trait alias contract expansion is not implemented for all consumers and export",
        ),
        (
            "no-copy-transitive",
            "trait alias contract expansion is not implemented for all consumers and export",
        ),
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/trait-aliases/default-arguments/{case}.rs"
        ))?;
        let report = report(&source)?;
        let Err(error) = crate::sema::declarations::analyze(&report) else {
            return Err("unfinished alias execution was published".into());
        };
        assert_eq!(error.message, expected, "{case}: {error}");
    }
    let source = crate::test_source::fixture(
        "pa31/tests/spec/trait-aliases/default-arguments/no-copy-direct.rs",
    )?;
    let report = report(&source)?;
    let Err(error) = crate::sema::declarations::analyze(&report) else {
        return Err("unused invalid default was accepted".into());
    };
    assert_eq!(
        error.message,
        "non-generic bound is not satisfied by a parameter default"
    );
    Ok(())
}
