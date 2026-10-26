//! Actual record measurements and call contexts precede source-client transport.
use super::*;
use crate::{
    resolve::{body_origin::DeclarationOrigin, expansion::Report, model::DefId},
    sema::{
        associated::{Owner, OwnerKind},
        model::{Declarations, NodeFacts, Operation},
    },
    support::constness::Constness,
};
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const _: [(); 16] = [(); size_of::<Operation>()];
const _: [(); 80] = [(); size_of::<NodeFacts>()];
const _: [(); 24] = [(); size_of::<crate::body::ir::Value>()];

// These owner alternatives are layout-only; neither can construct an origin.
#[allow(dead_code)]
struct OptionalSourceOwner {
    source: Option<NodeKey>,
    self_definition: DefId,
    kind: OwnerKind,
}
#[allow(dead_code)]
struct DenseOriginOwner {
    source: DeclarationOrigin,
    self_definition: DefId,
    kind: OwnerKind,
}

#[test]
fn actual_scoped_call_and_optional_owner_keep_dense_layouts() {
    println!(
        "CallUse={} OptionalCallUse={} Operation={} NodeFacts={} NodeKey={} OptionalNodeKey={} Owner={} OptionalSourceOwner={} DenseOriginOwner={} OwnerKind={} Declarations={} Catalog={} Contexts={} Descriptor={}",
        size_of::<SourceCallUse>(),
        size_of::<Option<SourceCallUse>>(),
        size_of::<Operation>(),
        size_of::<NodeFacts>(),
        size_of::<NodeKey>(),
        size_of::<Option<NodeKey>>(),
        size_of::<Owner>(),
        size_of::<OptionalSourceOwner>(),
        size_of::<DenseOriginOwner>(),
        size_of::<OwnerKind>(),
        size_of::<Declarations>(),
        size_of::<crate::sema::compiler_items::Catalog>(),
        size_of::<CallContexts>(),
        size_of::<Descriptor>()
    );
    assert_eq!(size_of::<NodeFacts>(), 80);
    assert_eq!(size_of::<Owner>(), 32);
    assert_eq!(size_of::<OwnerKind>(), 20);
    assert_eq!(size_of::<Owner>(), size_of::<OptionalSourceOwner>());
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
}

fn analyze(source: &str) -> Result<(Report, Declarations)> {
    let report = crate::test_source::source_provider(source, "source_effect_facts")?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    Ok((report, facts))
}
fn body(report: &Report, facts: &Declarations, name: &str) -> Result<NodeKey> {
    facts
        .functions
        .iter()
        .find_map(|function| {
            let definition = &report.resolution.definitions[function.definition.0 as usize];
            (report.package.names.get(definition.name) == name)
                .then_some(function.body)
                .flatten()
        })
        .ok_or_else(|| format!("missing actual body {name}").into())
}
fn effects(facts: &Declarations, owner: NodeKey) -> Result<Vec<Constness>> {
    facts
        .nodes
        .iter()
        .flatten()
        .filter_map(|node| {
            if let Some(Operation::Call {
                source_use: Some(source),
                ..
            }) = node.operation
            {
                (source.owner(facts) == Ok(owner)).then_some(source.effect_for(facts, owner))
            } else {
                None
            }
        })
        .map(|result| result.map_err(Into::into))
        .collect()
}

#[test]
fn source_calls_cover_plain_ufcs_dot_and_fn_once_operator_nodes() -> Result {
    for source in [
        crate::provider_cases::CALLABLE_FUNCTION_ITEM.to_owned(),
        crate::provider_cases::SOURCE_CLONE_UFCS.to_owned(),
        crate::provider_cases::SOURCE_CLONE_UFCS
            .replace("<u8 as Clone>::clone(&11u8)", "11u8.clone()"),
    ] {
        let (report, facts) = analyze(&source)?;
        let answer = body(&report, &facts, "answer")?;
        assert_eq!(effects(&facts, answer)?, vec![Constness::Ordinary]);
        for node in facts.nodes.iter().flatten() {
            if let Some(Operation::Call { source_use, .. }) = node.operation {
                assert!(source_use.is_some());
            }
        }
        if source.contains("fn apply") {
            let apply = body(&report, &facts, "apply")?;
            assert_eq!(effects(&facts, apply)?, vec![Constness::Ordinary]);
            assert!(
                facts
                    .invocations
                    .values()
                    .any(|invocation| invocation.kind.is_some())
            );
        }
    }
    Ok(())
}

