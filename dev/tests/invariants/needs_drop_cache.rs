//! Cached bools cannot authorize a changed direct call or incomplete source registry.
use super::*;
use crate::{
    body,
    const_eval::{self, Machine, State, code::Catalog},
    expand::config::Config,
    resolve::{
        self,
        bindings::Context,
        load::{InputFile, Sources},
    },
    sema,
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
    types::{Primitive, TypeId},
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Inputs;
impl Sources for Inputs {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
struct Prepared {
    report: resolve::expansion::Report,
    facts: sema::model::Declarations,
    catalog: Catalog,
    state: State,
    target: Instance,
    other: usize,
}
fn source_report(source: &str) -> Result<resolve::expansion::Report> {
    let provider = CrateIdentity::new("minicore", "needs_drop_calls")?;
    let context = Context {
        mode: Mode::Source {
            provider: provider.clone(),
        },
        current: Some(provider),
        crate_name: Some("minicore".into()),
        ..Default::default()
    };
    let input = InputFile {
        identity: "drop-calls.rs".into(),
        path: "drop-calls.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    };
    let expansion =
        resolve::expansion::analyze_with_context(input, &mut Inputs, &Config::default(), &context)?;
    Ok(resolve::analyze_expanded(expansion)?)
}
fn prepared(corrupt: bool) -> Result<Prepared> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../pa31/tests/spec/needs-drop/minimal-drop.rs");
    let minimal = std::fs::read_to_string(path)?;
    let prefix = minimal
        .split("\npub const fn needs_drop")
        .next()
        .ok_or("prefix")?;
    let source = format!(
        "{prefix}struct D;impl Drop for D{{fn drop(&mut self){{}}}}const fn first()->bool{{intrinsics::needs_drop::<D>()}}const fn other()->bool{{intrinsics::needs_drop::<u8>()}}"
    );
    let report = source_report(&source)?;
    let facts = sema::declarations::analyze(&report)?;
    let mut facts = sema::bodies::analyze(&report, facts)?;
    let mut program = body::build(&report, &facts)?;
    let mut catalog = Catalog::default();
    let mut first = None;
    let mut other = None;
    for body in &mut program.bodies {
        let body::ir::Owner::Function { definition, .. } = body.owner else {
            continue;
        };
        let name = report
            .package
            .names
            .get(report.resolution.definitions[definition.0 as usize].name);
        if !matches!(name, "first" | "other") {
            continue;
        }
        if corrupt && name == "other" {
            let value = body
                .values
                .iter_mut()
                .find(|v| matches!(v.kind, body::ir::Rvalue::Call { .. }))
                .ok_or("call")?;
            value.ty = TypeId::primitive(Primitive::U8);
        }
        let index = catalog.insert(body)?;
        if name == "first" {
            first = Some(index);
        } else {
            other = Some(index);
        }
    }
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::catalog(evaluator, &catalog, const_eval::Limits::default());
    machine
        .begin(first.ok_or("first")?, &[])
        .map_err(|e| e.message)?;
    let sema::values::execution::Progress::Prepare(
        sema::values::execution::Requirement::FunctionBody(target),
    ) = machine.advance().map_err(|e| e.message)?
    else {
        return Err("missing intrinsic preparation".into());
    };
    let (state, _) = machine.suspend();
    Ok(Prepared {
        report,
        facts,
        catalog,
        state,
        target,
        other: other.ok_or("other")?,
    })
}
#[test]
fn bool_cache_shares_query_but_keeps_full_instance_and_fresh_call() -> Result {
    let mut p = prepared(false)?;
    assert!(p.catalog.source_results.is_none());
    let call = p
        .state
        .source_needs_drop_call(&p.facts, &p.catalog, p.target)?
        .ok_or("call")?;
    assert!(p.catalog.evaluate_needs_drop(&p.facts, &call)?);
    let work = p
        .catalog
        .source_results
        .as_deref()
        .and_then(|r| r.drops.as_deref())
        .ok_or("query")?
        .query
        .steps;
    let bytes = p.catalog.bytes;
    for _ in 0..128 {
        let call = p
            .state
            .source_needs_drop_call(&p.facts, &p.catalog, p.target)?
            .ok_or("call")?;
        assert!(p.catalog.evaluate_needs_drop(&p.facts, &call)?);
    }
    assert_eq!(p.catalog.bytes, bytes);
    assert_eq!(
        p.catalog
            .source_results
            .as_deref()
            .and_then(|r| r.drops.as_deref())
            .ok_or("query")?
            .query
            .steps,
        work
    );
    let evaluator = sema::values::Evaluator::new(&p.report, &mut p.facts);
    let mut machine = Machine::catalog(evaluator, &p.catalog, const_eval::Limits::default());
    machine.begin(p.other, &[]).map_err(|e| e.message)?;
    let sema::values::execution::Progress::Prepare(
        sema::values::execution::Requirement::FunctionBody(other),
    ) = machine.advance().map_err(|e| e.message)?
    else {
        return Err("distinct instance reused cached answer".into());
    };
    let (state, _) = machine.suspend();
    let call = state
        .source_needs_drop_call(&p.facts, &p.catalog, other)?
        .ok_or("other call")?;
    assert!(!p.catalog.evaluate_needs_drop(&p.facts, &call)?);
    assert_eq!(p.catalog.needs_drop_result(p.target), Some(true));
    assert_eq!(p.catalog.needs_drop_result(other), Some(false));
    println!(
        "SourceResults={} DropResults={} CodeCatalog={} SourceCatalog={} query_steps={work} cache_bytes={bytes}",
        size_of::<super::super::SourceResults>(),
        size_of::<DropResults>(),
        size_of::<Catalog>(),
        size_of::<sema::compiler_items::Catalog>()
    );
    Ok(())
}
#[test]
fn warm_bool_result_rejects_wrong_statement_destination_and_reserved_roles() -> Result {
    let mut p = prepared(true)?;
    let call = p
        .state
        .source_needs_drop_call(&p.facts, &p.catalog, p.target)?
        .ok_or("call")?;
    p.catalog.evaluate_needs_drop(&p.facts, &call)?;
    let frame = p.state.frames.last().ok_or("frame")?;
    assert!(
        const_eval::reached_drop::validate(
            &p.facts,
            p.catalog.code(frame.body).ok_or("code")?,
            frame,
            frame.next + 1,
            p.target
        )
        .is_err()
    );
    let evaluator = sema::values::Evaluator::new(&p.report, &mut p.facts);
    let mut machine = Machine::catalog(evaluator, &p.catalog, const_eval::Limits::default());
    machine.begin(p.other, &[]).map_err(|e| e.message)?;
    assert!(machine.advance().is_err());
    drop(machine);
    p.facts.source_bindings = sema::compiler_items::Catalog::reserve(&p.report)?;
    assert!(
        p.state
            .source_needs_drop_call(&p.facts, &p.catalog, p.target)
            .is_err()
    );
    assert!(p.catalog.evaluate_needs_drop(&p.facts, &call).is_err());
    assert_eq!(p.catalog.needs_drop_result(p.target), Some(true));
    Ok(())
}
#[test]
fn tiny_result_budget_refuses_before_cold_query_allocation() -> Result {
    let mut p = prepared(false)?;
    let call = p
        .state
        .source_needs_drop_call(&p.facts, &p.catalog, p.target)?
        .ok_or("call")?;
    p.catalog.bytes = LIMIT;
    assert!(p.catalog.evaluate_needs_drop(&p.facts, &call).is_err());
    assert!(p.catalog.source_results.is_none());
    assert_eq!(p.catalog.bytes, LIMIT);
    Ok(())
}
