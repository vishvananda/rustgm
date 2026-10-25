//! Genuine source publication, complete contracts and fresh imported consumers.
use super::*;
use crate::{
    lower::instances::{SourcePublication, build_source_library_profile},
    pipeline,
    resolve::model::DefKind,
    source_packets,
    support::crates::{
        self,
        generics::{self as g, source as s},
    },
};
#[path = "versioned_attributes.rs"]
mod compatibility;
#[path = "origin_integrity.rs"]
mod origins;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&if name == "plain" {
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs".to_owned()
    } else {
        format!("pa31/tests/spec/source-publication-attributes/{name}.rs")
    })?)
}
fn exported(source: &str) -> Result<(crates::Interface, usize)> {
    exported_profile(source, SourcePublication::FunctionContracts)
}
fn exported_profile(
    source: &str,
    profile: SourcePublication,
) -> Result<(crates::Interface, usize)> {
    let report = crate::test_source::source_provider(source, "attribute_transport")?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual SourceCore provider")?
        .provider
        .clone();
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let (_, interface) =
        build_source_library_profile(&report, &mut facts, identity, prepared, profile)?;
    Ok((interface, facts.warnings().count()))
}
fn roundtrip(source: &str) -> Result<crates::Interface> {
    let (interface, _) = exported(source)?;
    assert_eq!(
        interface
            .source_descriptor()
            .ok_or("actual descriptor")?
            .profile(),
        s::Profile::Source2024FunctionContracts
    );
    let bytes = crates::encode(&interface)?;
    assert_eq!(bytes.get(12), Some(&10));
    assert_eq!(bytes.get(13..15), Some([1, 0].as_slice()));
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    Ok(decoded)
}
fn imported_definition(report: &Report, name: &str) -> Result<DefId> {
    report
        .resolution
        .crates
        .as_ref()
        .ok_or("actual imported inventory")?
        .definitions()
        .find_map(|(external, definition)| {
            external
                .path()
                .last()
                .is_some_and(|actual| actual == name)
                .then_some(definition)
        })
        .ok_or_else(|| format!("missing actual imported {name}").into())
}
fn idle(policy: &str, statement: &str) -> String {
    format!("#![feature(no_core)]#![no_core]{policy}extern crate base;pub fn idle(){{{statement}}}")
}

#[test]
fn marked_and_plain_providers_roundtrip_into_fresh_used_and_discarded_consumers() -> Result {
    for (provider, warnings) in [("plain", 0), ("must-use-reason", 1)] {
        let interface = roundtrip(&fixture(provider)?)?;
        let used = crate::test_source::fixture(
            "pa31/tests/spec/source-functions-transport/runtime-client.rs",
        )?;
        let _program =
            source_packets::client(crates::decode(&crates::encode(&interface)?)?, &used)?;
        let source = idle("", "base::identity(7u8);");
        let (report, analysis, _) = source_packets::client_analysis(interface, &source)?;
        let definition = imported_definition(&report, "identity")?;
        let view = analysis
            .facts
            .header_attribute_contracts(&report, definition)?;
        assert!(view.origin.source().is_none());
        let origin = view.origin.imported().ok_or("genuine imported origin")?;
        assert!(origin.site().is_some());
        assert_eq!(
            origin
                .external_definition()
                .ok_or("real function identity")?
                .path(),
            &["identity"]
        );
        assert_eq!(analysis.facts.warnings().count(), warnings);
        if warnings == 0 {
            assert!(matches!(view.must_use, ContractKnowledge::KnownAbsent));
        } else {
            let warning = analysis
                .facts
                .warnings()
                .next()
                .ok_or("actual discarded warning")?;
            assert_eq!(
                warning.code,
                crate::support::diagnostics::WarningCode::UnusedMustUse
            );
            assert_eq!(
                warning.diagnostic.message,
                "unused must_use result: keep result"
            );
            assert_eq!(
                warning.diagnostic.file.source(),
                Some(crate::resolve::unit::FileId(0))
            );
            assert_eq!(
                warning.diagnostic.span.start,
                u32::try_from(source.find("base::identity").ok_or("call site")?)?
            );
        }
    }
    Ok(())
}

#[test]
fn imported_contract_warnings_follow_consumer_lint_levels_and_explicit_discard() -> Result {
    let bytes = crates::encode(&roundtrip(&fixture("must-use-reason")?)?)?;
    for (policy, statement, count) in [
        ("", "let _ = base::identity(7u8);", 0),
        ("#![allow(unused_must_use)]", "base::identity(7u8);", 0),
        ("#![warn(unused_must_use)]", "base::identity(7u8);", 1),
        (
            "#![deny(unused_must_use)]",
            "#[allow(unused_must_use)]{base::identity(7u8);}",
            0,
        ),
    ] {
        let (_, analysis, _) =
            source_packets::client_analysis(crates::decode(&bytes)?, &idle(policy, statement))?;
        assert_eq!(analysis.facts.warnings().count(), count);
    }
    for policy in ["#![deny(unused_must_use)]", "#![forbid(unused_must_use)]"] {
        let error = source_packets::client_analysis(
            crates::decode(&bytes)?,
            &idle(policy, "base::identity(7u8);"),
        )
        .err()
        .ok_or("discarded imported result bypassed deny/forbid")?;
        assert!(
            error
                .to_string()
                .contains("unused must_use result: keep result")
        );
    }
    let error = source_packets::client_analysis(
        crates::decode(&bytes)?,
        &idle(
            "#![forbid(unused_must_use)]",
            "#[allow(unused_must_use)]{base::identity(7u8);}",
        ),
    )
    .err()
    .ok_or("forbid was weakened")?;
    assert!(error.to_string().contains("forbid"));
    Ok(())
}
