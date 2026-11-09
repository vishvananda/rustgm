//! A client's executable demand before the eager walk pays each imported
//! header's conformance once. An imported body's goal reads the coarse heads
//! bucket and pays for every primitive-headed Copy impl; a local body's
//! resolved goal pays only for its own. Counts come from the declared outcomes.
use crate::{
    lower::instances::SourcePublication, module_packets, sema::copy_trace::Capture, source_packets,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PREFIX: &[&str] = &["demanded conformance"];
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa29/tests/spec/scheduling-context/{name}.rs"
    ))?)
}
/// Every demanded owner of one fresh client analysis, in demand order.
fn demanded(provider: &str, client: &str) -> Result<Vec<String>> {
    let interface = module_packets::exported_profile(
        &fixture(provider)?,
        SourcePublication::ModuleBodies,
        "scheduling_context",
    )?;
    let source = fixture(client)?;
    let capture = Capture::matching(PREFIX)?;
    let analysis = source_packets::client_analysis(interface, &source);
    let rows = capture.rows()?;
    drop(capture);
    analysis?;
    Ok(rows)
}
#[test]
fn demanded_conformance_pays_each_imported_owner_once() -> Result {
    for (client, expected) in [("four", 14), ("eight", 14), ("local-placed", 1)] {
        let rows = demanded("primitives-provider", client)?;
        assert_eq!(rows.len(), expected, "{client}: {rows:?}");
        let mut owners = rows.clone();
        owners.sort_unstable();
        owners.dedup();
        assert_eq!(owners.len(), rows.len(), "{client} paid an owner twice");
    }
    // A provider without a matching or primitive Copy impl demands nothing.
    assert!(demanded("nominal-provider", "four")?.is_empty());
    Ok(())
}
/// A repeat demand of an owner that an earlier demand or the walk already
/// validated returns before the specialization gate: it charges the client's
/// graph nothing and pays no conformance again.
#[test]
fn repeat_demand_of_a_validated_owner_charges_nothing() -> Result {
    let interface = module_packets::exported_profile(
        &fixture("usize-provider")?,
        SourcePublication::ModuleBodies,
        "scheduling_context",
    )?;
    let report = source_packets::client_report(interface, &fixture("minspec")?)?;
    let mut checker = super::Checker::new(&report)?;
    checker
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    checker.run()?;
    let registry = checker
        .facts
        .generics
        .implementations
        .as_deref()
        .ok_or("actual client registry")?;
    let imported: Vec<_> = registry
        .headers()
        .filter(|header| header.source.is_none())
        .map(|header| header.owner)
        .collect();
    assert!(imported.len() >= 3, "imported impls {imported:?}");
    assert!(imported.iter().all(|&owner| registry.is_validated(owner)));
    let graph_work = |facts: &crate::sema::model::Declarations| {
        facts
            .generics
            .implementations
            .as_deref()
            .and_then(|registry| registry.specialization_graph())
            .map(|graph| graph.observation().work)
    };
    let before = graph_work(&checker.facts).ok_or("the minspec client keeps its graph")?;
    let capture = Capture::matching(&["demanded conformance", "imported conformance"])?;
    for &owner in &imported {
        checker.advance_impl_validation(owner, super::Phase::Begin)?;
    }
    let rows = capture.rows()?;
    drop(capture);
    assert!(rows.is_empty(), "a repeat demand paid again: {rows:?}");
    assert_eq!(graph_work(&checker.facts), Some(before));
    Ok(())
}
