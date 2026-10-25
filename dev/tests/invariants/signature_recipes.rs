//! Actual anonymous signature roots share a contextual DAG without call authority.
use super::*;
use crate::{resolve::expansion::Report, source_packets, support::crates, test_source};
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CARRY: &str = "pub fn carry<T>(value:fn(u8)->u8)->fn(u8)->u8{value}\
    pub fn plain(value:fn(u8)->u8)->fn(u8)->u8{value}\
    pub fn unsafe_carry<T>(value:unsafe fn(u8)->u8)->unsafe fn(u8)->u8{value}\
    pub fn generic<T>(value:fn(T)->T)->fn(T)->T{value}pub fn unit(){}";

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

fn raw_header<'a>(rows: &'a s::SourceRows, name: &str) -> Result<&'a s::FunctionHeader> {
    rows.functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
        })
        .ok_or_else(|| "actual raw function header".into())
}
fn planner_opponents(identity: &crates::CrateIdentity, rows: &s::SourceRows) -> Result {
    let header = raw_header(rows, "plain")?;
    let root = *header
        .callable
        .parameters
        .slice(&rows.roots)?
        .first()
        .ok_or("actual pointer input")?;
    let other = raw_header(rows, "carry")?.declaration;
    let unit = raw_header(rows, "unit")?.callable.result;
    for case in 0..6 {
        let mut changed = rows.clone();
        let s::Type::Function(signature) = &mut changed.types[root.0 as usize] else {
            return Err("actual pointer recipe".into());
        };
        match case {
            0 => signature.unsafe_ = !signature.unsafe_,
            1 => signature.parameters.end = signature.parameters.start,
            2 => signature.result = unit,
            3 => signature.convention = g::Convention::C,
            4 => signature.variadic = true,
            _ => changed.binders[signature.binder.0 as usize].owner = other,
        }
        assert!(
            g::Payload::from_source(identity, changed, 0, &mut g::Work::default()).is_err(),
            "function shape/domain case {case}"
        );
    }
    let generic = raw_header(rows, "generic")?;
    let s::FunctionBody::Original(body) = generic.body else {
        return Err("actual generic body".into());
    };
    let mut changed = rows.clone();
    let obligations = &mut changed.body_inputs[body.0 as usize].obligations;
    let before = obligations.len();
    obligations.retain(|input| {
        !matches!(input.kind, s::InputKind::WellFormed(root)
        if matches!(rows.types[root.0 as usize], s::Type::Function(_)))
    });
    assert!(
        obligations.len() < before,
        "opaque Function must retain actual conditional formation"
    );
    assert!(g::Payload::from_source(identity, changed, 0, &mut g::Work::default()).is_err());
    Ok(())
}

