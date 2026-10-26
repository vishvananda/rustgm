//! Try syntax retains exact source sites without granting missing Try execution.
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
        package::Error,
    },
    support::{diagnostics::WarningCode, edition::Edition, source::SourceFile},
    syntax::{
        ast::{Kind, Syntax},
        expression, gate_sites,
        transfer::{self, Extent},
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str, edition: Edition) -> std::result::Result<Report, Error> {
    resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "try.rs".into(),
            path: "try.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec()).map_err(Error::unlocated)?,
        },
        &mut NoFiles,
        &Config::in_edition(edition),
    )?)
}
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-try-blocks/{name}.rs"
    ))?)
}
#[test]
fn grammar_keeps_real_block_edges_statement_boundaries_and_explicit_semantic_fence() -> Result {
    let ast = expression::parse("try { try { 7 } } ").map_err(|error| error.message)?;
    let entries = ast.gate_sites.as_ref().ok_or("try gate owner")?;
    assert_eq!(entries.expressions().len(), 2);
    for (ordinal, entry) in entries.expressions().iter().enumerate() {
        entry
            .validate(&ast, u32::try_from(ordinal)?)
            .map_err(|error| error.message)?;
        assert!(entry.context_key().is_none());
    }
    let mut text = Vec::new();
    expression::write_ast("try { try { 7 } } ", &ast, &mut text)?;
    assert_eq!(String::from_utf8(text)?.matches("try-block ").count(), 2);
    for invalid in ["try 7", "try {", "try { 7 } catch {}"] {
        assert!(expression::parse(invalid).is_err(), "{invalid}");
    }
    expression::parse("{try {} let x=1;x}").map_err(|error| error.message)?;
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        let enabled = report(&fixture("enabled-value")?, edition)?;
        let error = crate::pipeline::analyze_source(&enabled)
            .err()
            .ok_or("try gained execution")?;
        assert!(
            error.message.contains("source Try control-flow semantics"),
            "{error}"
        );
        let question = report("fn f()->Option<u8>{Some(7)?;None}", edition)?;
        let error = crate::pipeline::analyze_source(&question)
            .err()
            .ok_or("question mark gained execution")?;
        assert!(
            error.message.contains("source Try control-flow semantics"),
            "{error}"
        );
    }
    Ok(())
}
#[test]
fn feature_authority_matches_real_endpoint_composition_and_caller_opponents() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for name in [
            "enabled-value",
            "enabled-unit",
            "literal-allowance",
            "forwarded-token",
            "literal-keyword-forwarded-block",
            "nested-forwarded-keyword",
        ] {
            report(&fixture(name)?, edition).map_err(|error| format!("{name}: {error}"))?;
        }
        for name in ["missing-gate", "ancestor-allowance", "forwarded-both"] {
            let error = report(&fixture(name)?, edition)
                .err()
                .ok_or_else(|| format!("{name} gained authority"))?;
            assert!(error.message.contains("try_blocks"), "{name}: {error}");
        }
    }
    Ok(())
}
#[test]
fn inactive_owners_warn_without_semantic_demand_and_changed_source_or_edges_reject() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        let disabled = report(&fixture("disabled-owner")?, edition)?;
        let analysis = crate::pipeline::analyze_source(&disabled)?;
        let warnings: Vec<_> = analysis.facts.warnings().collect();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, WarningCode::UnstableSyntaxPreExpansion);
        assert_eq!(warnings[0].diagnostic.message, "`try` blocks are unstable");
        let enabled = report(&fixture("disabled-enabled")?, edition)?;
        assert_eq!(
            crate::pipeline::analyze_source(&enabled)?
                .facts
                .warnings()
                .count(),
            0
        );
        assert!(report(&fixture("disabled-malformed")?, edition).is_err());
    }
    let mut found = report(&fixture("enabled-unit")?, Edition::Rust2024)?;
    let entry = *found.package.units[0]
        .configured
        .ast
        .expression_gate(0)
        .ok_or("actual try site")?;
    let fresh = found.package.units[0]
        .fresh_instance()
        .map_err(|error| error.message)?;
    fresh
        .configured
        .ast
        .validate_gate_sites_source(fresh.source.original_image())
        .map_err(|error| error.message)?;
    assert!(!std::sync::Arc::ptr_eq(
        found.package.units[0].source.original_image(),
        fresh.source.original_image()
    ));
    found.package.units[0].source = found.package.units[0].source.physical_copy();
    assert!(super::validate(&found).is_err());
    let mut found = report(&fixture("enabled-unit")?, Edition::Rust2024)?;
    found.package.units[0].configured.ast.nodes[entry.node.get() as usize].kind = Kind::TryBlock {
        body: entry.node,
        gate: 0,
    };
    assert!(super::validate(&found).is_err());
    Ok(())
}
#[test]
fn sparse_expression_transfer_rollback_retry_and_all_enclosing_costs_are_measured() -> Result {
    use std::fmt::Write;
    let source = expression::parse("try { try {} }").map_err(|error| error.message)?;
    let rejected = source
        .gate_sites
        .as_ref()
        .ok_or("source gate owner")?
        .expressions()[1]
        .keyword;
    let mut destination = Syntax::default();
    let empty = Extent::new(&destination);
    let mut prior = 0;
    for _ in 0..2 {
        assert!(
            transfer::append(
                &source,
                &mut destination,
                |span| if span == rejected {
                    Err(crate::syntax::token_tree::error(
                        span.start,
                        "declared transfer failure",
                    ))
                } else {
                    Ok(span)
                },
                |value| Ok(value.clone())
            )
            .is_err()
        );
        assert!(destination.nodes.is_empty());
        let rows = destination.gate_sites.as_ref().ok_or("failed debt owner")?;
        let (len, _, steps, _) = rows.expression_storage();
        assert_eq!(len, 0);
        assert!(steps > prior);
        prior = steps;
        assert!(
            empty
                .copy(&destination)
                .map_err(|error| error.message)?
                .gate_sites
                .as_ref()
                .ok_or("replayed debt")?
                .expression_storage()
                .2
                >= steps
        );
    }
    destination.root = transfer::append(&source, &mut destination, Ok, |value| Ok(value.clone()))
        .map_err(|error| error.message)?;
    assert_eq!(
        destination
            .gate_sites
            .as_ref()
            .ok_or("retained rows")?
            .expressions()
            .len(),
        2
    );
    for width in [8, 32, 128] {
        let mut text = String::from("#![feature(try_blocks)]fn f(){");
        for _ in 0..width {
            write!(text, "let _:Option<()>=try{{}};")?;
        }
        text.push('}');
        let found = report(&text, Edition::Rust2024)?;
        let ast = &found.package.units[0].configured.ast;
        let rows = ast.gate_sites.as_ref().ok_or("retained try rows")?;
        let (len, capacity, steps, bytes) = rows.expression_storage();
        assert_eq!(len, width);
        println!(
            "try width={width} rows={len} capacity={capacity} steps={steps} bytes={bytes} payload={}",
            ast.gate_sites_bytes()
        );
    }
    let plain = expression::parse("{}").map_err(|error| error.message)?;
    assert!(plain.gate_sites.is_none());
    println!(
        "try layouts Kind={} Node={} Item={} Syntax={} Unit={} Extent={} Prefix={} GateOwner={} Expression={} Declaration={}",
        size_of::<Kind>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::syntax::item::Item>(),
        size_of::<Syntax<crate::literal::storage::StoredValue>>(),
        size_of::<resolve::unit::Unit>(),
        size_of::<Extent>(),
        size_of::<gate_sites::Prefix>(),
        size_of::<gate_sites::Declarations>(),
        size_of::<gate_sites::Expression>(),
        size_of::<gate_sites::Declaration>()
    );
    println!(
        "try Original-Saved={:?}",
        resolve::unit::Unit::original_layout_bytes()
    );
    Ok(())
}
