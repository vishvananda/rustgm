//! Implicit constructors retain current source/type authority and paid bounded queries.
use super::source_constructor;
use crate::{
    resolve::{expansion::Report, unit::NodeKey},
    sema::{declarations, header_attributes, model::Declarations},
    support::{bindings::LangItem, source::SourceFile},
    types::{Primitive, TypeId},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn prepared() -> TestResult<(Report, Declarations, NodeKey)> {
    let text = crate::test_source::fixture("pa31/tests/spec/inclusive-calls/renamed.rs")?;
    let report = crate::test_source::source_provider(&text, "inclusive_identity")?;
    let facts = declarations::analyze(&report)?;
    let site = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .language(LangItem::RangeInclusiveNew)
        .ok_or("constructor")?
        .origin;
    Ok((report, facts, site))
}
#[test]
fn constructor_rejects_foreign_source_store_and_changed_actual_header() -> TestResult {
    let (mut report, mut facts, site) = prepared()?;
    let method = source_constructor(&report, &facts, site)?;
    let (foreign, _, _) = prepared()?;
    assert!(source_constructor(&foreign, &facts, site).is_err());
    let store = std::mem::take(&mut facts.types);
    assert!(source_constructor(&report, &facts, site).is_err());
    facts.types = store;
    let index = *facts.function_indices.get(&method).ok_or("actual header")?;
    let signature = facts.functions[index].signature;
    facts.functions[index].signature = TypeId::primitive(Primitive::U8);
    assert!(source_constructor(&report, &facts, site).is_err());
    facts.functions[index].signature = signature;
    let replacement = SourceFile::new(report.package.units[0].source.text().as_bytes().to_vec())?;
    let original = std::mem::replace(&mut report.package.units[0].source, replacement);
    assert!(source_constructor(&report, &facts, site).is_err());
    report.package.units[0].source = original;
    assert_eq!(source_constructor(&report, &facts, site)?, method);
    Ok(())
}
#[test]
fn repeated_constructor_queries_keep_fixed_work_storage_and_sticky_real_exhaustion() -> TestResult {
    let (report, mut facts, site) = prepared()?;
    // Observation totals cannot consume this source producer's local allowance.
    facts.body_work.canonical_steps = 2 * crate::support::crates::MAX_COMPONENTS;
    let mut previous = None;
    for count in [8, 32, 128] {
        let before =
            header_attributes::source_contract_storage(&facts).ok_or("paid source owner")?;
        for _ in 0..count {
            source_constructor(&report, &facts, site)?;
        }
        let after =
            header_attributes::source_contract_storage(&facts).ok_or("paid source owner")?;
        assert_eq!(
            (before.1, before.2, before.3, before.4),
            (after.1, after.2, after.3, after.4)
        );
        let work = after.0 - before.0;
        if let Some(old) = previous {
            assert_eq!(work, 4 * old);
        }
        previous = Some(work);
    }
    assert!(
        header_attributes::charge_source_contract_query(
            &facts,
            crate::support::crates::MAX_COMPONENTS
        )
        .is_err()
    );
    let failed = header_attributes::source_contract_costs(&facts);
    assert!(source_constructor(&report, &facts, site).is_err());
    let retried = header_attributes::source_contract_costs(&facts);
    assert!(retried.0 > failed.0);
    assert_eq!(retried.1, failed.1);
    Ok(())
}
