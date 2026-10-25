//! Runtime policy cannot replace actual reached CTFE call and fallback proofs.
use super::*;
use crate::{body, sema, types::TypeId};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn checked() -> Result<(
    crate::resolve::expansion::Report,
    Declarations,
    body::ir::Program,
)> {
    let source = r#"#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]
        #[lang="pointee_sized"]trait PointeeSized{}
        #[lang="meta_sized"]trait MetaSized:PointeeSized{}
        #[lang="sized"]trait Sized:MetaSized{}
        #[rustc_intrinsic]const fn ub_checks()->bool{false}
        const fn good()->bool{ub_checks()}
        const fn bad()->bool{ub_checks()}"#;
    let report = crate::test_source::source_provider(source, "ub_calls")?;
    let facts = sema::declarations::analyze(&report)?;
    let facts = sema::bodies::analyze(&report, facts)?;
    let program = body::build(&report, &facts)?;
    Ok((report, facts, program))
}
fn named<'a>(
    report: &crate::resolve::expansion::Report,
    program: &'a body::ir::Program,
    name: &str,
) -> Result<&'a body::ir::Body> {
    program.bodies.iter().find(|body| matches!(body.owner,
        body::ir::Owner::Function { definition, .. }
        if report.package.names.get(report.resolution.definitions[definition.0 as usize].name) == name))
        .ok_or_else(|| format!("missing {name}").into())
}
fn deny_foreign_code_and_invoke(
    machine: &mut Machine<'_, '_>,
    actual: &Code,
    copied: &Code,
    frame: &Frame,
    target: Instance,
) -> Result {
    let error = machine
        .source_runtime_check(copied, frame, frame.next, target)
        .err()
        .ok_or("foreign Code accepted")?;
    assert_eq!(
        error.message(),
        "source ub_checks belongs to another actual caller code"
    );
    let error = machine
        .function_call(
            actual,
            frame,
            target,
            crate::const_eval::function_calls::Form::Invoke,
        )
        .err()
        .ok_or("UB intrinsic Invoke granted direct-call permission")?;
    assert_eq!(
        error.message(),
        "source intrinsic Invoke execution is not implemented"
    );
    Ok(())
}

fn complete_intrinsic_call(machine: &mut Machine<'_, '_>, root_calls: usize) -> Result {
    let crate::sema::values::execution::Progress::Complete(value) =
        machine.advance().map_err(|e| e.message)?
    else {
        return Err("actual checked UB call remained pending".into());
    };
    assert_eq!(value.as_scalar()?, sema::model::Scalar::Boolean(true));
    assert_eq!(machine.work().calls, root_calls + 1);
    assert!(machine.work().steps >= 32);
    Ok(())
}

#[test]
fn ub_const_calls_require_actual_direct_caller_and_checked_fallback_even_when_repeated() -> Result {
    let (report, mut facts, program) = checked()?;
    let good_body = named(&report, &program, "good")?;
    let mut catalog = Catalog::default();
    let good = catalog.insert(good_body)?;
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::catalog(evaluator, &catalog, crate::const_eval::Limits::default());
    machine.begin(good, &[]).map_err(|e| e.message)?;
    let crate::sema::values::execution::Progress::Prepare(Requirement::FunctionBody(target)) =
        machine.advance().map_err(|e| e.message)?
    else {
        return Err("missing actual fallback prerequisite".into());
    };
    let (state, _) = machine.suspend();
    let root_calls = state.work.calls;
    assert_eq!(
        root_calls, 1,
        "root entry is counted before a pending intrinsic completes"
    );
    let selection = state
        .source_runtime_check_call(&facts, &catalog, target)?
        .ok_or("selection")?;
    assert!(selection.fallback);
    let frame = state.frames.last().ok_or("actual frame")?;
    assert!(
        validate(
            &facts,
            catalog.code(good).ok_or("caller")?,
            frame,
            frame.next + 1,
            target
        )
        .is_err()
    );
    let copied = Code::snapshot(good_body)?;
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::catalog(evaluator, &catalog, crate::const_eval::Limits::default());
    deny_foreign_code_and_invoke(
        &mut machine,
        catalog.code(good).ok_or("actual caller code")?,
        &copied,
        frame,
        target,
    )?;
    drop(machine);
    catalog.insert(named(&report, &program, "ub_checks")?)?;
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::catalog(evaluator, &catalog, crate::const_eval::Limits::default());
    machine.restore(state);
    complete_intrinsic_call(&mut machine, root_calls)?;
    drop(machine);
    for _ in 0..2 {
        let evaluator = sema::values::Evaluator::new(&report, &mut facts);
        let mut machine =
            Machine::catalog(evaluator, &catalog, crate::const_eval::Limits::default());
        machine.begin(good, &[]).map_err(|e| e.message)?;
        let root_calls = machine.work().calls;
        assert_eq!(root_calls, 1);
        complete_intrinsic_call(&mut machine, root_calls)?;
    }
    let mut bad_body = named(&report, &program, "bad")?.clone();
    let destination = bad_body
        .values
        .iter_mut()
        .find(|v| matches!(v.kind, body::ir::Rvalue::Call { .. }))
        .ok_or("call")?;
    destination.ty = TypeId::UNIT;
    let bad = catalog.insert(&bad_body)?;
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::catalog(evaluator, &catalog, crate::const_eval::Limits::default());
    machine.begin(bad, &[]).map_err(|e| e.message)?;
    assert_eq!(
        machine.advance().err().map(|e| e.message),
        Some("source intrinsic call differs from its checked const contract")
    );
    Ok(())
}
