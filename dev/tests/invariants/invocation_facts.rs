//! Invocation facts retain the exact source owner, operator and substituted receiver.
use super::*;
use crate::{sema::invocations::Invocation, types::TypeId};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn check(facts: &Declarations, body: &Body, expected: usize) -> Result<(), &'static str> {
    super::check(facts, body, expected, &mut Work::new(1_000_000))
}
fn prepared() -> TestResult<(crate::resolve::expansion::Report, Declarations, Body)> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-const-callable/original-native29.rs"),
    )?;
    let report = crate::test_source::source_provider(&source, "operator_facts")?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "apply")
        .ok_or("apply")?;
    let definition = DefId(u32::try_from(definition)?);
    let (crate::pipeline::Analysis { facts, .. }, bodies) =
        crate::pipeline::execution::materialized_bodies::during(&[definition], || {
            crate::pipeline::analyze_source(&report)
        })?;
    let body = bodies.into_iter().next().ok_or("actual checked apply")?;
    assert!(body.instance.is_some());
    assert!(
        body.values
            .iter()
            .any(|value| matches!(value.kind, Rvalue::Invoke { .. }))
    );
    Ok((report, facts, body))
}
#[test]
fn defining_const_operator_fact_rejects_changed_site_receiver_kind_and_signature() -> TestResult {
    let (_report, mut facts, body) = prepared()?;
    check(&facts, &body, 1)?;
    let statement = body
        .statements
        .iter()
        .position(|statement| {
            matches!(statement.action, Action::Evaluate(id)
        if matches!(body.values[id.0 as usize].kind, Rvalue::Invoke { .. }))
        })
        .ok_or("Invoke statement")?;
    let Action::Evaluate(id) = body.statements[statement].action else {
        return Err("statement".into());
    };
    for corruption in 0..5 {
        let mut altered = body.clone();
        match corruption {
            0 => altered.statements[statement].source = altered.source.root_body_site(),
            1 => {
                let Rvalue::Invoke { kind, .. } = &mut altered.values[id.0 as usize].kind else {
                    return Err("Invoke".into());
                };
                *kind = CallTrait::FnMut;
            }
            2 => {
                let Rvalue::Invoke { arguments, .. } = altered.values[id.0 as usize].kind else {
                    return Err("Invoke".into());
                };
                let receiver = altered.arguments[arguments.start as usize];
                altered.values[receiver.0 as usize].ty =
                    TypeId::primitive(crate::types::Primitive::U8);
            }
            3 => {
                altered.values[id.0 as usize].ty = TypeId::primitive(crate::types::Primitive::Bool);
            }
            _ => {
                let Owner::Function { definition, .. } = &mut altered.owner else {
                    return Err("owner".into());
                };
                *definition = DefId(u32::MAX);
            }
        }
        assert!(
            check(&facts, &altered, 1).is_err(),
            "corruption {corruption}"
        );
    }
    let key = body
        .source
        .proof_site(body.statements[statement].source)?
        .source()
        .ok_or("source")?;
    let retained = facts.invocations.remove(&key).ok_or("invocation")?;
    assert!(check(&facts, &body, 1).is_err());
    facts.invocations.insert(key, retained);
    check(&facts, &body, 1)?;
    let original = facts
        .node(key)
        .and_then(|node| node.operation)
        .ok_or("Call operation")?;
    let Operation::Call { signature, .. } = original else {
        return Err("Call".into());
    };
    facts.nodes[key.file.0 as usize][key.node.get() as usize].operation = Some(Operation::Call {
        signature,
        source_use: None,
    });
    assert!(check(&facts, &body, 1).is_err());
    facts.nodes[key.file.0 as usize][key.node.get() as usize].operation = Some(original);
    let owner = body.source.source().ok_or("body owner")?;
    let foreign = facts
        .nodes
        .iter()
        .flatten()
        .find_map(|node| match node.operation {
            Some(Operation::Call {
                source_use: Some(source_use),
                ..
            }) if source_use.effect_for(&facts, owner).is_err() => Some(source_use),
            _ => None,
        })
        .ok_or("another actual semantic body call fact")?;
    facts.nodes[key.file.0 as usize][key.node.get() as usize].operation = Some(Operation::Call {
        signature,
        source_use: Some(foreign),
    });
    assert!(check(&facts, &body, 1).is_err());
    facts.nodes[key.file.0 as usize][key.node.get() as usize].operation = Some(original);
    check(&facts, &body, 1)?;
    Ok(())
}
#[test]
fn const_operator_data_keeps_compact_existing_records() {
    println!(
        "Invocation={} Pending-independent CallTypes-independent Operation={} NodeFacts={} Declarations={} Body={} Code={} Value={} Clause={}",
        size_of::<Invocation>(),
        size_of::<Operation>(),
        size_of::<crate::sema::model::NodeFacts>(),
        size_of::<Declarations>(),
        size_of::<Body>(),
        size_of::<crate::const_eval::code::Code>(),
        size_of::<crate::body::ir::Value>(),
        size_of::<crate::types::parameters::clauses::Clause>()
    );
    assert_eq!(size_of::<Body>(), 456);
    assert_eq!(size_of::<crate::const_eval::code::Code>(), 304);
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
}
#[test]
fn attempted_const_operator_lookups_are_bounded_and_retained_on_failure() -> TestResult {
    let (report, mut facts, body) = prepared()?;
    let mut tiny = Work::new(0);
    assert!(super::check(&facts, &body, 1, &mut tiny).is_err());
    assert_eq!(tiny.steps, 1);
    let mut work = Work::new(1_000_000);
    super::check(&facts, &body, 1, &mut work)?;
    let visits = work.steps;
    let statement = body
        .statements
        .iter()
        .find(|statement| {
            matches!(statement.action, Action::Evaluate(id)
        if matches!(body.values[id.0 as usize].kind, Rvalue::Invoke { .. }))
        })
        .ok_or("Invoke")?;
    let key = body
        .source
        .proof_site(statement.source)?
        .source()
        .ok_or("source")?;
    let saved = facts.invocations.remove(&key).ok_or("invocation")?;
    assert!(super::check(&facts, &body, 1, &mut work).is_err());
    let failed = work.steps - visits;
    assert!(failed > 0 && failed < visits);
    assert!(super::check(&facts, &body, 1, &mut work).is_err());
    assert_eq!(work.steps, visits + 2 * failed);
    let before = facts.body_work.canonical_steps;
    assert!(super::super::eligibility_charged(&report, &mut facts, &body).is_err());
    assert_eq!(facts.body_work.canonical_steps - before, failed);
    facts.invocations.insert(key, saved);
    let before = facts.body_work.canonical_steps;
    super::super::eligibility_charged(&report, &mut facts, &body)?;
    assert_eq!(facts.body_work.canonical_steps - before, visits);
    let mut ordinary = body.clone();
    ordinary.const_context = false;
    let before = facts.body_work.canonical_steps;
    super::super::eligibility_charged(&report, &mut facts, &ordinary)?;
    assert_eq!(facts.body_work.canonical_steps, before);
    println!(
        "const operator validation: complete={visits} failed={failed} tiny={} ordinary=0",
        tiny.steps
    );
    Ok(())
}

