//! Unchanged original sources keep their actual feature scope through concrete dispatch.
use crate::{body, pipeline, resolve, sema::dispatch::Dispatcher};
use std::sync::Arc;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn analyzed(name: &str) -> TestResult<(resolve::expansion::Report, pipeline::Analysis)> {
    let source = crate::test_source::fixture(&format!(
        "pa31/tests/spec/feature-bound-predicates/{name}.rs"
    ))?;
    let report = crate::test_source::resolve(&source, true)?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn observed(report: &resolve::expansion::Report) -> TestResult<resolve::model::DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| {
            definition.kind == resolve::model::DefKind::Function
                && report.package.names.get(definition.name) == "observed"
        })
        .ok_or("actual observed source export")?;
    Ok(resolve::model::DefId(u32::try_from(index)?))
}
#[test]
fn unchanged_matching_and_generic_forward_sources_reach_lowir() -> TestResult {
    for name in ["matching-caller", "generic-forward-guarded"] {
        let (
            report,
            pipeline::Analysis {
                mut facts,
                prepared,
            },
        ) = analyzed(name)?;
        let exports = [observed(&report)?];
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &exports,
            },
            prepared,
        )?;
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == "observed" && symbol.body.is_some())
        );
        let mut output = Vec::new();
        crate::lowir::writer::write(&program, &mut output)?;
        assert!(!output.is_empty());
        println!(
            "feature-native {name} symbols={} bytes={}",
            program.symbols.len(),
            output.len()
        );
    }
    Ok(())
}
#[test]
fn concrete_dispatch_keeps_actual_scope_and_avoids_global_cache() -> TestResult {
    let (report, pipeline::Analysis { mut facts, .. }) = analyzed("matching-caller")?;
    let owner = observed(&report)?;
    let program = body::build_concrete(&report, &facts)?;
    let body = program
        .bodies
        .iter()
        .find(|body| {
            matches!(body.owner,
        body::ir::Owner::Function {definition,..} if definition==owner)
        })
        .ok_or("actual observed body")?;
    let contract = body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|contract| contract.key == body.key())
        .ok_or("actual observed contract")?;
    let target = body
        .values
        .iter()
        .find(|value| body::dispatch::trait_function(&facts, value))
        .ok_or("actual trait callable operand")?
        .ty;
    let mut selected = super::prepare(&mut facts, body, &contract, 1)?;
    selected.select(&mut facts, target)?;
    assert!(
        Dispatcher::new(contract.universals, 1)?
            .select(&mut facts, target)
            .is_err()
    );
    let other = facts
        .types
        .intern_feature_charged("unrelated_dispatch", &|_, _| Ok(()))?;
    let environment = Arc::new(
        crate::types::parameters::environment::Environment::new(&facts.types, &[])?
            .with_feature_assumptions_charged(&facts.types, &[other], &|_, _| Ok(()))?,
    );
    let mut unrelated = Dispatcher::in_body(
        &facts.types,
        contract.universals,
        1,
        environment,
        &|_, _| Ok(()),
    )?;
    assert!(unrelated.select(&mut facts, target).is_err());
    let other_source = program
        .bodies
        .iter()
        .find(|other| other.source != body.source)
        .ok_or("opposing actual body source")?
        .source;
    let mut wrong_body = body.clone();
    let mut wrong_contract = contract.clone();
    wrong_body.source = other_source;
    wrong_contract.source = other_source;
    assert!(super::prepare(&mut facts, &wrong_body, &wrong_contract, 1).is_err());
    for name in ["missing-caller", "unrelated-caller"] {
        let error = analyzed(name)
            .err()
            .ok_or("missing source caller accepted")?
            .to_string();
        assert!(error.contains("feature"), "{name}: {error}");
        assert!(!error.contains("not implemented"), "{name}: {error}");
    }
    Ok(())
}
