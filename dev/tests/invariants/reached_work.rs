//! Every repeated reached use pays afresh; immutable rows do not rescan other code.
use super::*;
use crate::sema::values::execution::{Progress, Requirement, Target};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-const-payment/work")
            .join(format!("{name}.rs")),
    )?)
}
fn replay(
    report: &crate::resolve::expansion::Report,
    facts: &mut crate::sema::model::Declarations,
    catalog: &Catalog,
    rows: &Inputs,
    root: usize,
) -> TestResult<(usize, usize)> {
    let before = facts.body_work.canonical_steps;
    let mut pending = None;
    let mut payments = 0;
    loop {
        let evaluator = crate::sema::values::Evaluator::new(report, facts);
        let mut machine = Machine::catalog(evaluator, catalog, super::super::Limits::default());
        if let Some(state) = pending.take() {
            machine.restore(state);
        } else {
            machine.begin(root, &[]).map_err(|e| e.message)?;
        }
        match machine.advance().map_err(|e| e.message)? {
            Progress::Complete(value) => {
                assert_eq!(value.as_scalar()?, crate::sema::model::Scalar::Integer(7));
                drop(machine);
                return Ok((payments, facts.body_work.canonical_steps - before));
            }
            Progress::Prepare(Requirement::FunctionBody(target)) => {
                let (mut state, _) = machine.suspend();
                assert!(state.paid_call.is_none());
                let reached = state.reached_const_call(facts, catalog, target)?;
                state.pay_const_call(facts, catalog, rows, &reached, usize::MAX)?;
                pending = Some(state);
                payments += 1;
            }
            Progress::Prepare(_) => {
                return Err("work control reached an unrelated prerequisite".into());
            }
        }
    }
}
fn observe(name: &str, expected: usize, requirements: usize) -> TestResult<(usize, usize, usize)> {
    let report = crate::test_source::source_provider(&fixture(name)?, "const_payment_work")?;
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
        .ok_or("N")?
        .definition;
    let context = prepared.constants.as_ref().ok_or("provider")?;
    context.inspect_const_call(Target::Initializer(definition), |catalog, rows, root| -> TestResult<_> {
        let storage = rows.observations();
        assert_eq!(storage.1, requirements);
        let (first, work) = replay(&report, &mut facts, catalog, rows, root)?;
        let (second, warm) = replay(&report, &mut facts, catalog, rows, root)?;
        assert_eq!(first, expected); assert_eq!(second, expected);
        assert_eq!(rows.observations(), storage);
        assert!(work > 0 && warm > 0);
        println!("{name}: payments={first}/{second} work={work}/{warm} rows={} requirements={} bytes={}", storage.0, storage.1, storage.2);
        Ok((warm, storage.1, storage.2))
    })?
}
#[test]
fn reached_calls_instances_and_unrelated_materialization_have_bounded_work() -> TestResult {
    for family in ["repeated-reached", "distinct-callers", "distinct-instances"] {
        let mut previous = None;
        for width in [8, 32, 128] {
            let callers = family == "distinct-callers";
            let requirements = if family == "repeated-reached" {
                1
            } else {
                width + usize::from(callers)
            };
            let measured = observe(
                &format!("{family}-{width}"),
                width * if callers { 2 } else { 1 },
                requirements,
            )?;
            if let Some((work, bytes)) = previous {
                assert!(measured.0 <= work * 5, "{family} repeated prefix work");
                assert!(measured.2 <= bytes * 5, "{family} retained prefix growth");
            }
            previous = Some((measured.0, measured.2));
        }
    }
    assert_eq!(
        observe("unrelated-noise-0", 8, 1)?,
        observe("unrelated-noise-128", 8, 1)?
    );
    Ok(())
}
