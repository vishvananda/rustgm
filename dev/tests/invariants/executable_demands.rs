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
