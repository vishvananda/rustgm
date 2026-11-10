//! The existing closure continuation owns paid cold casts without publishing proof.
use super::*;
use crate::{
    sema::declarations,
    types::{Primitive, infer::Domain},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn cast_nodes(report: &crate::resolve::expansion::Report) -> Vec<NodeId> {
    report.package.units[0]
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| matches!(node.kind, Kind::Cast { .. }))
        .filter_map(|(index, _)| u32::try_from(index).ok().map(NodeId::new))
        .collect()
}
#[test]
fn cold_cast_queue_keeps_original_order_and_known_casts_need_no_payload() -> TestResult {
    let text = crate::test_source::fixture("pa24/tests/spec/projection-casts/width-8-noise-0.rs")?;
    let report = crate::test_source::resolve(&text, true)?;
    let mut facts = declarations::analyze(&report)?;
    let nodes = cast_nodes(&report);
    let mut checker = Checker::new(&report, &mut facts);
    let known = checker
        .state
        .infer
        .known(&checker.facts.types, TypeId::primitive(Primitive::Usize))?;
    checker.state.work.push(Work::CollectClosure(None));
    assert!(!checker.defer_closure_cast(nodes[0], known, known)?);
    assert!(matches!(
        checker.state.work.first(),
        Some(Work::CollectClosure(None))
    ));
    let unknown = checker.state.infer.variable(Domain::Any)?;
    for &node in &nodes {
        assert!(checker.defer_closure_cast(node, unknown, known)?);
    }
    let Some(Work::CollectClosure(casts)) = checker.state.work.pop() else {
        return Err("actual continuation".into());
    };
    assert_eq!(casts.as_deref().map(Vec::as_slice), Some(nodes.as_slice()));
    checker.resume_closure_casts(casts)?;
    for expected in nodes {
        assert!(matches!(checker.state.work.pop(),Some(Work::Exit(node)) if node==expected));
    }
    assert!(checker.state.work.is_empty());
    Ok(())
}
#[test]
fn exhausted_source_payment_cannot_allocate_or_append_a_deferred_cast() -> TestResult {
    let text =
        crate::test_source::fixture("pa31/tests/spec/projection-casts/source-chain-called.rs")?;
    let report = crate::test_source::source_provider(&text, "deferred_casts")?;
    let mut facts = declarations::analyze(&report)?;
    let node = *cast_nodes(&report).last().ok_or("actual cast")?;
    let mut checker = Checker::new(&report, &mut facts);
    checker.state.work.push(Work::CollectClosure(None));
    let unknown = checker.state.infer.variable(Domain::Any)?;
    let known = checker
        .state
        .infer
        .known(&checker.facts.types, TypeId::primitive(Primitive::Usize))?;
    assert!(
        crate::sema::header_attributes::charge_source_contract_query(
            checker.facts,
            crate::support::crates::MAX_COMPONENTS
        )
        .is_err()
    );
    for _ in 0..2 {
        assert!(checker.defer_closure_cast(node, unknown, known).is_err());
        assert!(matches!(
            checker.state.work.first(),
            Some(Work::CollectClosure(None))
        ));
    }
    Ok(())
}
