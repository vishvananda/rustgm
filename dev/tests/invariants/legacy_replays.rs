//! Current source/typed signature identity and paid cold marker storage.
use super::*;
use std::fmt::Write as _;
#[test]
fn queries_reject_foreign_images_changed_signatures_store_and_lost_marker_exports() -> TestResult {
    let (report, mut analysis) = checked("owner-replay")?;
    let id = function(&report, "f")?;
    assert_eq!(analysis.facts.legacy_const_positions(&report, id)?, [0]);
    let foreign = test_source::resolve(&input("owner-replay")?, true)?;
    assert!(analysis.facts.legacy_const_positions(&foreign, id).is_err());
    let index = *analysis
        .facts
        .function_indices
        .get(&id)
        .ok_or("actual typed function")?;
    let old = analysis.facts.functions[index].signature;
    analysis.facts.functions[index].signature = crate::types::TypeId::UNIT;
    assert!(analysis.facts.legacy_const_positions(&report, id).is_err());
    analysis.facts.functions[index].signature = old;
    let key = analysis.facts.functions[index].node;
    let row = analysis
        .facts
        .must_use_contracts
        .as_deref_mut()
        .and_then(|c| c.headers.as_deref_mut())
        .and_then(|h| h.local.declarations.get_mut(&key))
        .and_then(|h| h.promises.as_deref_mut())
        .ok_or("actual source promises")?;
    let original = row.legacy.take().ok_or("actual legacy contract")?;
    assert!(
        analysis
            .facts
            .reject_function_advisory_transport(&report, id)
            .is_err()
    );
    analysis
        .facts
        .must_use_contracts
        .as_deref_mut()
        .and_then(|c| c.headers.as_deref_mut())
        .and_then(|h| h.local.declarations.get_mut(&key))
        .and_then(|h| h.promises.as_deref_mut())
        .ok_or("actual source promises")?
        .legacy = Some(original);
    let other = function(&report, "g")?;
    let row = analysis
        .facts
        .must_use_contracts
        .as_deref_mut()
        .and_then(|c| c.headers.as_deref_mut())
        .and_then(|h| h.local.declarations.get_mut(&key))
        .and_then(|h| h.promises.as_deref_mut())
        .and_then(|h| h.legacy.as_deref_mut())
        .ok_or("actual bound marker")?;
    row.bound = Some(other);
    assert!(analysis.facts.legacy_const_positions(&report, id).is_err());
    analysis
        .facts
        .must_use_contracts
        .as_deref_mut()
        .and_then(|c| c.headers.as_deref_mut())
        .and_then(|h| h.local.declarations.get_mut(&key))
        .and_then(|h| h.promises.as_deref_mut())
        .and_then(|h| h.legacy.as_deref_mut())
        .ok_or("actual bound marker")?
        .bound = Some(id);
    let old_store = std::mem::take(&mut analysis.facts.types);
    assert!(analysis.facts.legacy_const_positions(&report, id).is_err());
    analysis.facts.types = old_store;
    assert_eq!(analysis.facts.legacy_const_positions(&report, id)?, [0]);
    Ok(())
}
#[test]
fn repeated_and_failed_local_contract_queries_pay_actual_work_without_publishing_fakes()
-> TestResult {
    let (report, mut analysis) = checked("local-modern")?;
    let id = function(&report, "f")?;
    let first = super::super::super::source_contract_costs(&analysis.facts).0;
    for _ in 0..16 {
        assert_eq!(analysis.facts.legacy_const_positions(&report, id)?, [0]);
    }
    let warm = super::super::super::source_contract_costs(&analysis.facts).0;
    assert!(warm > first);
    let _ = super::super::super::reserve_source_contracts(
        &mut analysis.facts,
        crate::support::crates::MAX_COMPONENTS,
        0,
    );
    let before = super::super::super::source_contract_costs(&analysis.facts).0;
    assert!(analysis.facts.legacy_const_positions(&report, id).is_err());
    assert!(super::super::super::source_contract_costs(&analysis.facts).0 > before);
    assert!(analysis.facts.legacy_const_positions(&report, id).is_err());
    Ok(())
}
#[test]
fn cold_legacy_header_cost_is_linear_and_common_source_layouts_do_not_grow() -> TestResult {
    let mut previous = (0, 0);
    for width in [8, 32, 128] {
        let mut source = "#![feature(rustc_attrs)]\n".to_owned();
        for index in 0..width {
            writeln!(
                source,
                "#[rustc_legacy_const_generics(0)]pub fn f{index}<const N:usize>()->usize{{N}}"
            )?;
        }
        let report = test_source::resolve(&source, true)?;
        let analysis = pipeline::analyze_source(&report)?;
        let cost = super::super::super::source_contract_costs(&analysis.facts);
        println!(
            "legacy headers width={width} work={} bytes={}",
            cost.0, cost.1
        );
        if previous.0 != 0 {
            assert!(cost.0 < previous.0 * 6);
            assert!(cost.1 < previous.1 * 6);
        }
        previous = cost;
    }
    println!(
        "legacy Marker={} coldPromises={} Header={}",
        size_of::<Marker>(),
        size_of::<super::super::super::Promises>(),
        size_of::<Header>()
    );
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
