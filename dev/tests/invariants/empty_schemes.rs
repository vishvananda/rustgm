//! Empty source domains are completed declarations, never lookup defaults.
use super::*;
use crate::{source_packets, support::crates, test_source};
use std::{fmt::Write as _, sync::Arc};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn source(extra: &str) -> Result<String> {
    let mut source =
        test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    source.push_str(extra);
    Ok(source)
}
fn named(
    facts: &crate::sema::model::Declarations,
    report: &crate::resolve::expansion::Report,
    name: &str,
) -> Result<crate::sema::model::Function> {
    facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == name
        })
        .copied()
        .ok_or_else(|| "actual named function".into())
}
fn empty(scheme: &FunctionRegions) {
    assert_eq!(scheme.total, 0);
    assert_eq!(scheme.captures, 0);
    assert!(scheme.early.is_empty());
    assert!(scheme.late.is_empty());
    assert!(!scheme.has_declared_late);
    assert!(scheme.bounds.is_empty());
}
#[test]
fn completed_source_zero_domain_retains_actual_scheme() -> Result {
    let report = test_source::source_provider(
        &source(
            "pub fn keep(value:u8)->u8{value}pub fn empty(){} pub fn unused<'a>(value:u8)->u8{value}",
        )?,
        "empty_domain",
    )?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    for name in ["keep", "empty"] {
        let function = named(&checker.facts, &report, name)?;
        empty(
            checker
                .facts
                .lifetimes
                .functions
                .get(&function.definition)
                .ok_or("completed zero scheme")?,
        );
        assert!(!checker.function_binders.contains_key(&function.node));
        let header = checker
            .facts
            .function_header(function.definition)?
            .ok_or("actual source header")?;
        assert_eq!(header.universals, 0);
        assert_eq!(header.signature, function.signature);
        assert_eq!(
            header.body,
            function
                .body
                .map(crate::resolve::body_origin::BodyOrigin::Source)
        );
        match (
            header.environment,
            checker.facts.generics.body_environment(function.definition),
        ) {
            (Some(actual), Some(expected)) => assert!(Arc::ptr_eq(actual, expected)),
            (None, None) => {}
            _ => return Err("header changed its actual defining environment presence".into()),
        }
    }
    let unused = named(&checker.facts, &report, "unused")?;
    let scheme = checker
        .facts
        .lifetimes
        .functions
        .get(&unused.definition)
        .ok_or("actual declared lifetime domain")?;
    assert_eq!(scheme.total, 1);
    assert!(scheme.has_declared_late);
    Ok(())
}
#[test]
fn zero_domain_source_functions_cross_codec_and_fresh_proof() -> Result {
    let interface =
        source_packets::exported(&source("pub fn keep(value:u8)->u8{value}pub fn empty(){}")?)?;
    let bytes = crates::encode(&interface)?;
    assert_eq!(bytes.get(12), Some(&10));
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    let (report, mut analysis, current) = source_packets::client_analysis(
        decoded,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn answer()->u8{base::keep(7u8)}",
    )?;
    assert!(report.package.source_client().is_some());
    let function = named(&analysis.facts, &report, "answer")?;
    empty(
        analysis
            .facts
            .lifetimes
            .functions
            .get(&function.definition)
            .ok_or("actual client zero scheme")?,
    );
    let (program, _) = crate::lower::instances::build_library_prepared(
        &report,
        &mut analysis.facts,
        current,
        analysis.prepared,
    )?;
    assert!(program.symbols.iter().any(|symbol| symbol.body.is_some()));
    Ok(())
}
#[test]
fn missing_or_foreign_scheme_never_becomes_checked_empty() -> Result {
    let report = test_source::source_provider(
        &source("pub fn keep(v:u8)->u8{v}pub fn other(v:u8)->u8{v}")?,
        "empty_opponent",
    )?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    let function = named(&checker.facts, &report, "keep")?;
    let other = named(&checker.facts, &report, "other")?;
    assert_eq!(function.signature, other.signature);
    let scheme = checker
        .facts
        .lifetimes
        .functions
        .remove(&function.definition)
        .ok_or("genuine completed scheme")?;
    checker.facts.lifetimes.functions.remove(&other.definition);
    let mut steps = 0;
    let before = checker.facts.lifetimes.functions.len();
    assert!(
        checker
            .retain_empty_scheme(
                CompletedEmpty {
                    owner: other.definition,
                    source: function.node,
                    scheme: scheme.clone(),
                },
                &mut steps,
                usize::MAX
            )
            .is_err()
    );
    assert_eq!(steps, 1);
    assert_eq!(checker.facts.lifetimes.functions.len(), before);
    let capacity = checker.facts.lifetimes.functions.capacity();
    let mut exhausted = 0;
    assert!(
        checker
            .retain_empty_scheme(
                CompletedEmpty {
                    owner: function.definition,
                    source: function.node,
                    scheme,
                },
                &mut exhausted,
                0
            )
            .is_err()
    );
    assert_eq!(exhausted, 1);
    assert_eq!(checker.facts.lifetimes.functions.capacity(), capacity);
    let provider = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let error =
        crate::sema::crates::authored_export::Stage::begin(&report, &mut checker.facts, &provider)
            .err()
            .ok_or("removed scheme was inferred empty")?;
    assert_eq!(error, "source function has no completed lifetime scheme");
    Ok(())
}
#[test]
fn source_empty_completion_is_linear_and_legacy_stays_cold() -> Result {
    let base = test_source::resolve("", true)?;
    let mut ordinary = Checker::new(&base)?;
    ordinary.run()?;
    let ordinary_extent = (
        ordinary.facts.lifetimes.functions.len(),
        ordinary.facts.lifetimes.functions.capacity(),
    );
    let provider = test_source::source_provider(&source("")?, "empty_growth")?;
    let mut base = Checker::new(&provider)?;
    base.run()?;
    let base_count = base.facts.lifetimes.functions.len();
    for count in [1, 8, 32] {
        let mut extra = String::new();
        for index in 0..count {
            writeln!(extra, "pub fn empty{index}(x:u8)->u8{{x}}")?;
        }
        let report = test_source::source_provider(&source(&extra)?, "empty_growth")?;
        let mut checker = Checker::new(&report)?;
        checker.run()?;
        assert_eq!(checker.facts.lifetimes.functions.len(), base_count + count);
        assert!(
            retained::map(&checker.facts.lifetimes.functions)
                <= retained::new_map::<DefId, FunctionRegions>(base_count + count)
        );
        let function = named(&checker.facts, &report, "empty0")?;
        let capacity = checker.facts.lifetimes.functions.capacity();
        for _ in 0..1000 {
            assert_eq!(
                checker
                    .facts
                    .function_header(function.definition)?
                    .ok_or("actual zero header")?
                    .universals,
                0
            );
        }
        assert_eq!(checker.facts.lifetimes.functions.len(), base_count + count);
        assert_eq!(checker.facts.lifetimes.functions.capacity(), capacity);
        for prefix in ["", "#![feature(allow_internal_unstable)]"] {
            let report = test_source::resolve(&format!("{prefix}{extra}"), true)?;
            assert!(!report.package.uses_source_bindings());
            let mut legacy = Checker::new(&report)?;
            legacy.run()?;
            let function = named(&legacy.facts, &report, "empty0")?;
            assert!(
                !legacy
                    .facts
                    .lifetimes
                    .functions
                    .contains_key(&function.definition)
            );
            assert!(!legacy.function_binders.contains_key(&function.node));
            assert_eq!(
                (
                    legacy.facts.lifetimes.functions.len(),
                    legacy.facts.lifetimes.functions.capacity()
                ),
                ordinary_extent
            );
        }
    }
    Ok(())
}
