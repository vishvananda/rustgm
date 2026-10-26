//! Marker facts retain actual owners; absence and publication never become privileges.
use super::*;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
    },
    sema::{declarations, model::Declarations},
    support::crates,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn definition(report: &Report, name: &str) -> Result<DefId> {
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, actual)| {
            actual.kind == DefKind::Trait && report.package.names.get(actual.name) == name
        })
        .ok_or("actual source trait declaration")?;
    Ok(DefId(u32::try_from(index)?))
}
fn analyzed(text: &str) -> Result<(Report, Declarations)> {
    let report = crate::test_source::resolve(text, true)?;
    let facts = declarations::analyze(&report)?;
    Ok((report, facts))
}
#[test]
fn marker_authority_retains_actual_source_and_rejects_invalid_or_forwarded_requests() -> Result {
    let text = "#![feature(rustc_attrs)]trait Plain{}#[rustc_unsafe_specialization_marker]trait Fast{}fn main(){}";
    let (mut report, facts) = analyzed(text)?;
    let fast = definition(&report, "Fast")?;
    let plain = definition(&report, "Plain")?;
    assert!(marker(&facts, &report, fast)?);
    assert!(!marker(&facts, &report, plain)?);
    assert!(
        facts
            .generics
            .implementations
            .as_deref()
            .is_none_or(|registry| registry.specialization_graph().is_none())
    );
    let key = report.resolution.definitions[fast.0 as usize]
        .node
        .ok_or("real marker node")?;
    report.resolution.definitions[fast.0 as usize].node =
        report.resolution.definitions[plain.0 as usize].node;
    assert!(
        marker(&facts, &report, fast).is_err(),
        "different real owner accepted"
    );
    report.resolution.definitions[fast.0 as usize].node = Some(key);
    report.package.units[key.file.0 as usize].source =
        report.package.unit(key.file).source.physical_copy();
    assert!(
        marker(&facts, &report, fast)
            .err()
            .ok_or("foreign image accepted")?
            .message
            .contains("incarnation")
    );
    marker_source_requests()?;
    marker_costs()?;
    failed_budget_aliases_remain_cumulative()?;
    Ok(())
}
fn marker_source_requests() -> Result {
    for (source, message) in [
        (
            "#[rustc_unsafe_specialization_marker]trait Fast{}fn main(){}",
            "requires rustc_attrs",
        ),
        (
            "#![feature(rustc_attrs)]#[rustc_unsafe_specialization_marker(reason)]trait Fast{}fn main(){}",
            "requires word form",
        ),
        (
            "#![feature(rustc_attrs)]#[rustc_unsafe_specialization_marker]struct S;fn main(){}",
            "actual local trait",
        ),
        (
            "#![feature(rustc_attrs)]#[rustc_unsafe_specialization_marker]#[rustc_unsafe_specialization_marker]trait Fast{}fn main(){}",
            "duplicate rustc_unsafe_specialization_marker",
        ),
    ] {
        let report = crate::test_source::resolve(source, true)?;
        let error = declarations::analyze(&report)
            .err()
            .ok_or("invalid marker accepted")?;
        assert!(error.message.contains(message), "{error}");
    }
    for (name, permitted) in [
        ("marker-literal-allowance", true),
        ("marker-forwarded-allowance", false),
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/specialization-graph/initial/{name}.rs"
        ))?;
        let report = crate::test_source::resolve(&source, true)?;
        let result = declarations::analyze(&report);
        if permitted {
            let facts = result?;
            assert!(marker(&facts, &report, definition(&report, "Fast")?)?);
        } else {
            assert!(
                result
                    .err()
                    .ok_or("forwarded caller gained marker authority")?
                    .message
                    .contains("requires rustc_attrs")
            );
        }
    }
    assert!(crate::test_source::resolve("#[unknown_marker]trait Fast{}fn main(){}", true).is_err());
    Ok(())
}
fn marker_costs() -> Result {
    use std::fmt::Write;
    let (_, ordinary) = analyzed("trait Plain{}fn main(){}")?;
    assert!(
        ordinary
            .must_use_contracts
            .as_deref()
            .is_none_or(|owner| owner.specialization.is_none())
    );
    for width in [8_usize, 32, 128] {
        let mut text = String::from("#![feature(rustc_attrs)]");
        for index in 0..width {
            write!(
                &mut text,
                "#[rustc_unsafe_specialization_marker]trait M{index}{{}}"
            )?;
        }
        let (report, facts) = analyzed(&text)?;
        let table = facts
            .must_use_contracts
            .as_deref()
            .and_then(|owner| owner.specialization.as_deref())
            .ok_or("actual cold marker owner")?;
        let (rows, capacity, steps, bytes) = table.storage();
        assert!(table.complete);
        assert_eq!(rows, width);
        assert_eq!(table.definitions.len(), width);
        assert!(steps < 256 * width + 64);
        assert!(bytes < 4096 * width + 8192);
        for index in 0..width {
            assert!(marker(
                &facts,
                &report,
                definition(&report, &format!("M{index}"))?
            )?);
        }
        eprintln!(
            "marker width={width} rows={rows} capacity={capacity} work={steps} logical_bytes={bytes} LocalMarkers={} Contracts={}",
            size_of::<LocalMarkers>(),
            size_of::<crate::sema::must_use::Contracts>()
        );
    }
    Ok(())
}
#[test]
fn marker_publication_is_closed_and_imported_contracts_remain_unknown() -> Result {
    use crate::lower::instances::{SourcePublication, build_source_library_profile};
    let base = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let plain = format!("{base}\npub trait ProbeMarker{{}}\n");
    let marked =
        format!("{base}\n#[rustc_unsafe_specialization_marker]pub trait ProbeMarker{{}}\n");
    let interface = crate::module_packets::exported_profile(
        &plain,
        SourcePublication::ModuleContracts,
        "marker_publication",
    )?;
    assert_eq!(
        interface
            .source_descriptor()
            .ok_or("actual descriptor")?
            .profile(),
        crates::SourceProfile::Source2024ModuleContracts
    );
    let report = crate::test_source::source_provider(&marked, "marker_publication")?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let actual = definition(&report, "ProbeMarker")?;
    assert!(marker(&analysis.facts, &report, actual)?);
    let guard = analysis
        .facts
        .publication_header_contracts(&report, actual)
        .err()
        .ok_or("untransported selected trait accepted")?;
    assert_eq!(
        guard.message,
        "source unsafe-specialization-marker contract transport is not implemented"
    );
    let error = build_source_library_profile(
        &report,
        &mut analysis.facts,
        identity,
        analysis.prepared,
        SourcePublication::ModuleContracts,
    )
    .err()
    .ok_or("module publisher dropped real marker contract")?;
    assert!(
        error
            .message
            .contains("unsafe-specialization-marker contract transport"),
        "{error}"
    );
    let descriptor = interface.source_descriptor().ok_or("actual descriptor")?;
    let declaration = descriptor
        .rows()
        .traits
        .first()
        .ok_or("actual imported trait")?
        .declaration;
    let identity = descriptor.identity().clone();
    let (report, analysis, _) = crate::source_packets::client_analysis(
        interface,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn unused(){}",
    )?;
    let imported = report
        .resolution
        .crates
        .as_ref()
        .ok_or("actual imported inventory")?
        .generic_definition(&identity, declaration)?;
    assert!(
        report.resolution.definitions[imported.0 as usize]
            .node
            .is_none()
    );
    let error = marker(&analysis.facts, &report, imported)
        .err()
        .ok_or("imported absence inferred")?;
    assert!(error.message.contains("unknown or untransported"));
    Ok(())
}

fn failed_budget_aliases_remain_cumulative() -> Result {
    use super::retention_work::Budget;
    let budget = Budget::new(64)?;
    let alias = budget.alias()?;
    let before = budget.usage();
    assert!(alias.charge(10_000_000, 0).is_err());
    let failed = budget.usage();
    assert!(failed.0 > before.0);
    assert!(budget.alias().is_err());
    assert!(alias.charge(1, 16).is_err());
    let retried = budget.usage();
    assert!(retried.0 > failed.0);
    assert_eq!(retried.1, failed.1 + 16);
    eprintln!(
        "shared graph budget Budget={} before={before:?} failed={failed:?} retried={retried:?}",
        size_of::<Budget>()
    );
    Ok(())
}
