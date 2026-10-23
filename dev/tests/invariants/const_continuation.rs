//! A real paid source provides structure; every interpreter use still pays anew.
use super::*;
use crate::{
    const_eval,
    sema::values::execution::{Progress, Requirement, Target},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn reached_payment_is_one_use_and_keeps_the_same_box_across_suspension() -> Result {
    let report = crate::test_source::source_provider(
        crate::provider_cases::UNUSED_CONDITIONAL_CONST_PAID,
        "one_use_const_payment",
    )?;
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
    context.inspect_const_call(
        Target::Initializer(definition),
        |catalog, rows, root| -> Result {
            for corrupt in [0, 1, 2, 3, 4, 5, 6, 7] {
                let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
                let mut machine =
                    Machine::catalog(evaluator, catalog, const_eval::Limits::default());
                machine.begin(root, &[]).map_err(|e| e.message)?;
                let Progress::Prepare(Requirement::FunctionBody(target)) =
                    machine.advance().map_err(|e| e.message)?
                else {
                    return Err("cached code skipped the reached payment".into());
                };
                let (mut state, _) = machine.suspend();
                let reached = state.reached_const_call(&facts, catalog, target)?;
                let slots = state.retained_slots();
                assert!(
                    state
                        .pay_const_call(&mut facts, catalog, rows, &reached, slots)
                        .is_err()
                );
                assert!(state.paid_call.is_none());
                state.pay_const_call(&mut facts, catalog, rows, &reached, usize::MAX)?;
                assert!(
                    state
                        .pay_const_call(&mut facts, catalog, rows, &reached, usize::MAX)
                        .is_err()
                );
                let pointer = std::ptr::from_ref(state.paid_call.as_deref().ok_or("payment")?);
                let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
                let mut machine =
                    Machine::catalog(evaluator, catalog, const_eval::Limits::default());
                machine.restore(state);
                let (mut state, _) = machine.suspend();
                assert_eq!(
                    pointer,
                    std::ptr::from_ref(state.paid_call.as_deref().ok_or("moved payment")?)
                );
                let paid = state.paid_call.as_mut().ok_or("payment")?;
                match corrupt {
                    1 => paid.site.frame += 1,
                    2 => paid.site.position += 1,
                    3 => paid.caller = Arc::clone(&paid.callee),
                    4 => state.frames.last_mut().ok_or("frame")?.next += 1,
                    5 => paid.callee = Arc::clone(&paid.caller),
                    6 => paid.target_code = usize::MAX,
                    7 => {
                        paid.site.form = crate::const_eval::function_calls::Form::Selected {
                            // A genuine direct-call payment cannot acquire selected
                            // callback authority by changing its cold form tag.
                            selector: paid.site.target,
                        }
                    }
                    _ => {}
                }
                let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
                let mut machine =
                    Machine::catalog(evaluator, catalog, const_eval::Limits::default());
                machine.restore(state);
                let result = machine.advance();
                if corrupt == 0 {
                    let Progress::Complete(value) = result.map_err(|e| e.message)? else {
                        return Err("paid call did not complete".into());
                    };
                    assert_eq!(value.as_scalar()?, crate::sema::model::Scalar::Integer(7));
                } else {
                    assert!(result.is_err(), "corrupt payment {corrupt} accepted");
                }
                let (state, _) = machine.suspend();
                assert!(
                    state.paid_call.is_none(),
                    "consumed or rejected payment retained"
                );
            }
            Ok(())
        },
    )??;
    Ok(())
}
