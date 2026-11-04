//! Source applicability is a trait contract, never a spelling or a body permission.
use crate::{pipeline, resolve::model::DefKind};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/applicable-traits";
fn source(name: &str) -> Result<String, std::io::Error> {
    crate::test_source::fixture(&format!("{PACKET}/{name}.rs"))
}
#[test]
fn real_trait_headers_impls_and_specialization_use_the_independent_contract() -> TestResult {
    let cases = crate::test_source::fixture(&format!("{PACKET}/cases.tsv"))?;
    for line in cases.lines() {
        let (path, expected) = line.split_once('\t').ok_or("applicable case")?;
        let input = crate::test_source::fixture(path)?;
        let result = crate::test_source::source_provider(&input, "applicable_headers")
            .and_then(|r| pipeline::analyze_source(&r).map_err(|e| e.to_string()));
        if expected == "0" {
            result.map_err(|e| format!("{path}: {e}"))?;
        } else {
            assert!(result.is_err(), "{path}");
        }
    }
    assert_eq!(cases.lines().count(), 43);
    Ok(())
}
#[test]
fn claimed_policy_authenticates_actual_trait_image_and_preserves_marker_priority() -> TestResult {
    for (name, expected) in [("declaration", true), ("unsafe-priority", false)] {
        let mut report =
            crate::test_source::source_provider(&source(name)?, "applicable_identity")?;
        let analysis = pipeline::analyze_source(&report)?;
        let definition = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find(|(_, d)| d.kind == DefKind::Trait && report.package.names.get(d.name) == "Always")
            .and_then(|(i, _)| u32::try_from(i).ok().map(crate::resolve::model::DefId))
            .ok_or("actual applicable trait")?;
        assert_eq!(
            crate::sema::specialization::applicable(&analysis.facts, &report, definition)?,
            expected
        );
        let key = report.resolution.definitions[definition.0 as usize]
            .node
            .ok_or("actual trait source")?;
        report.package.units[key.file.0 as usize].source =
            report.package.unit(key.file).source.physical_copy();
        assert!(
            crate::sema::specialization::applicable(&analysis.facts, &report, definition).is_err()
        );
    }
    Ok(())
}
#[test]
fn applicability_queries_and_impl_work_share_sticky_failed_payment() -> TestResult {
    let report = crate::test_source::source_provider(&source("declaration")?, "applicable_budget")?;
    let analysis = pipeline::analyze_source(&report)?;
    let table = analysis
        .facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.specialization.as_deref())
        .ok_or("applicability owner")?;
    let alias = table.budget()?;
    assert!(alias.charge(10_000_001, 0).is_err());
    assert!(table.charge(0, 0).is_err());
    assert!(alias.charge(0, 0).is_err());
    Ok(())
}
#[test]
fn actual_applicability_policy_cannot_disappear_into_legacy_metadata() -> TestResult {
    let base = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    // The module profile already carries exact sizing-role flags. It must keep
    // that working path while rejecting unencoded policy on an ordinary trait.
    crate::module_packets::exported(&format!("{base}\npub trait Always{{}}"))?;
    let input = format!("{base}\n#[rustc_specialization_trait]pub trait Always{{}}");
    let error = crate::module_packets::exported(&input)
        .err()
        .ok_or("applicability policy was lost")?
        .to_string();
    assert!(
        error.contains("specialization-trait contract transport"),
        "{error}"
    );
    Ok(())
}
#[test]
fn original_marked_and_ordinary_trait_dispatch_reach_native_inputs() -> TestResult {
    let input = source("supertrait-operation")?;
    let report = crate::test_source::source_provider(&input, "applicable_native")?;
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
        .join("../obj/parallel/applicable-traits-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
