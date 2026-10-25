//! Fresh foreign impl checks differ from selection of provider-owned valid impls.
use crate::{
    module_packets as test, source_packets,
    support::crates::{self, generics::source as s},
};
use test::Result;
#[test]
fn fresh_module_clients_preserve_aliases_domains_provider_selection_and_advisories() -> Result {
    for (provider, client) in [
        ("root-ordinary-provider", "root-fresh-impl"),
        ("namespace-provider", "nested-fresh-impl"),
        ("restricted-provider", "sealed-public-bound"),
        ("restricted-provider", "select-provider-impl"),
    ] {
        let interface = test::roundtrip(&test::fixture(provider)?)?;
        source_packets::client(interface, &test::fixture(client)?)
            .map_err(|e| format!("{provider}/{client}: {e}"))?;
    }
    for client in [
        "sealed-foreign-impl",
        "sealed-foreign-feature-does-not-override",
        "sealed-private-path-denied",
    ] {
        let interface = test::roundtrip(&test::fixture("restricted-provider")?)?;
        let error = match source_packets::client_analysis(interface, &test::fixture(client)?) {
            Ok(_) => return Err(format!("unexpected accepted {client}").into()),
            Err(e) => e,
        };
        let text = error.to_string();
        if client == "sealed-private-path-denied" {
            assert!(text.contains("private"), "{text}");
        } else {
            assert!(text.contains("restricted module"), "{text}");
        }
    }
    canonical_domain()?;
    advisory_origins()?;
    let no_const = source_packets::client_analysis(
        test::roundtrip(&test::fixture("namespace-provider")?)?,
        &test::fixture("no-ctfe-permission")?,
    );
    assert!(no_const.is_err());
    Ok(())
}

fn canonical_domain() -> Result {
    let interface = test::roundtrip(&test::fixture("restricted-provider")?)?;
    let descriptor = interface.source_descriptor().ok_or("descriptor")?;
    let domain = descriptor
        .module_contracts()?
        .rows()
        .domains
        .iter()
        .find(|row| matches!(row.domain, s::ImplementationDomain::RestrictedTo { .. }))
        .ok_or("restricted trait")?;
    let (report, analysis, _) = source_packets::client_analysis(
        crates::decode(&crates::encode(&interface)?)?,
        &test::fixture("sealed-public-bound")?,
    )?;
    let imported = report.resolution.crates.as_ref().ok_or("imported")?;
    let definition = imported.generic_definition(interface.identity(), domain.declaration)?;
    assert_eq!(
        analysis
            .facts
            .associated
            .imported_implementation_domain(definition)?,
        domain.domain
    );
    assert!(
        !analysis
            .facts
            .associated
            .imported_denies_implementation(definition)?
    );
    assert!(
        imported
            .definitions()
            .all(|(_, actual)| actual != definition)
    );
    let rows = descriptor.rows();
    let module = rows.declarations[domain.defining_module.0 as usize]
        .name
        .as_deref();
    assert_eq!(module, Some("nonzero"));
    Ok(())
}

fn advisory_origins() -> Result {
    let root = test::roundtrip(&test::fixture("root-ordinary-provider")?)?;
    let row = root
        .source_descriptor()
        .ok_or("root descriptor")?
        .rows()
        .traits
        .iter()
        .find(|h| {
            root.source_descriptor().is_some_and(|d| {
                d.rows().declarations[h.declaration.0 as usize]
                    .name
                    .as_deref()
                    == Some("Open")
            })
        })
        .ok_or("ordinary Open header")?
        .declaration;
    let encoded = crates::encode(&root)?;
    let (real, real_analysis, _) = source_packets::client_analysis(
        crates::decode(&encoded)?,
        &test::fixture("root-fresh-impl")?,
    )?;
    let id = real
        .resolution
        .crates
        .as_ref()
        .ok_or("actual inventory")?
        .generic_definition(root.identity(), row)?;
    let (foreign, _, _) = source_packets::client_analysis(
        crates::decode(&encoded)?,
        &test::fixture("root-fresh-impl")?,
    )?;
    for _ in 0..8 {
        let view = real_analysis.facts.header_attribute_contracts(&real, id)?;
        assert!(matches!(
            view.must_use,
            crate::sema::header_attributes::ContractKnowledge::KnownPresent(_)
        ));
    }
    assert!(
        real_analysis
            .facts
            .header_attribute_contracts(&foreign, id)
            .is_err()
    );
    assert!(
        real_analysis
            .facts
            .header_attribute_contracts(&real, id)
            .is_ok()
    );
    Ok(())
}
