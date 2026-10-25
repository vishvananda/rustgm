//! Real producer, shared sort and forest work stay bounded before publication.
use super::*;
use crate::{
    module_packets as test, sema::crates::authored_export::Stage, support::crates::generics as g,
};
use std::fmt::Write;
use test::Result;
#[test]
fn complete_module_costs_prepay_sort_forest_transactions_and_enclosing_layouts() -> Result {
    sorting_attempts()?;
    growth()?;
    shared_parents()?;
    let interface = test::exported(&test::fixture("restricted-provider")?)?;
    let (rows, attrs, modules) = test::raw(&interface)?;
    let mut exhausted = Work {
        steps: crates::MAX_COMPONENTS,
        ..Work::default()
    };
    assert!(
        g::Payload::from_source_module_contracts(
            interface.identity(),
            rows.clone(),
            attrs.clone(),
            modules.clone(),
            0,
            &mut exhausted
        )
        .is_err()
    );
    assert!(
        g::Payload::from_source_module_contracts(
            interface.identity(),
            rows,
            attrs,
            modules,
            0,
            &mut Work::default()
        )
        .is_ok()
    );
    assert_eq!(size_of::<s::SourceRows>(), 616);
    assert_eq!(size_of::<s::Columns>(), 592);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::resolve::expansion::Report>(), 3040);
    assert_eq!(size_of::<s::TraitFlags>(), 2);
    println!(
        "v2 layouts ValidatedRows={} Payload={} Interface={} Columns={} SourceRows={} ModuleColumns={} ModuleRows={} Binding={} Domain={} ForestFrame={} Attributes={}",
        size_of::<s::ValidatedRows>(),
        size_of::<g::Payload>(),
        size_of::<crates::Interface>(),
        size_of::<s::Columns>(),
        size_of::<s::SourceRows>(),
        size_of::<ModuleContractColumns>(),
        size_of::<ModuleContractRows>(),
        size_of::<ModuleBindingRow>(),
        size_of::<TraitDomainRow>(),
        size_of::<module_index::Frame>(),
        size_of::<s::HeaderAttributeColumns>()
    );
    Ok(())
}
fn sorting_attempts() -> Result {
    for mut values in [Vec::<u32>::new(), vec![1]] {
        let before = values.clone();
        let mut failed = Work {
            steps: crates::MAX_COMPONENTS,
            ..Work::default()
        };
        assert!(
            super::super::binding_order::sort(&mut values, &mut failed, &mut 0, |a, b, w| {
                w.charge(1)?;
                Ok(a.cmp(b))
            })
            .is_err()
        );
        assert_eq!(failed.steps, crates::MAX_COMPONENTS + 1);
        assert_eq!(values, before);
        let mut fresh = Work::default();
        super::super::binding_order::sort(&mut values, &mut fresh, &mut 0, |a, b, w| {
            w.charge(1)?;
            Ok(a.cmp(b))
        })?;
        assert_eq!(fresh.steps, 1 + 4 * values.len());
    }
    Ok(())
}
fn growth() -> Result {
    let mut baseline = None;
    for width in [8_usize, 32, 128] {
        let mut source = crate::test_source::fixture(
            "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
        )?;
        source.push_str("pub mod owner { pub trait T {} }\n");
        for i in 0..width {
            writeln!(&mut source, "pub use owner::T as Alias_{i};")?;
        }
        let report = crate::test_source::source_provider(&source, "domain_growth")?;
        let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
        let identity = report
            .package
            .source_core()
            .ok_or("provider")?
            .provider
            .clone();
        let mut stage = Stage::begin_module_contracts(&report, &mut facts, &identity)?;
        let names = stage.module_contract_rows(&report, &facts)?;
        let observed = stage.work();
        assert_eq!(names.namespace.bindings.len(), 9 + width);
        assert!(observed.scratch_bytes >= 2 * names.namespace.bindings.len() * size_of::<usize>());
        let interface = test::exported(&source)?;
        let payload = interface.generic_payload().ok_or("payload")?;
        if let Some((declarations, steps, bytes)) = baseline {
            assert_eq!(stage.rows.declarations.len(), declarations);
            assert!(observed.steps <= steps + (width - 8) * 256);
            assert!(payload.retained_bytes() <= bytes + (width - 8) * 512 + 4096);
        } else {
            baseline = Some((
                stage.rows.declarations.len(),
                observed.steps,
                payload.retained_bytes(),
            ));
        }
        println!(
            "v2 alias width={width} emitterSteps={} retained={} peak={} sortScratch={}",
            observed.steps,
            payload.retained_bytes(),
            payload.peak_bytes(),
            observed.scratch_bytes
        );
    }
    Ok(())
}
fn shared_parents() -> Result {
    for width in [8_usize, 32, 128] {
        let mut source = crate::test_source::fixture(
            "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
        )?;
        for depth in 0..32 {
            writeln!(&mut source, "pub mod d{depth} {{")?;
        }
        for i in 0..width {
            writeln!(&mut source, "pub fn f{i}<T>(x:T)->T {{x}}")?;
        }
        source.push_str(&"}".repeat(32));
        let interface = test::exported(&source)?;
        let (rows, _, _) = test::raw(&interface)?;
        let mut header_work = Work::default();
        let headers = HeaderIndex::new(&rows, 0, &mut header_work)?;
        let mut work = Work::default();
        let _index = module_index::ModuleIndex::new(&rows, &headers, &mut work)?;
        assert!(work.steps <= rows.declarations.len() * 16 + 8);
        println!(
            "v2 sharedParent depth32 functions={width} declarations={} forestSteps={} scratch={}",
            rows.declarations.len(),
            work.steps,
            module_index::scratch_bytes(rows.declarations.len())
        );
    }
    Ok(())
}
