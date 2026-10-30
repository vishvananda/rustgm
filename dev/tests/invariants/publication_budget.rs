//! An exclusive structural query cannot roll back its actual exporter charges.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn structural_query_costs_survive_failure_and_sticky_work_or_storage_denial() -> Result {
    let text = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let report = crate::test_source::source_provider(&text, "query_payments")?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let mut stage = Stage::begin_module_conversions(&report, &mut facts, &identity)?;
    let foreign = crate::test_source::source_provider(&text, "foreign_query_payments")?;
    assert!(Stage::begin_module_conversions(&foreign, &mut facts, &identity).is_err());
    let changed = crate::test_source::source_provider(
        &format!("{text}\npub fn changed()->u8{{1}}"),
        "query_payments",
    )?;
    assert!(Stage::begin_module_conversions(&changed, &mut facts, &identity).is_err());
    let before = (stage.steps, stage.bytes);
    let rows = stage.rows.declarations.len();
    let result: std::result::Result<(), &str> = stage.paying(|pay| {
        pay(3, 16)?;
        Err("query failed after actual payment")
    });
    assert_eq!(result, Err("query failed after actual payment"));
    assert_eq!((stage.steps, stage.bytes), (before.0 + 4, before.1 + 16));
    stage.steps = crates::MAX_COMPONENTS;
    for _ in 0..2 {
        assert!(stage.paying(|pay| pay(1, 0)).is_err());
    }
    assert_eq!(stage.steps, crates::MAX_COMPONENTS + 2);
    stage.steps = 0;
    stage.bytes = crates::MAX_BYTES;
    for _ in 0..2 {
        assert!(stage.paying(|pay| pay(0, 1)).is_err());
    }
    assert_eq!(stage.bytes, crates::MAX_BYTES + 2);
    assert_eq!(stage.rows.declarations.len(), rows);
    Ok(())
}
