//! Genuine owner/gate, source incarnation and metadata preservation opponents.
use crate::{
    pipeline,
    resolve::model::{DefId, DefKind},
    sema::header_attributes::ContractKnowledge,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/fundamental-policy";
fn source(name: &str) -> Result<String, std::io::Error> {
    crate::test_source::fixture(&format!("{PACKET}/{name}.rs"))
}
#[test]
fn source_policy_matches_the_independent_owner_gate_and_overlap_contract() -> TestResult {
    let rows = crate::test_source::fixture(&format!("{PACKET}/cases.tsv"))?;
    for row in rows.lines() {
        let (path, status) = row.split_once('\t').ok_or("fundamental inventory")?;
        let input = crate::test_source::fixture(path)?;
        let actual = crate::test_source::source_provider(&input, "fundamental_headers")
            .and_then(|r| pipeline::analyze_source(&r).map_err(|e| e.to_string()));
        if status == "0" {
            actual.map_err(|e| format!("{path}: {e}"))?;
        } else {
            assert!(actual.is_err(), "{path}");
        }
    }
    assert_eq!(rows.lines().count(), 34);
    Ok(())
}
#[test]
fn fundamental_policy_authenticates_defining_identity_and_source_image() -> TestResult {
    for (name, kind) in [("unit", DefKind::Struct), ("trait", DefKind::Trait)] {
        let mut report =
            crate::test_source::source_provider(&source(name)?, "fundamental_identity")?;
        let facts = pipeline::analyze_source(&report)?.facts;
        let id = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find(|(_, d)| {
                d.kind == kind && ["Wrap", "Bound"].contains(&report.package.names.get(d.name))
            })
            .map(|(i, _)| u32::try_from(i).map(DefId))
            .transpose()?
            .ok_or("fundamental owner")?;
        assert!(super::fundamental_policy(&facts, id)?);
        assert!(matches!(
            facts.fundamental_contract(&report, id)?,
            ContractKnowledge::KnownPresent(_)
        ));
        let key = report.resolution.definitions[id.0 as usize]
            .node
            .ok_or("source")?;
        report.package.units[key.file.0 as usize].source =
            report.package.unit(key.file).source.physical_copy();
        assert!(facts.fundamental_contract(&report, id).is_err());
    }
    Ok(())
}
#[test]
fn general_policy_is_not_lost_and_existing_sizing_metadata_remains_supported() -> TestResult {
    let base = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    crate::module_packets::exported(&format!("{base}\npub trait Ordinary{{}}"))?;
    for body in [
        "#[fundamental]pub trait Bound{}",
        "#[fundamental]pub struct Wrap{pub value:u8}",
    ] {
        let input = format!("#![feature(fundamental)]\n{base}\n{body}");
        let error = crate::module_packets::exported(&input)
            .err()
            .ok_or("fundamental policy erased")?
            .to_string();
        assert!(
            error.contains("fundamental policy requires authenticated"),
            "{error}"
        );
    }
    Ok(())
}
#[test]
fn failed_queries_keep_the_shared_source_budget_exhausted() -> TestResult {
    let report = crate::test_source::source_provider(&source("unit")?, "fundamental_budget")?;
    let facts = pipeline::analyze_source(&report)?.facts;
    assert!(
        crate::sema::header_attributes::charge_source_contract_query(&facts, usize::MAX).is_err()
    );
    assert!(facts.require_declaration_policy_transport(&report).is_err());
    assert!(crate::sema::header_attributes::charge_source_contract_query(&facts, 0).is_err());
    Ok(())
}
#[test]
fn ordinary_crates_do_not_need_a_source_provider_for_declaration_policy() -> TestResult {
    for source in [
        "#![feature(fundamental)]#[fundamental]struct Wrap;fn main(){}",
        "#![feature(fundamental)]#[fundamental]trait Bound{}fn main(){}",
        "#![feature(rustc_attrs)]#[rustc_specialization_trait]trait Bound{}fn main(){}",
    ] {
        let report = crate::test_source::resolve(source, true)?;
        pipeline::analyze_source(&report)?;
    }
    Ok(())
}
#[test]
fn marked_nominal_and_trait_dispatch_produce_actual_native_inputs() -> TestResult {
    let input = source("native-operation")?;
    let report = crate::test_source::source_provider(&input, "fundamental_native")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in ["selected", "ordinary"] {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|s| s.name == name && s.body.is_some())
                .count(),
            1
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/fundamental-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
