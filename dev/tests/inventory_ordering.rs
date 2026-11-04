//! Request completion must use the source inventory's actual finite payer.
use crate::resolve as test_source;
use rustgm::{
    expand::config::{Config, SourceContractLimits},
    resolve::model::DefKind,
    support::diagnostics::LintCap,
};
use std::fmt::Write as _;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn actual_source_inventory_crosses_the_metadata_ceiling_without_losing_its_owner() -> TestResult {
    let mut source = test_source::fixture("pa31/tests/spec/inventory-budget/provider.rs")?;
    for index in 0..131_072 {
        writeln!(source, "pub fn item_{index:05}(){{}}")?;
    }
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    config.set_source_contract_limits(SourceContractLimits::new(
        14_500_624,
        1_073_741_824,
        67_108_864,
    ));
    let report = test_source::source_provider_config(&source, "request_order", &config)?;
    let inventory = report
        .package
        .library_policy()
        .and_then(|library| library.inventory())
        .ok_or("unpublished source inventory")?;
    assert!(inventory.steps() > 1_000_000);
    assert!(inventory.steps() < 14_500_624);
    assert_eq!(
        report
            .resolution
            .definitions
            .iter()
            .filter(|definition| definition.kind == DefKind::Function)
            .count(),
        131_073
    );
    // The same input cannot acquire a new payer after configured exhaustion.
    config.set_source_contract_limits(SourceContractLimits::new(
        1_000_000,
        1_073_741_824,
        67_108_864,
    ));
    let error = test_source::source_provider_config(&source, "request_order", &config)
        .err()
        .ok_or("source work exhaustion was ignored")?;
    assert!(
        error.contains("source macro contracts exceed their work budget"),
        "{error}"
    );
    Ok(())
}

#[test]
fn ordinary_feature_validation_still_requires_completed_inventory() -> TestResult {
    test_source::resolve("fn main(){}", true)?;
    let error = test_source::resolve("#![feature(unknown_source_gate)]fn main(){}", true)
        .err()
        .ok_or("undefined feature request was accepted")?;
    assert!(
        error.contains("undefined library feature request"),
        "{error}"
    );
    Ok(())
}
