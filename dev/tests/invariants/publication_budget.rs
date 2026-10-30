//! An exclusive structural query cannot roll back its actual exporter charges.
use super::*;
use crate::{
    resolve::{expansion::Report, model::DefKind},
    sema::model::Declarations,
    types::{Primitive, TypeId},
};
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn alias_stage(suffix: &str) -> Result<(Report, Declarations, Stage)> {
    let text = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let report =
        crate::test_source::source_provider(&format!("{text}\n{suffix}"), "alias_payments")?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let stage = Stage::begin_module_conversions(&report, &mut facts, &identity)?;
    Ok((report, facts, stage))
}
fn alias_provenance() -> Result {
    let (report, mut facts, mut stage) = alias_stage("type Hidden=u8;")?;
    let alias = report
        .resolution
        .definitions
        .iter()
        .position(|definition| definition.kind == DefKind::TypeAlias)
        .ok_or("actual alias declaration")?;
    let actual = Some(TypeId::primitive(Primitive::U8));
    assert_eq!(facts.definitions[alias], actual);
    assert_eq!(stage.definitions[alias], None);
    let rows = stage.module_contract_rows(&report, &facts)?;
    for changed in [None, Some(TypeId::UNIT)] {
        facts.definitions[alias] = changed;
        assert!(stage.module_contract_rows(&report, &facts).is_err());
    }
    facts.definitions[alias] = actual;
    assert_eq!(
        stage
            .module_contract_rows(&report, &facts)?
            .namespace
            .bindings
            .len(),
        rows.namespace.bindings.len()
    );
    let identity = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    assert!(Stage::begin_module_safety(&report, &mut facts, &identity).is_err());
    // These are valid Rust declarations with an explicitly wider metadata
    // requirement, not independently observed language-negative fixtures.
    for suffix in [
        "pub type Exposed=u8;",
        "type Generic<T>=T;",
        "type Generic<'a>=&'a u8;",
        "trait AliasOwner{type Hidden;}",
    ] {
        assert!(alias_stage(suffix).is_err(), "{suffix}");
    }
    let (report, facts, mut stage) = alias_stage("type Hidden=u8;use Hidden as Rebound;")?;
    assert!(stage.module_contract_rows(&report, &facts).is_err());
    Ok(())
}
fn alias_scaling() -> Result {
    let (report, facts, mut ordinary) = alias_stage("")?;
    let ordinary_bindings = ordinary
        .module_contract_rows(&report, &facts)?
        .namespace
        .bindings
        .len();
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut source = String::new();
        for index in 0..width {
            writeln!(source, "type Hidden{index}=u8;")?;
        }
        let (report, facts, mut stage) = alias_stage(&source)?;
        let rows = stage.module_contract_rows(&report, &facts)?;
        assert_eq!(rows.namespace.bindings.len(), ordinary_bindings);
        observations.push((width, stage.steps, stage.bytes));
        stage.steps = crates::MAX_COMPONENTS;
        assert!(stage.module_contract_rows(&report, &facts).is_err());
    }
    for pair in observations.windows(2) {
        let [(a, aw, ab), (b, bw, bb)] = pair else {
            unreachable!()
        };
        assert!(bw * a <= aw * b + 256 * b);
        assert!(bb * a <= ab * b + 256 * b);
    }
    eprintln!("private alias closure width/work/bytes: {observations:?}");
    Ok(())
}
#[test]
fn structural_query_costs_survive_failure_and_sticky_work_or_storage_denial() -> Result {
    alias_provenance()?;
    alias_scaling()?;
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
