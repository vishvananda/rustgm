//! Mechanical record checks supplement genuine execution; a record never proves a body.
use super::*;
use crate::{pipeline, resolve::expansion::Report};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn source() -> TestResult<(Report, pipeline::Analysis)> {
    let input = crate::test_source::fixture("pa25/tests/spec/const-closures/execution-pure.rs")?;
    let report = crate::test_source::source_provider(&input, "sealed_capture_records")?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn shared<'a>(report: &Report, analysis: &'a pipeline::Analysis) -> TestResult<&'a Body> {
    let owner = analysis
        .facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "shared"
        })
        .ok_or("actual shared function")?
        .definition;
    analysis
        .prepared
        .concrete
        .get(&crate::body::ir::BodyKey::Declaration(owner))
        .ok_or_else(|| "retained shared original".into())
}
fn records(types: &mut Store, body: &Body) -> TestResult<super::super::Prepared> {
    let mut work = 0;
    let plan = Plan::collect(types, body, &mut work)?;
    Ok(super::super::Prepared::from_proved(
        types,
        body,
        Default::default(),
        None,
        64 * 1024 * 1024,
        &mut work,
        Preparation { plan, pay: None },
    )?)
}
fn location(code: &Code) -> TestResult<(usize, ValueId, ValueId)> {
    code.operands
        .iter()
        .find_map(|row| match row {
            PreparedOperand::ClosureCall { destination, call } => {
                Some((call.statement as usize, *destination, call.receiver))
            }
            _ => None,
        })
        .ok_or_else(|| "actual captured call record".into())
}

#[test]
fn binding_rejects_prepublication_header_source_and_operand_changes() -> TestResult {
    let (report, mut analysis) = source()?;
    let body = shared(&report, &analysis)?.clone();
    for fault in 0..8 {
        let mut prepared = records(&mut analysis.facts.types, &body)?;
        let code = &mut prepared.code;
        let (statement, destination, receiver) = location(code)?;
        match fault {
            0 => code.const_context = false,
            1 => {
                let Owner::Function { definition, .. } = code.owner else {
                    return Err("actual function owner".into());
                };
                let actual = analysis.facts.definitions[definition.0 as usize]
                    .ok_or("actual function type")?;
                code.instance = if code.instance.is_some() {
                    None
                } else {
                    Some(Instance::new(&analysis.facts.types, actual)?)
                };
            }
            2 => {
                code.source = analysis
                    .facts
                    .functions
                    .first()
                    .ok_or("actual function")?
                    .node
                    .into()
            }
            3 => {
                let changed = code.source.root_body_site();
                assert_ne!(code.statements[statement].source, changed);
                code.statements[statement].source = changed;
            }
            4 => code.values[destination.0 as usize].ty = TypeId::UNIT,
            5 => code.values[receiver.0 as usize].ty = TypeId::UNIT,
            6 => {
                let Rvalue::Invoke { signature, .. } =
                    &mut code.values[destination.0 as usize].kind
                else {
                    return Err("actual Invoke".into());
                };
                *signature = TypeId::UNIT;
            }
            _ => code.statements[statement].action = Action::Evaluate(receiver),
        }
        assert!(
            Code::bind_closure_context(&Rc::new(prepared.code)).is_err(),
            "fault {fault}"
        );
    }
    Ok(())
}

#[test]
fn exact_records_replay_but_foreign_store_code_and_call_rows_reject() -> TestResult {
    let (report, mut analysis) = source()?;
    let body = shared(&report, &analysis)?.clone();
    let original = Rc::new(records(&mut analysis.facts.types, &body)?.code);
    Code::bind_closure_context(&original)?;
    let (statement, destination, _) = location(&original)?;
    let statement = u32::try_from(statement)?;
    for _ in 0..2 {
        let mut work = 0;
        assert!(
            original
                .closure_call(&analysis.facts.types, statement, destination, &mut work)?
                .is_some()
        );
        assert!(work > 0);
    }
    let (_, foreign) = source()?;
    assert!(
        original
            .require_closure_context(&foreign.facts.types)
            .is_err()
    );
    let mut plain = Code::snapshot(&body)?;
    assert!(
        plain
            .require_closure_context(&analysis.facts.types)
            .is_err()
    );
    plain.operands.clone_from(&original.operands);
    assert!(
        plain
            .require_closure_context(&analysis.facts.types)
            .is_err()
    );
    let mut another = records(&mut analysis.facts.types, &body)?;
    let old = original
        .operands
        .iter()
        .find(|row| matches!(row, PreparedOperand::ClosureCall { .. }))
        .ok_or("original row")?
        .clone();
    let selected = another
        .code
        .operands
        .iter_mut()
        .find(|row| matches!(row, PreparedOperand::ClosureCall { .. }))
        .ok_or("new row")?;
    *selected = old;
    assert!(Code::bind_closure_context(&Rc::new(another.code)).is_err());
    Ok(())
}

#[test]
fn rejected_cold_growth_is_paid_before_record_publication() -> TestResult {
    use std::cell::Cell;
    let (report, mut analysis) = source()?;
    let body = shared(&report, &analysis)?;
    let mut code = Code::snapshot(body)?;
    let mut work = 0;
    let plan = Plan::collect(&analysis.facts.types, body, &mut work)?;
    let before = code.operands.len();
    let paid = Cell::new((0usize, 0usize));
    let pay = |w: usize, b: usize| {
        paid.set((
            paid.get().0.saturating_add(w),
            paid.get().1.saturating_add(b),
        ));
        Ok(())
    };
    let body = body.clone();
    assert!(
        plan.install(
            &mut code,
            &mut analysis.facts.types,
            &body,
            None,
            Cost {
                budget: 0,
                work: &mut work,
                pay: Some(&pay)
            }
        )
        .is_err()
    );
    assert!(paid.get().0 > 0 && paid.get().1 > 0);
    assert_eq!(code.operands.len(), before);
    assert!(code.closure_context().is_none());
    Ok(())
}
