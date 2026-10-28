//! Warm physical layout never changes a reached field operand or caller contract.
use super::*;
use crate::{body, sema, test_source, types::Kind};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn checked() -> Result<(
    crate::resolve::expansion::Report,
    Declarations,
    body::ir::Program,
)> {
    let source =
        test_source::fixture("pa31/tests/spec/source-offset-of/provider/reached-provider.rs")?;
    let prefix = source.split("fn payload").next().ok_or("source prefix")?;
    let source = format!(
        "{prefix}const fn first()->usize{{offset_of::<Fields>(0,0)}}const fn second()->usize{{offset_of::<Fields>(0,1)}}const fn third()->usize{{offset_of::<Fields>(0,2)}}const fn malformed()->usize{{offset_of::<Fields>(0,0)}}"
    );
    let report = test_source::source_provider(&source, "offset_operands")?;
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
    program.bodies.iter().find(|b| matches!(b.owner,body::ir::Owner::Function{definition,..}
        if report.package.names.get(report.resolution.definitions[definition.0 as usize].name)==name))
        .ok_or_else(|| format!("missing {name}").into())
}
#[test]
fn warm_layout_keeps_distinct_operands_and_rejects_changed_current_calls() -> Result {
    let (report, mut facts, program) = checked()?;
    // Declarations may prepare this closed container eagerly. Exercise a cold
    // physical query without changing any typed source/header/field facts.
    facts.layouts = crate::abi::layout::Layouts::default();
    let mut catalog = Catalog::default();
    let first = catalog.insert(named(&report, &program, "first")?)?;
    let second = catalog.insert(named(&report, &program, "second")?)?;
    let third = catalog.insert(named(&report, &program, "third")?)?;
    let mut malformed = named(&report, &program, "malformed")?.clone();
    let destination = malformed
        .values
        .iter()
        .position(|v| matches!(v.kind, body::ir::Rvalue::Call { .. }))
        .ok_or("call")?;
    malformed.values[destination].ty = TypeId::primitive(Primitive::U8);
    let wrong = catalog.insert(&malformed)?;
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::catalog(evaluator, &catalog, super::super::Limits::default());
    machine.begin(first, &[]).map_err(|e| e.message)?;
    let sema::values::execution::Progress::Prepare(super::super::Requirement::FunctionBody(target)) =
        machine.advance().map_err(|e| e.message)?
    else {
        return Err("expected concrete layout demand".into());
    };
    let (state, _) = machine.suspend();
    let call = state
        .source_offset_call(&facts, &catalog, target)?
        .ok_or("call witness")?;
    let frame = state.frames.last().ok_or("frame")?;
    assert!(
        validate(
            &facts,
            catalog.code(first).ok_or("code")?,
            frame,
            frame.next + 1,
            target
        )
        .is_err()
    );
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(target.ty()) else {
        return Err("signature".into());
    };
    let arguments = facts
        .types
        .intern_arguments(&[crate::types::arguments::Argument::Type(TypeId::primitive(
            Primitive::U32,
        ))])?;
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(target.ty()) else {
        return Err("item".into());
    };
    let changed = facts.types.intern(Kind::FunctionItem {
        item,
        arguments,
        signature,
    })?;
    assert!(
        state
            .source_offset_call(&facts, &catalog, Instance::new(&facts.types, changed)?)
            .is_err()
    );
    facts
        .aggregates
        .prepare_storage_fields(&mut facts.types, call.element())?;
    facts
        .layouts
        .demand(&facts.types, &facts.aggregates, call.element())?;
    assert!(catalog.layout_result(target).is_none());
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::catalog(evaluator, &catalog, super::super::Limits::default());
    machine.restore(state);
    let sema::values::execution::Progress::Complete(value) =
        machine.advance().map_err(|e| e.message)?
    else {
        return Err("warm first call".into());
    };
    assert_eq!(
        value.as_scalar().map_err(sema::values::Failure::message)?,
        Scalar::Integer(0)
    );
    for (body, expected) in [(second, 8), (third, 16)] {
        machine.begin(body, &[]).map_err(|e| e.message)?;
        let sema::values::execution::Progress::Complete(value) =
            machine.advance().map_err(|e| e.message)?
        else {
            return Err("warm field call".into());
        };
        assert_eq!(
            value.as_scalar().map_err(sema::values::Failure::message)?,
            Scalar::Integer(expected)
        );
    }
    machine.begin(wrong, &[]).map_err(|e| e.message)?;
    assert!(machine.advance().is_err());
    assert!(catalog.layout_result(target).is_none());
    Ok(())
}