#[test]
fn semantic_body_contexts_preserve_const_definition_initializer_and_nested_identity() -> Result {
    let source = format!(
        "{}\nconst fn seven()->u8{{7}}const fn defining()->u8{{seven()}}const CLOSED:u8=seven();fn runtime()->u8{{seven()}}fn inline_child()->u8{{const{{seven()}}}}fn closure_parent()->u8{{let _f=||seven();7}}",
        crate::provider_cases::CALLABLE_FUNCTION_ITEM
    );
    let (report, facts) = analyze(&source)?;
    assert_eq!(
        effects(&facts, body(&report, &facts, "defining")?)?,
        vec![Constness::Conditional]
    );
    assert_eq!(
        effects(&facts, body(&report, &facts, "runtime")?)?,
        vec![Constness::Ordinary]
    );
    let initializer = facts
        .initializers
        .iter()
        .find(|initializer| {
            report
                .package
                .names
                .get(report.resolution.definitions[initializer.definition.0 as usize].name)
                == "CLOSED"
        })
        .ok_or("initializer")?;
    assert_eq!(
        effects(&facts, initializer.semantic_source())?,
        vec![Constness::Const]
    );
    let closure = facts.closures.all().first().ok_or("closure")?;
    assert_eq!(
        effects(&facts, closure.semantic_source())?,
        vec![Constness::Ordinary]
    );
    assert_ne!(
        closure.semantic_source(),
        body(&report, &facts, "closure_parent")?
    );
    let inline = facts
        .constant_bodies()
        .into_iter()
        .find_map(|(source, _)| {
            effects(&facts, source)
                .ok()
                .filter(|effects| !effects.is_empty())
                .map(|effects| (source, effects))
        })
        .ok_or("inline child")?;
    assert_eq!(inline.1, vec![Constness::Conditional]);
    assert_ne!(inline.0, body(&report, &facts, "inline_child")?);
    let original = facts
        .nodes
        .iter()
        .flatten()
        .find_map(|node| match node.operation {
            Some(Operation::Call {
                source_use: Some(source),
                ..
            }) if source.owner(&facts) == Ok(closure.semantic_source()) => Some(source),
            _ => None,
        })
        .ok_or("closure call")?;
    assert!(
        original
            .effect_for(&facts, body(&report, &facts, "closure_parent")?)
            .is_err()
    );
    Ok(())
}

