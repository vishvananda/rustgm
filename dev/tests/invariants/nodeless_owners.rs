//! A checked local root cannot turn an imported/missing header into empty data.
use super::*;
use crate::{
    pipeline,
    resolve::{model::DefKind, unit::FileId},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn imported_case() -> Result<(Report, pipeline::Analysis, DefId, DefId)> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let interface = crate::source_packets::exported(&source)?;
    let (report, analysis, _) = crate::source_packets::client_analysis(
        interface,
        "#![feature(no_core)]#![no_core]#![must_use]extern crate base;pub fn idle(){}",
    )?;
    let root = report.resolution.crate_root.ok_or("actual crate owner")?;
    let imported = report
        .resolution
        .crates
        .as_ref()
        .ok_or("actual imported inventory")?
        .definitions()
        .find_map(|(external, definition)| {
            matches!(external.path(), [name] if name == "identity").then_some(definition)
        })
        .ok_or("real imported header")?;
    Ok((report, analysis, root, imported))
}
fn rejected(report: &Report, facts: &Declarations, definition: DefId) {
    assert!(facts.must_use_contract(report, definition).is_err());
    assert!(
        facts
            .inline_hint_for_definition(report, definition)
            .is_err()
    );
}

#[test]
fn real_local_crate_owner_with_imports_is_unsupported_owner_not_unknown_header() -> Result {
    let (report, mut analysis, root, imported) = imported_case()?;
    assert!(
        report.resolution.definitions[root.0 as usize]
            .node
            .is_none()
    );
    assert!(
        report.resolution.definitions[imported.0 as usize]
            .node
            .is_none()
    );
    assert_eq!(analysis.facts.warnings().count(), 1);
    assert_eq!(
        analysis
            .facts
            .warnings()
            .next()
            .ok_or("wrong-owner warning")?
            .code
            .name(),
        "unused_attributes"
    );
    assert!(analysis.facts.must_use_contract(&report, root)?.is_none());
    assert!(
        analysis
            .facts
            .inline_hint_for_definition(&report, root)?
            .is_none()
    );
    assert!(
        analysis
            .facts
            .header_attribute_contracts(&report, root)
            .is_err(),
        "nonheader compatibility minted completed header absence"
    );
    rejected(&report, &analysis.facts, imported);
    let ledger = analysis
        .facts
        .must_use_contracts
        .as_deref_mut()
        .ok_or("cold contracts")?
        .headers
        .take();
    rejected(&report, &analysis.facts, imported);
    assert!(
        analysis
            .facts
            .header_attribute_contracts(&report, imported)
            .is_err()
    );
    analysis
        .facts
        .must_use_contracts
        .as_deref_mut()
        .ok_or("cold contracts")?
        .headers = ledger;
    Ok(())
}

#[test]
fn forged_crate_root_kind_scope_and_package_origin_do_not_waive_contract_queries() -> Result {
    let (mut report, analysis, root, imported) = imported_case()?;
    report.resolution.crate_root = Some(imported);
    rejected(&report, &analysis.facts, imported);
    report.resolution.crate_root = Some(root);
    report.resolution.definitions[root.0 as usize].kind = DefKind::Function;
    rejected(&report, &analysis.facts, root);
    report.resolution.definitions[root.0 as usize].kind = DefKind::Module;
    let members = report.resolution.definitions[root.0 as usize]
        .members
        .take();
    rejected(&report, &analysis.facts, root);
    report.resolution.definitions[root.0 as usize].members = members;
    let scope = report.resolution.module_scopes[0];
    report.resolution.scopes[scope.0 as usize].parent = Some(scope);
    rejected(&report, &analysis.facts, root);
    report.resolution.scopes[scope.0 as usize].parent = None;
    let file = report.package.modules[0].file;
    report.package.modules[0].file = FileId(u32::MAX);
    rejected(&report, &analysis.facts, root);
    report.package.modules[0].file = file;
    assert!(analysis.facts.must_use_contract(&report, root)?.is_none());
    assert!(
        analysis
            .facts
            .inline_hint_for_definition(&report, root)?
            .is_none()
    );
    Ok(())
}
