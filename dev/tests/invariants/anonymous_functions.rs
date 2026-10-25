//! Actual anonymous signature roots share a contextual DAG without call authority.
use super::*;
use crate::{resolve::expansion::Report, source_packets, support::crates, test_source};
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CARRY: &str = "pub fn carry<T>(value:fn(u8)->u8)->fn(u8)->u8{value}\
    pub fn plain(value:fn(u8)->u8)->fn(u8)->u8{value}\
    pub fn unsafe_carry<T>(value:unsafe fn(u8)->u8)->unsafe fn(u8)->u8{value}";

fn source(extra: &str) -> Result<String> {
    let mut source =
        test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    source.push_str(extra);
    Ok(source)
}
fn prepared(extra: &str) -> Result<(Report, Declarations)> {
    let report = test_source::source_provider(&source(extra)?, "anonymous_types")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    Ok((report, facts))
}
fn stage(report: &Report, facts: &mut Declarations) -> Result<Stage> {
    Ok(Stage::begin(
        report,
        facts,
        &report.package.source_core().ok_or("provider")?.provider,
    )?)
}
fn root(
    report: &Report,
    facts: &Declarations,
    stage: &mut Stage,
    name: &str,
) -> Result<(TypeId, Context)> {
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == name
        })
        .ok_or("actual source function")?;
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(function.signature) else {
        return Err("actual function signature".into());
    };
    let ty = *facts
        .types
        .list(parameters)
        .ok_or("actual parameters")?
        .first()
        .ok_or("actual input")?;
    Ok((ty, stage.original_context(function.definition)?))
}
fn header<'a>(rows: &'a s::Columns, name: &str) -> Result<&'a s::FunctionHeader> {
    rows.functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
        })
        .ok_or_else(|| "actual metadata function".into())
}
fn anonymous(rows: &s::SourceRows) -> usize {
    rows.binders
        .iter()
        .filter(|binder| binder.kind == s::BinderKind::CallableType)
        .count()
}
fn inspected(extra: &str) -> Result<(crates::CrateIdentity, s::SourceRows)> {
    let report = test_source::source_provider(&source(extra)?, "anonymous_types")?;
    let identity = report
        .package
        .source_core()
        .ok_or("provider")?
        .provider
        .clone();
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let inspected = crate::lower::instances::inspect_source_transport(
        &report,
        &mut analysis.facts,
        &identity,
        analysis.prepared,
    )?;
    Ok((identity, inspected.rows))
}

fn item_summaries() -> Result {
    let report = test_source::source_provider(
        &source(
            "pub fn selected<T>(value:T)->T{value}pub fn item(){let _=selected::<fn(u8)->u8>;}",
        )?,
        "anonymous_arguments",
    )?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let facts = &analysis.facts;
    let program = crate::body::build_concrete(&report, facts)?;
    let mut actual_argument = false;
    for body in &program.bodies {
        for value in &body.values {
            if let Some(Kind::FunctionItem { arguments, .. }) = facts.types.kind(value.ty)
                && facts
                    .types
                    .arguments(arguments)
                    .ok_or("actual item arguments")?
                    .any(|argument| {
                        matches!(argument, crate::types::arguments::Argument::Type(ty)
                        if matches!(facts.types.kind(ty), Some(Kind::Function { .. })))
                    })
            {
                assert_eq!(
                    facts.types.contains_anonymous_callables(value.ty),
                    Some(true)
                );
                actual_argument = true;
            }
        }
    }
    assert!(actual_argument);
    for function in &facts.functions {
        let ty = facts.definitions[function.definition.0 as usize].ok_or("actual definition")?;
        assert_eq!(facts.types.contains_anonymous_callables(ty), Some(false));
    }
    Ok(())
}

#[test]
fn anonymous_function_headers_cross_codec_without_declaration_surrogates() -> Result {
    let interface = source_packets::exported(&source(CARRY)?)?;
    let rows = interface
        .source_descriptor()
        .ok_or("source descriptor")?
        .rows();
    assert!(
        !rows
            .declarations
            .iter()
            .any(|row| row.name.as_deref() == Some("function_type"))
    );
    assert_eq!(rows.functions.len(), 5);
    for name in ["carry", "plain", "unsafe_carry"] {
        let header = header(&rows, name)?;
        let inputs = header.callable.parameters.slice(&rows.roots)?;
        assert_eq!(inputs.len(), 1);
        assert_eq!(header.callable.result, inputs[0]);
        let s::Type::Function(signature) = rows.types[inputs[0].0 as usize] else {
            return Err("anonymous callable recipe".into());
        };
        let binder = &rows.binders[signature.binder.0 as usize];
        assert_eq!(binder.kind, s::BinderKind::CallableType);
        assert_eq!(binder.owner, header.declaration);
        assert!(binder.parent.is_none());
        assert_eq!(binder.count, 0);
        assert_eq!(
            signature.parameters.slice(&rows.roots)?,
            &[signature.result]
        );
        assert_eq!(signature.unsafe_, name == "unsafe_carry");
        assert!(!signature.variadic);
        assert_eq!(signature.convention, g::Convention::Rust);
    }
    let bytes = crates::encode(&interface)?;
    assert_eq!(bytes.get(12), Some(&10));
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    source_packets::client_analysis(
        decoded,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn idle(){}",
    )?;
    Ok(())
}