#[test]
fn ordinary_calls_retain_absent_state_and_source_data_does_not_change_view_bytes() -> Result {
    let report = crate::test_source::resolve("fn f()->u8{7}fn caller()->u8{f()}", true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let facts = crate::sema::bodies::analyze(&report, facts)?;
    assert!(facts.source_bindings.is_none());
    let mut calls = 0;
    for node in facts.nodes.iter().flatten() {
        if let Some(Operation::Call { source_use, .. }) = node.operation {
            assert!(source_use.is_none());
            calls += 1;
        }
    }
    assert_eq!(calls, 1);
    let (report, mut facts) = analyze(crate::provider_cases::CALLABLE_FUNCTION_ITEM)?;
    let mut before = Vec::new();
    crate::sema::type_view::write(&report, &facts, &mut before)?;
    for node in facts.nodes.iter_mut().flatten() {
        if let Some(Operation::Call { source_use, .. }) = &mut node.operation {
            *source_use = None;
        }
    }
    let mut after = Vec::new();
    crate::sema::type_view::write(&report, &facts, &mut after)?;
    assert_eq!(before, after);
    Ok(())
}

fn row_count(facts: &Declarations) -> usize {
    facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.call_contexts.as_deref())
        .map_or(0, |contexts| contexts.rows.len())
}
fn replay<'a>(
    report: &'a Report,
    facts: &'a mut Declarations,
    owner: NodeKey,
) -> Result<(Checker<'a>, Solved)> {
    let mut calls = Vec::new();
    let mut types = Vec::new();
    for (index, node) in facts.nodes[owner.file.0 as usize].iter().enumerate() {
        if let Some(Operation::Call {
            signature,
            source_use: Some(source),
        }) = node.operation
            && source.owner(facts) == Ok(owner)
        {
            calls.push((
                crate::syntax::ast::NodeId::new(u32::try_from(index)?),
                crate::types::infer::TermId(u32::try_from(types.len())?),
            ));
            types.push(signature);
        }
    }
    let mut checker = Checker::new(report, facts);
    checker.state.body = owner;
    checker.state.file = owner.file;
    checker.state.calls = calls;
    checker
        .state
        .terms
        .prepare(owner.file, checker.facts.nodes[owner.file.0 as usize].len());
    for &(node, _) in &checker.state.calls {
        if let crate::syntax::ast::Kind::Call { callee, .. } = checker.kind(node) {
            let key = checker.key(callee);
            let ty = checker
                .facts
                .node(key)
                .and_then(|facts| facts.ty)
                .ok_or("replayed call has no completed callee type")?;
            let term = crate::types::infer::TermId(u32::try_from(types.len())?);
            types.push(ty);
            checker.state.terms.insert(key, term);
        }
    }
    Ok((
        checker,
        Solved {
            types,
            defining_opaques: crate::support::hash::HashMap::default(),
            region_equalities: Vec::new(),
            steps: 0,
        },
    ))
}
fn use_for(facts: &Declarations, owner: NodeKey) -> Result<SourceCallUse> {
    facts
        .nodes
        .iter()
        .flatten()
        .find_map(|node| match node.operation {
            Some(Operation::Call {
                source_use: Some(source),
                ..
            }) if source.owner(facts) == Ok(owner) => Some(source),
            _ => None,
        })
        .ok_or_else(|| "missing source call use".into())
}
#[test]
fn current_rows_reject_wrong_owners_and_cannot_carry_a_foreign_effect() -> Result {
    let (report, facts) = analyze(crate::provider_cases::CALLABLE_FUNCTION_ITEM)?;
    let owner = body(&report, &facts, "answer")?;
    let source = facts
        .nodes
        .iter()
        .flatten()
        .find_map(|node| match node.operation {
            Some(Operation::Call {
                source_use: Some(source),
                ..
            }) if source.owner(&facts) == Ok(owner) => Some(source),
            _ => None,
        })
        .ok_or("answer call")?;
    assert!(
        source
            .effect_for(
                &facts,
                NodeKey {
                    file: crate::resolve::unit::FileId(owner.file.0 + 1),
                    ..owner
                }
            )
            .is_err()
    );
    assert!(
        SourceCallUse(NonZeroU32::MAX)
            .effect_for(&facts, owner)
            .is_err()
    );
    let ordinary_report = crate::test_source::resolve("fn answer()->u8{7}", true)?;
    let ordinary = crate::sema::declarations::analyze(&ordinary_report)?;
    let ordinary = crate::sema::bodies::analyze(&ordinary_report, ordinary)?;
    assert!(source.effect_for(&ordinary, owner).is_err());
    drop(ordinary_report);
    // Different actual bodies retain different effects at coincident local rows.
    let prefix = crate::provider_cases::ABSENT_ROLE_LITERAL.replace("fn one()", "const fn one()");
    let (runtime_report, runtime) = analyze(&format!("{prefix}fn answer()->u8{{one()}}"))?;
    let (defining_report, defining) = analyze(&format!("{prefix}const fn answer()->u8{{one()}}"))?;
    let runtime_owner = body(&runtime_report, &runtime, "answer")?;
    let defining_owner = body(&defining_report, &defining, "answer")?;
    assert_eq!(runtime_owner, defining_owner);
    let foreign = use_for(&defining, defining_owner)?;
    assert_eq!(
        foreign.effect_for(&defining, defining_owner)?,
        Constness::Conditional
    );
    assert_eq!(foreign, use_for(&runtime, runtime_owner)?);
    assert_eq!(
        use_for(&runtime, runtime_owner)?.effect_for(&defining, defining_owner)?,
        Constness::Conditional
    );
    assert_eq!(
        foreign.effect_for(&runtime, runtime_owner)?,
        Constness::Ordinary
    );
    Ok(())
}
#[test]
fn repeated_publication_is_shared_and_context_failures_are_transactional() -> Result {
    let (report, mut facts) = analyze(crate::provider_cases::CALLABLE_FUNCTION_ITEM)?;
    let owner = body(&report, &facts, "answer")?;
    let before = row_count(&facts);
    let (mut checker, solved) = replay(&report, &mut facts, owner)?;
    checker.publish_calls(&solved)?;
    assert_eq!(row_count(checker.facts), before);
    assert_eq!(checker.state.steps, 3 * checker.state.calls.len());
    checker.state.in_function = false;
    assert!(checker.publish_calls(&solved).is_err());
    assert_eq!(row_count(checker.facts), before);
    checker.state.in_function = true;
    let invalid = crate::syntax::ast::NodeId::new(u32::try_from(
        checker.facts.nodes[owner.file.0 as usize].len() + 1,
    )?);
    checker
        .state
        .calls
        .push((invalid, crate::types::infer::TermId(0)));
    assert!(checker.publish_calls(&solved).is_err());
    assert_eq!(row_count(checker.facts), before);
    checker.state.calls.pop();
    checker.state.steps = 999_999;
    assert!(checker.publish_calls(&solved).is_err());
    assert!(checker.state.steps > 1_000_000);
    assert_eq!(row_count(checker.facts), before);
    let mut contexts = CallContexts::default();
    let mut steps = 0;
    assert!(contexts.reserve(&mut steps, 1).is_err());
    assert_eq!(contexts.rows.capacity(), 0);
    assert_eq!(contexts.rows.len(), 0);
    Ok(())
}
#[test]
fn context_storage_tracks_bodies_instead_of_calls_and_charges_real_ports() -> Result {
    let (empty_report, empty) = analyze(crate::provider_cases::ABSENT_ROLE_LITERAL)?;
    assert_eq!(row_count(&empty), 0);
    assert!(
        empty
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .call_contexts
            .is_none()
    );
    assert!(
        SourceCallUse(NonZeroU32::MIN)
            .effect_for(&empty, body(&empty_report, &empty, "one")?)
            .is_err()
    );
    for width in [8usize, 32, 128] {
        let mut source = crate::provider_cases::ABSENT_ROLE_LITERAL.to_owned();
        source.push_str("fn answer()->u8{");
        source.push_str(&"one();".repeat(width));
        source.push_str("7}");
        let (report, mut facts) = analyze(&source)?;
        assert_eq!(row_count(&facts), 1);
        let owner = body(&report, &facts, "answer")?;
        let (mut checker, solved) = replay(&report, &mut facts, owner)?;
        assert_eq!(checker.state.calls.len(), width);
        checker.publish_calls(&solved)?;
        assert_eq!(checker.state.steps, 3 * width);
        assert_eq!(row_count(checker.facts), 1);
        let mut source = crate::provider_cases::ABSENT_ROLE_LITERAL.to_owned();
        for index in 0..width {
            write!(source, "fn caller{index}()->u8{{one()}}")?;
        }
        let (_, facts) = analyze(&source)?;
        assert_eq!(row_count(&facts), width);
        let contexts = facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .call_contexts
            .as_deref()
            .ok_or("contexts")?;
        let bytes = CallContexts::bytes(contexts.rows.capacity())?;
        assert!(bytes <= MAX_RETAINED);
        println!(
            "contexts width={width} rows={} capacity={} retained={bytes} repeated_ports={}",
            contexts.rows.len(),
            contexts.rows.capacity(),
            3 * width
        );
    }
    Ok(())
}
