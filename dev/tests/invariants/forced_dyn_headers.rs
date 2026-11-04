//! Real declaration policy, cached inherited rejection and ordinary execution.
use crate::{
    pipeline,
    resolve::model::{DefId, DefKind},
    sema::header_attributes::ContractKnowledge,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/forced-dyn-policy";
fn source(name: &str) -> Result<String, std::io::Error> {
    crate::test_source::fixture(&format!("{PACKET}/{name}.rs"))
}
fn definition(
    report: &crate::resolve::expansion::Report,
    name: &str,
) -> Result<DefId, Box<dyn std::error::Error>> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.kind == DefKind::Trait && report.package.names.get(d.name) == name)
        .map(|(i, _)| u32::try_from(i).map(DefId))
        .transpose()?
        .ok_or_else(|| "forced dyn trait".into())
}
#[test]
fn source_restriction_matches_independent_formation_and_declaration_outcomes() -> TestResult {
    let rows = crate::test_source::fixture(&format!("{PACKET}/cases.tsv"))?;
    let mut mismatches = Vec::new();
    for row in rows.lines() {
        let (path, status) = row.split_once('\t').ok_or("forced dyn inventory")?;
        let input = crate::test_source::fixture(path)?;
        let actual =
            crate::test_source::source_provider(&input, "forced_dyn_headers").and_then(|r| {
                pipeline::analyze_source(&r)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            });
        if actual.is_ok() != (status == "0") {
            mismatches.push(format!("{path}: {actual:?}"));
        }
    }
    assert_eq!(rows.lines().count(), 40);
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    Ok(())
}
#[test]
fn distinct_policies_share_the_actual_header_without_changing_their_meanings() -> TestResult {
    for name in ["declaration", "both-policies"] {
        let mut report =
            crate::test_source::source_provider(&source(name)?, "forced_dyn_identity")?;
        let facts = pipeline::analyze_source(&report)?.facts;
        let id = definition(&report, "Forced")?;
        assert!(super::forced_dyn_policy(&facts, id)?);
        assert_eq!(
            super::fundamental_policy(&facts, id)?,
            name == "both-policies"
        );
        assert!(matches!(
            facts.dyn_restriction_contract(&report, id)?,
            ContractKnowledge::KnownPresent(_)
        ));
        let key = report.resolution.definitions[id.0 as usize]
            .node
            .ok_or("source")?;
        report.package.units[key.file.0 as usize].source =
            report.package.unit(key.file).source.physical_copy();
        assert!(facts.dyn_restriction_contract(&report, id).is_err());
    }
    Ok(())
}
#[test]
fn source_policy_budget_cannot_be_restarted_or_converted_into_absence() -> TestResult {
    let input = format!("{}\ntrait Ordinary{{}}", source("declaration")?);
    let report = crate::test_source::source_provider(&input, "forced_dyn_budget")?;
    let facts = pipeline::analyze_source(&report)?.facts;
    let forced = definition(&report, "Forced")?;
    let ordinary = definition(&report, "Ordinary")?;
    assert!(
        crate::sema::header_attributes::charge_source_contract_query(&facts, usize::MAX).is_err()
    );
    assert!(super::forced_dyn_policy(&facts, forced).is_err());
    assert!(super::forced_dyn_policy(&facts, ordinary).is_err());
    assert!(super::fundamental_policy(&facts, ordinary).is_err());
    assert!(facts.require_declaration_policy_transport(&report).is_err());
    Ok(())
}
#[test]
fn existing_sizing_flags_roundtrip_and_general_policy_cannot_disappear() -> TestResult {
    let base = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    crate::module_packets::exported(&base)?;
    let error = crate::module_packets::exported(&format!(
        "{base}\n#[rustc_dyn_incompatible_trait]pub trait Forced{{}}"
    ))
    .err()
    .ok_or("forced dyn policy erased")?
    .to_string();
    assert!(
        error.contains("forced dyn policy requires authenticated"),
        "{error}"
    );
    Ok(())
}
