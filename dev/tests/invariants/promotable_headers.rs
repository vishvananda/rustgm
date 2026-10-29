//! Real source promises retain owner/const witnesses without authorizing borrow promotion.
use super::*;
use crate::{
    pipeline,
    support::{crates::MAX_COMPONENTS, diagnostics::LintCap},
    test_source,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BASE: &str = "pa31/tests/spec/source-promotable/";
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!("{BASE}{name}.rs"))?)
}
fn report(source: &str, tag: &str) -> TestResult<Report> {
    let mut config = crate::expand::config::Config::default();
    config.set_lint_cap(LintCap::Allow);
    Ok(test_source::source_provider_config(source, tag, &config)?)
}
fn function(report: &Report, name: &str) -> TestResult<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| {
            d.kind == crate::resolve::model::DefKind::Function
                && report.package.names.get(d.name) == name
        })
        .ok_or_else(|| format!("missing actual source function {name}"))?;
    Ok(DefId(u32::try_from(index)?))
}
#[test]
fn actual_promotable_forms_keep_required_ctfe_and_ordinary_source_calls() -> TestResult {
    let source = input("source-runtime")?;
    let report = report(&source, "promotable_forms")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    for name in ["word", "list", "value", "unsafe_word"] {
        let definition = function(&report, name)?;
        let ContractKnowledge::KnownPresent(site) =
            facts.promotable_contract(&report, definition)?
        else {
            return Err("actual marker missing".into());
        };
        assert_eq!(
            site.declaration,
            report.resolution.definitions[definition.0 as usize]
                .node
                .ok_or("source owner")?
        );
    }
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/promotable-header-v1/candidate");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn source_marker_keeps_const_unsafe_and_promotion_boundaries_distinct() -> TestResult {
    for name in ["missing-pair", "nonconst", "unsafe-call"] {
        let source = input(name)?;
        let outcome = report(&source, name)
            .and_then(|r| pipeline::analyze_source(&r).map(|_| ()).map_err(Into::into));
        assert!(outcome.is_err(), "{name}");
    }
    let source = input("borrow-required")?;
    let report = report(&source, "promotable_borrow")?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("borrow promotion silently granted")?;
    assert!(
        error.to_string().contains("marked function-call promotion"),
        "{error}"
    );
    Ok(())
}
#[test]
fn marker_queries_reject_foreign_images_changed_flags_and_sticky_exhaustion() -> TestResult {
    let source = input("source-runtime")?;
    let report = report(&source, "promotable_queries")?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let word = function(&report, "word")?;
    let foreign = self::report(&source, "promotable_queries")?;
    assert!(facts.promotable_contract(&foreign, word).is_err());
    let ContractKnowledge::KnownPresent(site) = facts.promotable_contract(&report, word)? else {
        return Err("marker".into());
    };
    let cold = super::super::source_contract_storage(&facts).ok_or("header ledger")?;
    for _ in 0..128 {
        assert!(matches!(
            facts.promotable_contract(&report, word)?,
            ContractKnowledge::KnownPresent(_)
        ));
    }
    let warm = super::super::source_contract_storage(&facts).ok_or("header ledger")?;
    assert_eq!(cold.1, warm.1);
    assert_eq!(cold.5, size_of::<Header>());
    println!(
        "promotable Header={} Marker={} warm128work={} retainedgrowth={}",
        size_of::<Header>(),
        size_of::<Marker>(),
        warm.0 - cold.0,
        warm.1 - cold.1
    );
    let key = site.declaration;
    let marker = facts
        .must_use_contracts
        .as_deref_mut()
        .and_then(|c| c.headers.as_deref_mut())
        .and_then(|l| l.local.declarations.get_mut(&key))
        .and_then(|h| h.promotable.as_deref_mut())
        .ok_or("stored marker")?;
    let saved = marker.effect;
    marker.effect = usize::MAX - 1;
    assert!(facts.promotable_contract(&report, word).is_err());
    let marker = facts
        .must_use_contracts
        .as_deref_mut()
        .and_then(|c| c.headers.as_deref_mut())
        .and_then(|l| l.local.declarations.get_mut(&key))
        .and_then(|h| h.promotable.as_deref_mut())
        .ok_or("stored marker")?;
    marker.effect = saved;
    assert!(facts.promotable_contract(&report, word).is_ok());
    let headers = &facts
        .must_use_contracts
        .as_deref()
        .ok_or("contracts")?
        .headers
        .as_deref()
        .ok_or("headers")?
        .local;
    headers.steps.set(MAX_COMPONENTS);
    assert!(facts.promotable_contract(&report, word).is_err());
    assert!(facts.promotable_contract(&report, word).is_err());
    Ok(())
}