#[test]
fn anonymous_function_context_and_binder_opponents_fail_before_publication() -> Result {
    let (identity, rows) = inspected(CARRY)?;
    let binder = rows
        .binders
        .iter()
        .position(|row| row.kind == s::BinderKind::CallableType)
        .ok_or("actual anonymous frame")?;
    for kind in [
        s::BinderKind::PublicFunction,
        s::BinderKind::OriginalFunction,
    ] {
        let mut changed = rows.clone();
        changed.binders[binder].kind = kind;
        assert!(g::Payload::from_source(&identity, changed, 0, &mut g::Work::default()).is_err());
    }
    let mut changed = rows.clone();
    changed.binders[binder].count = 1;
    assert!(g::Payload::from_source(&identity, changed, 0, &mut g::Work::default()).is_err());
    let (report, mut facts) = prepared(CARRY)?;
    let mut staged = stage(&report, &mut facts)?;
    let (ty, context) = root(&report, &facts, &mut staged, "carry")?;
    let (other_report, mut other_facts) = prepared(CARRY)?;
    let mut other = stage(&other_report, &mut other_facts)?;
    let (other_ty, other_context) = root(&other_report, &other_facts, &mut other, "carry")?;
    assert_eq!(
        staged.ty(&other_facts, other_ty, other_context).err(),
        Some("source type memo belongs to another canonical arena")
    );
    let original = std::mem::take(&mut facts.types);
    assert_eq!(
        staged.ty(&facts, ty, context).err(),
        Some("source type memo belongs to another canonical arena")
    );
    facts.types = original;
    assert!(staged.ty(&facts, ty, context).is_ok());
    let (plain, context) = root(&report, &facts, &mut staged, "plain")?;
    let extent = (
        staged.rows.types.len(),
        staged.rows.binders.len(),
        staged.rows.roots.len(),
    );
    staged.bytes = crates::MAX_BYTES;
    assert!(staged.ty(&facts, plain, context).is_err());
    assert_eq!(
        extent,
        (
            staged.rows.types.len(),
            staged.rows.binders.len(),
            staged.rows.roots.len()
        )
    );
    let (report, mut facts) =
        prepared("pub fn regional<T>(f:for<'a>fn(&'a u8)->&'a u8)->for<'a>fn(&'a u8)->&'a u8{f}")?;
    assert_eq!(
        stage(&report, &mut facts)
            .err()
            .map(|error| error.to_string())
            .as_deref(),
        Some("source anonymous callable requires its supported closed Rust frame")
    );
    Ok(())
}

#[test]
fn anonymous_function_shared_roots_use_one_persistent_dag() -> Result {
    item_summaries()?;
    let (_, body_only) = inspected("pub fn body_only(){let slot:fn(u8)->u8;}")?;
    assert_eq!(anonymous(&body_only), 1);
    assert!(
        body_only
            .originals
            .iter()
            .flat_map(|body| &body.locals)
            .any(|local| { matches!(body_only.types[local.ty.0 as usize], s::Type::Function(_)) })
    );
    let (report, mut facts) =
        prepared("pub fn shared<T>(left:fn(u8)->u8,right:fn(u8)->u8)->fn(u8)->u8{left}")?;
    let mut staged = stage(&report, &mut facts)?;
    let (ty, context) = root(&report, &facts, &mut staged, "shared")?;
    let selected = staged.ty(&facts, ty, context)?;
    let extent = (
        staged.rows.types.len(),
        staged.rows.binders.len(),
        staged.rows.roots.len(),
        staged.graph.types.capacity(),
    );
    let before = staged.work();
    for _ in 0..1000 {
        assert_eq!(staged.ty(&facts, ty, context)?, selected);
    }
    let after = staged.work();
    assert_eq!(
        extent,
        (
            staged.rows.types.len(),
            staged.rows.binders.len(),
            staged.rows.roots.len(),
            staged.graph.types.capacity()
        )
    );
    assert_eq!(after.cache_hits - before.cache_hits, 1000);
    assert_eq!(after.roots - before.roots, 1000);
    assert_eq!(after.reserved_bytes, before.reserved_bytes);
    assert_eq!(after.contextual_types, before.contextual_types);
    staged.steps = crates::MAX_COMPONENTS;
    assert!(staged.ty(&facts, ty, context).is_err());
    assert!(staged.steps > crates::MAX_COMPONENTS);
    assert_eq!(
        extent,
        (
            staged.rows.types.len(),
            staged.rows.binders.len(),
            staged.rows.roots.len(),
            staged.graph.types.capacity()
        )
    );
    Ok(())
}

#[test]
fn anonymous_function_work_and_storage_scale_with_actual_shallow_dag() -> Result {
    let (base_report, mut base_facts) = prepared("")?;
    let was_anchored = base_facts.types.existing_arena_identity().is_some();
    let baseline = stage(&base_report, &mut base_facts)?;
    assert_eq!(
        base_facts.types.existing_arena_identity().is_some(),
        was_anchored
    );
    assert!(baseline.graph.arena.is_none());
    for depth in [1, 4, 8] {
        let mut ty = "u8".to_owned();
        for _ in 0..depth {
            ty = format!("fn({ty})->u8");
        }
        for width in [1, 8, 32] {
            let mut extra = String::new();
            for index in 0..width {
                writeln!(
                    extra,
                    "pub fn carry{index}<T>(left:{ty},right:{ty})->{ty}{{left}}"
                )?;
            }
            let (report, mut facts) = prepared(&extra)?;
            let mut staged = stage(&report, &mut facts)?;
            // Original and public frames form distinct actual contexts. Equal
            // child roots within each context share each completed DAG node.
            assert_eq!(anonymous(&staged.rows), 2 * depth * width);
            let work = staged.work();
            assert!(work.steps <= baseline.work().steps + width * (depth + 1) * 2048);
            assert!(
                work.reserved_bytes <= baseline.work().reserved_bytes + width * (depth + 1) * 32768
            );
            let (ty, context) = root(&report, &facts, &mut staged, "carry0")?;
            let selected = staged.ty(&facts, ty, context)?;
            assert_eq!(staged.ty(&facts, ty, context)?, selected);
        }
    }
    Ok(())
}
