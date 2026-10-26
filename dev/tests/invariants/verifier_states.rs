//! Only a completed immutable forest admits member execution; ordinary analysis stays available.
use super::*;
use crate::sema::{specialization, traits::implementations::Implementations};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn frozen(lane: &str, name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/specialization-graph/{lane}/{name}.rs"
    ))?)
}
fn verified(name: &str, source: &str, expected_vertices: usize, depth: u32) -> TestResult {
    let report = crate::test_source::resolve(source, true)?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    let registry = checker
        .facts
        .generics
        .implementations
        .as_deref()
        .ok_or("actual registry")?;
    let graph = registry.specialization_graph().ok_or("actual graph")?;
    let before = graph.observation();
    assert!(before.sealed);
    assert_eq!(before.vertices, expected_vertices);
    assert_eq!(before.edges, expected_vertices - 1);
    assert_eq!(before.depth, depth);
    assert!(
        registry
            .headers()
            .all(|row| registry.is_validated(row.owner))
    );
    specialization::prepare_graph(&mut checker.facts, &report)?;
    let graph = checker
        .facts
        .generics
        .implementations
        .as_deref()
        .and_then(Implementations::specialization_graph)
        .ok_or("same graph")?;
    let after = graph.observation();
    assert_eq!(
        after.bytes, before.bytes,
        "warm closure allocated another graph"
    );
    assert!(after.work > before.work);
    assert!(after.work - before.work < 16);
    assert!(after.work < 100_000);
    assert!(after.bytes < 4 * 1024 * 1024);
    eprintln!(
        "verified forest {name} vertices={} edges={} depth={} capacity={} work={} logical_bytes={} Graph={} Registry={}",
        after.vertices,
        after.edges,
        after.depth,
        after.capacity,
        after.work,
        after.bytes,
        size_of::<specialization::Graph>(),
        size_of::<Implementations>()
    );
    compatible_insertion_is_closed(&checker.facts, after.root.ok_or("actual graph root")?)?;
    Ok(())
}
fn compatible_insertion_is_closed(facts: &Declarations, root: DefId) -> TestResult {
    let registry = facts
        .generics
        .implementations
        .as_deref()
        .ok_or("same registry")?;
    let header = registry.header(root).ok_or("actual root header")?;
    let (application, _) = header
        .predicate
        .bound
        .trait_parts()
        .ok_or("real trait application")?;
    let identity = crate::sema::traits::implementations::Identity::User(
        facts
            .types
            .trait_application(application)
            .ok_or("real application")?
            .item,
    );
    assert!(
        registry
            .specialization_graph()
            .ok_or("sealed graph")?
            .require_insertion(identity, None)
            .is_err()
    );
    Ok(())
}
fn rejected(source: &str, expected: &str) -> TestResult {
    let report = crate::test_source::resolve(source, true)?;
    let error = crate::sema::declarations::analyze(&report)
        .err()
        .ok_or("invalid graph accepted")?;
    assert!(
        error.message.contains(expected),
        "expected {expected}: {error}"
    );
    Ok(())
}
#[test]
fn specialization_verifier_checks_immediate_finality_min_authority_and_closed_execution()
-> TestResult {
    for name in [
        "chain-and-inherited-body-explicit-middle",
        "reverse-chain-order-explicit-middle",
    ] {
        verified(name, &frozen("corrected-chains", name)?, 3, 2)?;
    }
    for name in [
        "actual-unsafe-marker-selection",
        "full-static-structured-self",
        "literal-impl-graph-allowance",
    ] {
        verified(name, &frozen("initial", name)?, 2, 1)?;
    }
    for (name, message) in [
        ("chain-and-inherited-body", "final inherited member"),
        ("reverse-chain-order", "final inherited member"),
        ("full-plus-min-static", "introduces a static lifetime"),
        ("min-literal-full-static", "introduces a static lifetime"),
        ("full-lifetime-only-order", "equivalent"),
        ("unmarked-name-has-no-authority", "non-marker predicate"),
        ("distinct-marker-identity", "non-marker predicate"),
    ] {
        rejected(&frozen("initial", name)?, message)?;
    }
    let report =
        crate::test_source::resolve("trait V{}impl V for u8{}impl V for u16{}fn main(){}", true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    assert!(
        facts
            .generics
            .implementations
            .as_deref()
            .ok_or("ordinary registry")?
            .specialization_graph()
            .is_none()
    );
    rejected(
        "trait V{}impl<T>V for T{}impl V for u8{}fn main(){}",
        "overlapping trait implementation headers require coherence proof",
    )?;
    layouts();
    Ok(())
}
fn layouts() {
    use crate::{
        literal::storage::StoredValue,
        syntax::{
            ast::{Node, Syntax},
            item::Item,
        },
    };
    eprintln!(
        "specialization enclosing Syntax={} Unit={} Report={} Node={} Item={} Scope={} Definition={} NodeFacts={} Contracts={}",
        size_of::<Syntax<StoredValue>>(),
        size_of::<crate::resolve::unit::Unit>(),
        size_of::<Report>(),
        size_of::<Node>(),
        size_of::<Item>(),
        size_of::<crate::resolve::model::Scope>(),
        size_of::<crate::resolve::model::Definition>(),
        size_of::<crate::sema::model::NodeFacts>(),
        size_of::<crate::sema::must_use::Contracts>()
    );
}
