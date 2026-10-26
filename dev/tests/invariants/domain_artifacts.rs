//! Complete wire facts and old byte contracts cannot acquire default policy.
use crate::{
    lower::instances::SourcePublication,
    module_packets as test,
    support::crates::{
        self,
        generics::{self as g, source as s},
    },
};
use test::Result;
#[test]
fn complete_domains_namespace_advisories_and_old_bytes_are_versioned() -> Result {
    old_bytes()?;
    advisory_columns()?;
    let interface = test::roundtrip(&test::fixture("restricted-provider")?)?;
    let descriptor = interface.source_descriptor().ok_or("actual descriptor")?;
    assert!(!descriptor.profile().ordinary_functions());
    let view = descriptor.module_contracts()?;
    assert_eq!(view.rows().domains.len(), descriptor.rows().traits.len());
    assert!(
        view.rows()
            .domains
            .iter()
            .any(|row| matches!(row.domain, s::ImplementationDomain::RestrictedTo { .. }))
    );
    malformed_rows(&interface)?;
    let bytes = crates::encode(&interface)?;
    for (tag, version) in [(10, 0), (10, 1), (11, 2), (10, 99)] {
        let mut changed = bytes.clone();
        changed[12] = tag;
        changed[13..15].copy_from_slice(&u16::to_le_bytes(version));
        assert!(crates::decode(&changed).is_err());
    }
    for end in [bytes.len() - 1, bytes.len() / 2] {
        assert!(crates::decode(&bytes[..end]).is_err());
    }
    Ok(())
}
fn old_bytes() -> Result {
    let plain = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    for (source, profile, golden) in [
        (
            plain.clone(),
            SourcePublication::LegacyFunctions,
            "pa31/tests/spec/source-attribute-transport/legacy-functions-v0.hex",
        ),
        (
            plain,
            SourcePublication::FunctionContracts,
            "pa31/tests/spec/module-contracts-v2/legacy-v1-plain.hex",
        ),
        (
            crate::test_source::fixture(
                "pa31/tests/spec/source-publication-attributes/must-use-reason.rs",
            )?,
            SourcePublication::FunctionContracts,
            "pa31/tests/spec/module-contracts-v2/legacy-v1-marked.hex",
        ),
        (
            crate::test_source::fixture(
                "pa31/tests/spec/source-publication-attributes/required-inline.rs",
            )?,
            SourcePublication::FunctionContracts,
            "pa31/tests/spec/module-contracts-v2/legacy-v1-bodyless_inline.hex",
        ),
    ] {
        let interface = test::exported_profile(&source, profile, "attribute_transport")?;
        let bytes = crates::encode(&interface)?;
        assert_eq!(bytes, test::hex(golden)?, "{golden}");
        assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    }
    Ok(())
}

fn advisory_columns() -> Result {
    let marked = crate::test_source::fixture(
        "pa31/tests/spec/source-publication-attributes/must-use-reason.rs",
    )?;
    let interface = test::roundtrip(&marked)?;
    let rows = interface.source_descriptor().ok_or("v2 descriptor")?.rows();
    let attributes = rows.header_attributes().ok_or("v2 complete attributes")?;
    assert_eq!(attributes.functions.len(), rows.functions.len());
    assert_eq!(attributes.traits.len(), rows.traits.len());
    assert!(attributes.functions.iter().any(|row| {
        row.must_use
            .as_ref()
            .is_some_and(|v| v.reason.as_deref() == Some("keep result"))
    }));
    let bodyless = crate::test_source::fixture(
        "pa31/tests/spec/source-publication-attributes/required-inline.rs",
    )?;
    let interface = test::roundtrip(&bodyless)?;
    assert!(
        interface
            .source_descriptor()
            .ok_or("v2 inline descriptor")?
            .rows()
            .header_attributes()
            .ok_or("v2 inline columns")?
            .functions
            .iter()
            .any(|row| row
                .inline
                .is_some_and(|v| v.applicability == s::InlineApplicability::DiagnosedNoBody))
    );
    Ok(())
}

fn malformed_rows(interface: &crates::Interface) -> Result {
    let (rows, attributes, modules) = test::raw(interface)?;
    for case in 0..10 {
        let mut changed = modules.clone();
        match case {
            0 => {
                changed.domains.pop();
            }
            1 => changed.domains.push(changed.domains[0]),
            2 => changed.domains[0].declaration_site = u32::MAX,
            3 => changed.domains[0].defining_module = changed.domains[0].declaration,
            4 => {
                let row = changed
                    .domains
                    .iter_mut()
                    .find(|row| matches!(row.domain, s::ImplementationDomain::RestrictedTo { .. }))
                    .ok_or("restricted row")?;
                row.domain = s::ImplementationDomain::Unrestricted;
            }
            5 => changed.namespace.bindings[0].target.artifact = s::ArtifactRef(1),
            6 => changed.namespace.bindings[0].source_site = u32::MAX,
            7 => changed
                .namespace
                .bindings
                .push(changed.namespace.bindings[0].clone()),
            8 => {
                changed.namespace.modules.pop();
            }
            _ => {
                changed.namespace.bindings[0].namespace =
                    match changed.namespace.bindings[0].namespace {
                        crates::Namespace::Type => crates::Namespace::Value,
                        crates::Namespace::Value => crates::Namespace::Type,
                    }
            }
        }
        assert!(
            g::Payload::from_source_module_contracts(
                interface.identity(),
                rows.clone(),
                attributes.clone(),
                changed,
                0,
                &mut g::Work::default()
            )
            .is_err(),
            "domain corruption {case}"
        );
    }
    let mut missing = attributes.clone();
    missing.traits.pop();
    assert!(
        g::Payload::from_source_module_contracts(
            interface.identity(),
            rows.clone(),
            missing,
            modules,
            0,
            &mut g::Work::default()
        )
        .is_err()
    );
    assert!(
        g::Payload::from_source(
            interface.identity(),
            rows.clone(),
            0,
            &mut g::Work::default()
        )
        .is_err()
    );
    assert!(
        g::Payload::from_source_contracts(
            interface.identity(),
            rows.clone(),
            attributes,
            0,
            &mut g::Work::default()
        )
        .is_err()
    );
    assert!(
        g::Payload::from_source_provided(interface.identity(), rows, 0, &mut g::Work::default())
            .is_err()
    );
    Ok(())
}
