//! Even a warm false answer requires the exact completed registration marker.
use super::*;
#[test]
fn warmed_false_answer_cannot_outlive_actual_registration_readiness() -> Result {
    let tail = r#"#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[rustc_intrinsic]const fn needs_drop<T:?Sized>()->bool;
const fn first()->bool{needs_drop::<u8>()}
"#;
    let source = SOURCE.replace(
        "const_trait_impl",
        "const_trait_impl,intrinsics,rustc_attrs",
    ) + tail;
    let report = report_source(&source)?;
    let facts = sema::declarations::analyze(&report)?;
    let mut facts = sema::bodies::analyze(&report, facts)?;
    let program = body::build(&report, &facts)?;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "first"
        })
        .ok_or("first")?;
    let code = program.bodies.iter().find(|b| matches!(b.owner, body::ir::Owner::Function { definition, .. } if definition==function.definition)).ok_or("body")?;
    let mut catalog = const_eval::code::Catalog::default();
    let index = catalog.insert(code)?;
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine =
        const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default());
    machine.begin(index, &[]).map_err(|e| e.message)?;
    let sema::values::execution::Progress::Prepare(
        sema::values::execution::Requirement::FunctionBody(target),
    ) = machine.advance().map_err(|e| e.message)?
    else {
        return Err("not a reached intrinsic".into());
    };
    let (state, _) = machine.suspend();
    let call = state
        .source_needs_drop_call(&facts, &catalog, target)?
        .ok_or("call")?;
    assert!(!catalog.evaluate_needs_drop(&facts, &call)?);
    let source = facts.source_bindings.as_deref_mut().ok_or("catalog")?;
    let actual = source.destruction_registration.ok_or("completed empty")?;
    source.destruction_registration = Some(crate::resolve::model::DefId(u32::MAX));
    assert!(
        state
            .source_needs_drop_call(&facts, &catalog, target)
            .is_err()
    );
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or("catalog")?
        .destruction_registration = None;
    assert!(
        state
            .source_needs_drop_call(&facts, &catalog, target)
            .is_err()
    );
    assert!(catalog.evaluate_needs_drop(&facts, &call).is_err());
    assert_eq!(catalog.needs_drop_result(target), Some(false));
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine =
        const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default());
    machine.restore(state);
    assert!(machine.advance().is_err());
    drop(machine);
    let source = facts.source_bindings.as_deref_mut().ok_or("catalog")?;
    source.destruction_registration = Some(actual);
    source.require_source_destructors_prepared()?;
    Ok(())
}