fn regional_substitution() -> Result {
    let (_, mut rows) = inspected(
        "pub fn typed<T>(value:fn(T)->T)->fn(T)->T{value}\
         pub fn borrower<'a,U>(borrowed:&'a u8,value:fn(U)->U)->fn(U)->U{typed::<U>(value)}",
    )?;
    let caller = raw_header(&rows, "borrower")?;
    let borrowed = *caller
        .original
        .parameters
        .slice(&rows.roots)?
        .first()
        .ok_or("actual borrowed formal")?;
    let s::FunctionBody::Original(body) = caller.body else {
        return Err("actual borrower body".into());
    };
    let target = rows.originals[body.0 as usize]
        .calls
        .first()
        .ok_or("actual generic Call")?
        .target;
    let arguments = rows.function_applications[target.0 as usize].arguments;
    assert_eq!(arguments.end - arguments.start, 1);
    // Replace one real application's type argument with this actual caller's
    // declared reference. Domain ownership still holds; the rewritten Function
    // must reject the newly introduced caller region before it can publish.
    rows.arguments[arguments.start as usize] = s::Argument::Type(borrowed);
    let mut work = g::Work::default();
    let headers = s::HeaderIndex::new(&rows, 0, &mut work)?;
    let domains = headers.domains(&mut work)?;
    let error = domains
        .original_inputs(&mut work)
        .err()
        .ok_or("regional Function substitution was accepted")?;
    assert_eq!(
        error,
        "source Function substitution captured regional or function-item inputs"
    );
    Ok(())
}
fn planner_metrics(rows: &s::SourceRows) -> Result<(s::InputWork, usize)> {
    let mut work = g::Work::default();
    let headers = s::HeaderIndex::new(rows, 0, &mut work)?;
    let domains = headers.domains(&mut work)?;
    let plan = domains.original_inputs(&mut work)?;
    assert!(plan.retained_bytes() <= plan.peak_bytes());
    assert!(plan.peak_bytes() <= crates::MAX_BYTES);
    Ok((plan.work(), plan.peak_bytes()))
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

fn call_formations() -> Result {
    let source = source(
        "pub fn nested(value:fn(u8)->u8)->fn(u8)->u8{value}\
         pub fn twice(value:fn(u8)->u8)->fn(u8)->u8{let next=nested(value);nested(next)}\
         pub fn other(value:fn(u8)->u8)->fn(u8)->u8{nested(value)}\
         pub fn scalar(value:u8)->u8{value}\
         pub fn simple(value:u8)->u8{let next=scalar(value);scalar(next)}\
         pub fn simple_other(value:u8)->u8{scalar(value)}",
    )?;
    let report = test_source::source_provider(&source, "actual_call_formations")?;
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let program = crate::body::build_concrete(&report, &analysis.facts)?;
    let mut staged = stage(&report, &mut analysis.facts)?;
    let mut nested = Vec::new();
    let mut shared = Vec::new();
    let mut contexts = Vec::new();
    for body in &program.bodies {
        let crate::body::ir::Owner::Function { definition, .. } = body.owner else {
            continue;
        };
        let context = staged.original_context(definition)?;
        for (index, value) in body.values.iter().enumerate() {
            let crate::body::ir::Rvalue::Call { signature, .. } = value.kind else {
                continue;
            };
            let value = u32::try_from(index)?;
            let parts = signatures::Parts::actual(&analysis.facts, signature)?;
            let is_nested = staged
                .anonymous_signature_field(&analysis.facts, parts)?
                .is_some();
            let recipe = staged.ty(&analysis.facts, signature, context)?;
            let actual = staged.call_signature(&analysis.facts, signature, context, value)?;
            assert_ne!(actual, recipe);
            let s::Type::Function(selected) = staged.rows.types[actual.0 as usize] else {
                return Err("actual call signature recipe".into());
            };
            let frame = &staged.rows.binders[selected.binder.0 as usize];
            if is_nested {
                assert_eq!(frame.owner, context.owner);
                assert_eq!(
                    frame.kind,
                    s::BinderKind::CallSignature {
                        body: context.body.ok_or("actual body")?,
                        value
                    }
                );
                nested.push(actual);
                contexts.push(context);
            } else {
                assert_eq!(frame.kind, s::BinderKind::CallableType);
                assert!(frame.parent.is_none());
                shared.push(actual);
            }
            let before = staged.work();
            for _ in 0..8 {
                assert_eq!(
                    staged.call_signature(&analysis.facts, signature, context, value)?,
                    actual
                );
            }
            assert_eq!(staged.work().steps - before.steps, 16);
            assert_eq!(staged.work().reserved_bytes, before.reserved_bytes);
        }
    }
    assert_eq!(nested.len(), 3);
    assert!(nested[0] != nested[1] && nested[1] != nested[2] && nested[0] != nested[2]);
    assert!(contexts.windows(2).any(|pair| pair[0] == pair[1]));
    assert!(contexts.windows(2).any(|pair| pair[0] != pair[1]));
    assert_eq!(shared.len(), 3);
    assert!(shared.iter().all(|&actual| actual == shared[0]));
    Ok(())
}

fn key_layout(ty: TypeId, context: Context) {
    struct BeforeKey {
        ty: TypeId,
        context: Option<Context>,
    }
    let before = BeforeKey {
        ty,
        context: Some(context),
    };
    assert!(size_of::<Key>() >= size_of::<BeforeKey>());
    assert!(size_of::<Key>() <= size_of::<BeforeKey>() + size_of::<Formation>());
    eprintln!(
        "authored key {:?}/{:?} bytes before={} actual={} formation={} graph={}",
        before.ty,
        before.context,
        size_of::<BeforeKey>(),
        size_of::<Key>(),
        size_of::<Formation>(),
        size_of::<Graph>()
    );
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
    assert_eq!(rows.functions.len(), 7);
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
    call_formations()?;
    let (identity, rows) = inspected(CARRY)?;
    planner_opponents(&identity, &rows)?;
    regional_substitution()?;
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
    key_layout(ty, context);
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
    let (_, base_rows) = inspected("")?;
    let (base_plan, base_peak) = planner_metrics(&base_rows)?;
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
            let (_, rows) = inspected(&extra)?;
            let (plan, peak) = planner_metrics(&rows)?;
            assert!(plan.expression_nodes <= base_plan.expression_nodes + 8 * (width + depth + 1));
            assert!(plan.rewritten_uses <= base_plan.rewritten_uses + 64 * width * (depth + 1));
            assert!(plan.steps <= base_plan.steps + 4096 * width * (depth + 1));
            assert!(peak <= base_peak + 65536 * width * (depth + 1));
            eprintln!("Function planner depth={depth} width={width} {plan:?} peak={peak}");
        }
    }
    Ok(())
}
