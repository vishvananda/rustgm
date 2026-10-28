//! Actual reached calls charge every attempted typed operand before inspecting it.
use super::*;
use crate::{
    body::ir::{Action, Rvalue},
    const_eval::{Constant, Limits},
    sema::model::{Declarations, Scalar},
};

fn source(width: usize) -> TestResult<String> {
    let (prefix, _) = crate::provider_cases::UNUSED_CONDITIONAL_CONST_PAID
        .split_once("const fn unused")
        .ok_or("paid source prefix")?;
    let parameters = (0..width)
        .map(|i| format!("a{i}:u8"))
        .collect::<Vec<_>>()
        .join(",");
    let arguments = (0..width)
        .map(|i| if i + 1 == width { "7u8" } else { "0u8" })
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!(
        "{prefix}const fn unused<T:[const]Clone>({parameters})->u8{{a{last}}}\n\
        const N:u8=unused::<u8>({arguments});\n\
        #[unsafe(no_mangle)]pub extern \"C\" fn answer()->u8{{N}}",
        last = width - 1
    ))
}

fn suspended(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    catalog: &Catalog,
    root: usize,
    limits: Limits,
) -> TestResult<(State, Instance)> {
    let evaluator = crate::sema::values::Evaluator::new(report, facts);
    let mut machine = Machine::catalog(evaluator, catalog, limits);
    machine.begin(root, &[]).map_err(|e| e.message)?;
    let Progress::Prepare(Requirement::FunctionBody(target)) =
        machine.advance().map_err(|e| e.message)?
    else {
        return Err("cached body skipped its actual reached payment".into());
    };
    Ok((machine.suspend().0, target))
}

fn corrupt(state: &mut State, catalog: &Catalog, index: usize) -> TestResult {
    let frame = state.frames.last_mut().ok_or("actual reached frame")?;
    let body = catalog.code(frame.body).ok_or("actual caller Code")?;
    let Action::Evaluate(value) = body
        .statements
        .get(frame.next as usize)
        .ok_or("actual statement")?
        .action
    else {
        return Err("reached evaluation".into());
    };
    let Rvalue::Call { arguments, .. } = body.values[value.0 as usize].kind else {
        return Err("reached direct call".into());
    };
    let arguments = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("arguments")?;
    let operand = arguments.get(index).ok_or("actual typed operand")?;
    let slot = frame
        .values
        .get_mut(operand.0 as usize)
        .ok_or("evaluated operand slot")?;
    assert!(slot.is_some());
    *slot = Some(Constant::scalar(Scalar::Boolean(true)));
    Ok(())
}

fn rejected(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    catalog: &Catalog,
    rows: &Inputs,
    root: usize,
    index: usize,
    budget: Option<usize>,
) -> TestResult<usize> {
    let limits = Limits::default();
    let (mut state, target) = suspended(report, facts, catalog, root, limits)?;
    corrupt(&mut state, catalog, index)?;
    let before = state.work.steps;
    let steps = budget.map_or(limits.steps, |extra| before + extra);
    let error = state
        .reached_const_call(facts, catalog, rows, target, steps)
        .err()
        .ok_or("malformed actual scalar acquired a reached witness")?;
    if budget.is_some() {
        assert_eq!(error, "constant execution step limit exceeded");
    } else {
        assert!(error.contains("scalar differs"), "{error}");
    }
    assert!(state.paid_call.is_none());
    assert!(state.work.steps > before);
    Ok(state.work.steps - before)
}

fn observe(width: usize) -> TestResult<usize> {
    let report = crate::test_source::source_provider(&source(width)?, "arity_payment_work")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let definition = facts
        .initializers
        .iter()
        .find(|i| {
            report
                .package
                .names
                .get(report.resolution.definitions[i.definition.0 as usize].name)
                == "N"
        })
        .ok_or("actual N")?
        .definition;
    let context = prepared.constants.as_ref().ok_or("source provider")?;
    context.inspect_const_call(
        Target::Initializer(definition),
        |catalog, rows, root| -> TestResult<usize> {
            let limits = Limits::default();
            let storage = rows.observations();
            let (mut state, target) = suspended(&report, &mut facts, catalog, root, limits)?;
            let before = state.work.steps;
            let reached = state.reached_const_call(&facts, catalog, rows, target, limits.steps)?;
            let cold = state.work.steps - before;
            let before = state.work.steps;
            state.reached_const_call(&facts, catalog, rows, target, limits.steps)?;
            assert_eq!(state.work.steps - before, cold);
            state.pay_const_call(&mut facts, catalog, rows, &reached, limits)?;
            let paid_steps = state.work.steps;
            let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
            let mut machine = Machine::catalog(
                evaluator,
                catalog,
                Limits {
                    steps: paid_steps,
                    ..limits
                },
            );
            machine.restore(state);
            let error = machine
                .advance()
                .err()
                .ok_or("paid resume skipped the configured step budget")?;
            assert_eq!(error.message, "constant execution step limit exceeded");
            let (state, _) = machine.suspend();
            assert!(state.paid_call.is_none());
            assert!(state.work.steps > paid_steps);
            let first = rejected(&report, &mut facts, catalog, rows, root, 0, None)?;
            let last = rejected(&report, &mut facts, catalog, rows, root, width - 1, None)?;
            assert_eq!(last - first, 4 * (width - 1));
            rejected(
                &report,
                &mut facts,
                catalog,
                rows,
                root,
                width - 1,
                Some(cold / 2),
            )?;
            assert_eq!(rows.observations(), storage);
            println!("paid_arity={width} cold_warm={cold} rejected_first={first} last={last}");
            Ok(cold)
        },
    )?
}

#[test]
fn attempted_arity_work_is_linear_warm_and_preserved_on_rejection_and_resume() -> TestResult {
    let small = observe(8)?;
    let medium = observe(32)?;
    let large = observe(128)?;
    assert!(medium > small);
    assert_eq!(large - medium, 4 * (medium - small));
    Ok(())
}