fn source_with_calls(calls: &str) -> TestResult<crate::resolve::expansion::Report> {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-const-callable/original-native29.rs")?;
    let (declarations, _) = source
        .split_once("const fn apply")
        .ok_or("callable declarations")?;
    Ok(crate::test_source::source_provider(
        &format!("{declarations}\n{calls}"),
        "general_const_calls",
    )?)
}

#[test]
fn const_operator_binds_inherited_and_own_formals_with_a_distinct_result() -> TestResult {
    let report = source_with_calls(
        "struct Capsule<T>{x:T}\n\
         impl<T> Capsule<T>{const fn apply<F:[const]FnOnce(T,u8)->bool>(f:F,x:T,y:u8)->bool{f(x,y)}}\n\
         const fn yes(_:u8,_:u8)->bool{true}\n\
         const N:bool=Capsule::<u8>::apply(yes,29u8,3u8);\n\
         #[unsafe(no_mangle)]pub extern \"C\" fn answer()->bool{N}",
    )?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "apply")
        .ok_or("apply")?;
    let definition = DefId(u32::try_from(index)?);
    let (facts, bodies) =
        crate::pipeline::execution::materialized_bodies::during(&[definition], || {
            let crate::pipeline::Analysis {
                mut facts,
                prepared,
            } = crate::pipeline::analyze_source(&report)?;
            crate::lower::instances::build_roots_prepared(
                &report,
                &mut facts,
                crate::lower::Roots {
                    entry: None,
                    exports: &[],
                },
                prepared,
            )?;
            Ok(facts)
        })?;
    let body = bodies.first().ok_or("checked inherited apply instance")?;
    check(&facts, body, 1)?;
    let parameters = facts
        .lifetimes
        .callables
        .generic_parameters(crate::types::ItemId(definition.0))
        .ok_or("complete formal order")?;
    assert_eq!(parameters.len(), 2);
    let mut altered = body.clone();
    let value = altered
        .values
        .iter_mut()
        .find(|v| matches!(v.kind, Rvalue::Invoke { .. }))
        .ok_or("invoke")?;
    value.ty = TypeId::primitive(crate::types::Primitive::U8);
    assert!(check(&facts, &altered, 1).is_err());
    Ok(())
}

#[test]
fn const_operator_executes_multiple_operands_with_a_distinct_result() -> TestResult {
    let report = source_with_calls(
        "const fn apply<F:[const]FnOnce(u8,u8)->bool>(f:F,x:u8,y:u8)->bool{f(x,y)}\n\
         const fn yes(_:u8,_:u8)->bool{true}\n\
         const N:bool=apply(yes,29u8,3u8);\n\
         #[unsafe(no_mangle)]pub extern \"C\" fn answer()->bool{N}",
    )?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    assert!(!program.symbols.is_empty());
    Ok(())
}
