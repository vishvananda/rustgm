//! Frozen Rust outcomes distinguish target authority, local ownership and Allow.
use super::*;
use crate::{pipeline, support::diagnostics::LintCap, test_source};
use std::fmt::Write;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BASE: &str = "pa31/tests/spec/source-incoherent-inherent/local/";
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!("{BASE}{name}.rs"))?)
}
fn source_report(source: &str) -> TestResult<Report> {
    let mut config = crate::expand::config::Config::default();
    config.set_lint_cap(LintCap::Allow);
    Ok(test_source::source_provider_config(
        source,
        "incoherent_targets",
        &config,
    )?)
}
fn definition(report: &Report, name: &str) -> TestResult<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|row| {
            matches!(
                row.kind,
                DefKind::Trait | DefKind::Struct | DefKind::Enum | DefKind::ForeignType
            ) && report.package.names.get(row.name) == name
        })
        .ok_or("actual target definition")?;
    Ok(DefId(u32::try_from(index)?))
}
#[test]
fn independently_declared_source_outcomes_keep_owning_target_and_call_permissions() -> TestResult {
    for (name, accepted) in [
        ("has-targets", true),
        ("has-word", true),
        ("has-missing-feature", false),
        ("has-list", false),
        ("has-value", false),
        ("has-duplicate", false),
        ("has-unsafe-wrapper", false),
        ("has-wrong-function", false),
        ("has-wrong-module", false),
        ("has-wrong-alias", false),
        ("has-wrong-static", false),
        ("has-wrong-impl", false),
        ("has-wrong-method", false),
        ("cfg-removed", true),
        ("macro-authorized", true),
        ("macro-unprivileged", false),
        ("macro-global-request", true),
        ("macro-captured-caller", false),
        ("alias-actual-owner", true),
        ("local-nominal-duplicate", false),
        ("allow-wrong-const", false),
        ("allow-wrong-trait-required", false),
        ("allow-wrong-trait-provided", false),
        ("allow-wrong-trait-impl", false),
        ("allow-list", false),
        ("allow-value", false),
        ("allow-duplicate", false),
        ("allow-no-unsafe-grant", false),
        ("allow-no-const-grant", false),
        ("auto-only", false),
        ("original-error-header-dyn", true),
    ] {
        let source = input(name)?;
        let result = if name == "original-error-header-dyn" {
            source_report(&source)
        } else {
            test_source::resolve(&source, true).map_err(Into::into)
        }
        .and_then(|report| {
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
            if matches!(name, "alias-actual-owner" | "original-error-header-dyn") {
                let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../obj/parallel/incoherent-targets-v1/candidate")
                    .join(name);
                std::fs::create_dir_all(&directory)?;
                std::fs::write(directory.join("source.rs"), &source)?;
                crate::lowir::writer::write(
                    &program,
                    &mut std::fs::File::create(directory.join("provider.lowir"))?,
                )?;
            }
            Ok::<_, Box<dyn std::error::Error>>(())
        });
        if let Err(error) = &result {
            println!("incoherent target {name}: {error}");
        }
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}
#[test]
fn marker_binds_actual_nominal_trait_and_foreign_definitions_without_alias_privilege() -> TestResult
{
    let report = test_source::resolve(&input("has-targets")?, true)?;
    let facts = pipeline::analyze_source(&report)?.facts;
    for name in ["T", "S", "E", "U", "Opaque"] {
        let id = definition(&report, name)?;
        let ContractKnowledge::KnownPresent(site) =
            facts.incoherent_inherent_contract(&report, id)?
        else {
            return Err(format!("missing Has on {name}").into());
        };
        assert_ne!(site.owner, site.declaration);
        assert_eq!(
            report.resolution.definitions[id.0 as usize].node,
            Some(site.declaration)
        );
    }
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
fn marker(facts: &mut Declarations, key: NodeKey) -> TestResult<&mut Marker> {
    Ok(facts
        .must_use_contracts
        .as_deref_mut()
        .and_then(|row| row.headers.as_deref_mut())
        .and_then(|ledger| ledger.local.declarations.get_mut(&key))
        .and_then(|header| header.promises.as_deref_mut())
        .and_then(|promises| promises.incoherent.as_mut())
        .ok_or("actual Has marker")?)
}
#[test]
fn source_query_rejects_replayed_image_owner_effect_kind_and_sticky_exhaustion() -> TestResult {
    let source = input("has-word")?;
    let report = test_source::resolve(&source, true)?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let id = definition(&report, "T")?;
    let foreign = test_source::resolve(&source, true)?;
    assert!(facts.incoherent_inherent_contract(&foreign, id).is_err());
    let ContractKnowledge::KnownPresent(site) = facts.incoherent_inherent_contract(&report, id)?
    else {
        return Err("actual target marker".into());
    };
    let saved = *marker(&mut facts, site.declaration)?;
    for changed in [
        Marker {
            owner_slot: usize::MAX,
            ..saved
        },
        Marker {
            effect: usize::MAX - 1,
            ..saved
        },
        Marker {
            target: Target::Struct,
            ..saved
        },
    ] {
        *marker(&mut facts, site.declaration)? = changed;
        assert!(facts.incoherent_inherent_contract(&report, id).is_err());
    }
    *marker(&mut facts, site.declaration)? = saved;
    let costs = super::super::source_contract_costs(&facts);
    for _ in 0..128 {
        assert!(matches!(
            facts.incoherent_inherent_contract(&report, id)?,
            ContractKnowledge::KnownPresent(_)
        ));
    }
    let warm = super::super::source_contract_costs(&facts);
    assert_eq!(warm.1, costs.1, "warm queries retain no new rows");
    println!(
        "Has warm128 work={} bytes={}",
        warm.0 - costs.0,
        warm.1 - costs.1
    );
    let local = &facts
        .must_use_contracts
        .as_deref()
        .and_then(|row| row.headers.as_deref())
        .ok_or("actual ledger")?
        .local;
    local.steps.set(crate::support::crates::MAX_COMPONENTS);
    assert!(facts.incoherent_inherent_contract(&report, id).is_err());
    assert!(facts.incoherent_inherent_contract(&report, id).is_err());
    Ok(())
}
#[test]
fn existing_scalar_and_source_metadata_cannot_drop_target_authority() -> TestResult {
    use crate::{sema::crates::authored_export::Stage, support::crates::CrateIdentity};
    let report = source_report(&input("original-error-header-dyn")?)?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let identity = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .current
        .clone();
    for begin in [
        Stage::begin,
        Stage::begin_with_ordinary_traits,
        Stage::begin_module_contracts,
        Stage::begin_module_availability,
        Stage::begin_module_feature_predicates,
        Stage::begin_module_definitions,
        Stage::begin_module_compatibility,
        Stage::begin_module_callables,
        Stage::begin_module_safety,
    ] {
        let error = begin(&report, &mut facts, &identity)
            .err()
            .ok_or("target authority silently dropped")?;
        assert!(error.contains("incoherent target authority"), "{error}");
    }
    let ordinary = test_source::resolve(&input("has-targets")?, true)?;
    let mut ordinary_facts = pipeline::analyze_source(&ordinary)?.facts;
    let error = crate::sema::crates::export(
        &ordinary,
        &mut ordinary_facts,
        CrateIdentity::new("target", "legacy")?,
    )
    .err()
    .ok_or("scalar export silently dropped Has")?;
    assert!(
        error.to_string().contains("incoherent target authority"),
        "{error}"
    );
    Ok(())
}
#[test]
fn sparse_target_retention_and_warm_queries_scale_linearly() -> TestResult {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source =
            "#![feature(rustc_attrs)]#![allow(internal_features,dead_code)]".to_owned();
        for index in 0..width {
            write!(
                source,
                "#[rustc_has_incoherent_inherent_impls]struct T{index};"
            )?;
        }
        let report = test_source::resolve(&source, true)?;
        let facts = pipeline::analyze_source(&report)?.facts;
        let initial = super::super::source_contract_costs(&facts);
        for index in 0..width {
            assert!(matches!(
                facts.incoherent_inherent_contract(
                    &report,
                    definition(&report, &format!("T{index}"))?
                )?,
                ContractKnowledge::KnownPresent(_)
            ));
        }
        let measured = super::super::source_contract_costs(&facts);
        assert_eq!(initial.1, measured.1);
        if let Some((steps, bytes)) = previous {
            assert!(
                measured.0 <= steps * 4 + 32,
                "work grew faster than source width"
            );
            assert!(
                measured.1 <= bytes * 4 + 256,
                "retention grew faster than source width"
            );
        }
        println!(
            "Has width={width} work={} bytes={} query={}",
            measured.0,
            measured.1,
            measured.0 - initial.0
        );
        previous = Some(measured);
    }
    Ok(())
}
