//! Actual foreign trait identity is established before ownership is rejected.
use super::Result;

pub(super) fn checked(source: &str) -> std::result::Result<crate::lowir::program::Program, String> {
    let mut report = crate::test_source::resolve(source, true)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
    let entry = crate::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared)
        .map_err(|e| e.to_string())
}

pub(super) fn foreign_trait_requires_real_imported_owner() -> Result {
    let fixture =
        |name| crate::test_source::fixture(&format!("pa29/tests/spec/local-dyn-owner/{name}.rs"));
    let interface = crate::module_packets::roundtrip(&fixture("provider")?)?;
    let imported = crate::support::crates::decode(&crate::support::crates::encode(&interface)?)?;
    let (report, _, _) =
        crate::source_packets::client_analysis(imported, &fixture("bare-client")?)?;
    let actual: Vec<_> = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .filter(|(_, definition)| {
            definition.kind == crate::resolve::model::DefKind::Trait
                && report.package.names.get(definition.name) == "Open"
        })
        .collect();
    assert_eq!(actual.len(), 1, "one genuine imported principal");
    let (index, definition) = actual[0];
    assert!(
        definition.node.is_none(),
        "foreign trait must not acquire source authority"
    );
    println!("foreign dyn actual imported principal DefId({index})");
    let Err(error) =
        crate::source_packets::client_analysis(interface, &fixture("foreign-inherent")?)
    else {
        return Err("foreign inherent impl unexpectedly acquired local ownership".into());
    };
    assert!(error.to_string().contains(
        "foreign object inherent permission and lookup require their wider authenticated profile"
    ), "{error}");
    Ok(())
}
