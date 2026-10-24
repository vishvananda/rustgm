//! Genuine materialized caller/operand values bind each selected const entry.
use super::*;
use crate::{
    body::ir::{Body, Owner},
    const_eval::{Limits, code::Catalog},
    pipeline::execution::materialized_bodies,
    sema::values::{
        Constant, Evaluator,
        execution::{Progress, Requirement},
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn checked(
    name: &str,
    callback: bool,
) -> TestResult<(crate::resolve::expansion::Report, Declarations, Vec<Body>)> {
    let source =
        crate::test_source::fixture(&format!("pa31/tests/spec/const-eval-select-ctfe/{name}.rs"))?;
    let report = crate::test_source::source_provider(&source, "selector_guards")?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == "choose")
        .ok_or("choose")?;
    let definition = crate::resolve::model::DefId(u32::try_from(index)?);
    let mut targets = vec![definition];
    if callback {
        let index = report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == "ct")
            .ok_or("ct")?;
        targets.push(crate::resolve::model::DefId(u32::try_from(index)?));
    }
    let (analysis, bodies) =
        materialized_bodies::during(&targets, || crate::pipeline::analyze_source(&report))?;
    let body = bodies.first().ok_or("actual materialized choose")?;
    assert!(
        matches!(body.owner, Owner::Function { definition: actual, .. } if actual == definition)
    );
    Ok((report, analysis.facts, bodies))
}

fn selectors(facts: &Declarations, body: &Body) -> TestResult<Vec<(Instance, Instance)>> {
    let mut selected = Vec::new();
    for value in &body.values {
        let Rvalue::Call { callee, .. } = value.kind else {
            continue;
        };
        let ty = body.values[callee.0 as usize].ty;
        if !matches!(facts.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            continue;
        }
        let raw = Instance::new(&facts.types, ty)?;
        if let Some(call) = source_select(facts, raw)? {
            selected.push((
                body.callable_target(raw),
                body.callable_target(Instance::new(&facts.types, call.types[1])?),
            ));
        }
    }
    assert_eq!(selected.len(), 2);
    assert_ne!(selected[0].1, selected[1].1);
    Ok(selected)
}

#[test]
fn source_selector_reached_values_reject_wrong_callee_callback_tuple_and_site() -> TestResult {
    let (report, mut facts, bodies) = checked("same-item-instances", false)?;
    let body = bodies.first().ok_or("caller")?;
    let selected = selectors(&facts, body)?;
    for corrupt in 0..6 {
        let mut catalog = Catalog::default();
        let root = catalog.insert(body)?;
        let mut machine = Machine::catalog(
            Evaluator::new(&report, &mut facts),
            &catalog,
            Limits::default(),
        );
        machine.begin(root, &[]).map_err(|e| e.message)?;
        let Progress::Prepare(Requirement::FunctionBody(target)) =
            machine.advance().map_err(|e| e.message)?
        else {
            return Err("selector entered its placeholder or skipped actual F preparation".into());
        };
        let (mut state, _) = machine.suspend();
        assert_eq!(target, selected[0].1);
        let frame = state.frames.last_mut().ok_or("reached caller")?;
        let code = catalog.code(frame.body).ok_or("actual caller code")?;
        let call = validate(&facts, &catalog, code, frame, Phase::Suspended)?
            .ok_or("actual selector witness")?;
        assert_eq!(call.callback(), target);
        assert_eq!(call.width(&facts)?, 0);
        let Rvalue::Call { callee, .. } = code.values[call.value().0 as usize].kind else {
            return Err("Call".into());
        };
        let [arg, constant, runtime] = call.operands()?;
        match corrupt {
            1 => frame.values[callee.0 as usize] = Some(Constant::function(selected[1].0)),
            2 => frame.values[constant.0 as usize] = frame.values[runtime.0 as usize],
            3 => {
                frame.values[arg.0 as usize] =
                    Some(Constant::scalar(crate::sema::model::Scalar::Integer(7)));
            }
            4 => frame.body = usize::MAX,
            5 => frame.block = crate::body::ir::BlockId(u32::MAX),
            _ => {}
        }
        let result = validate(&facts, &catalog, code, frame, Phase::Suspended);
        if corrupt == 0 {
            assert!(result?.is_some());
            let position = frame.next;
            frame.next = u32::MAX;
            assert!(validate(&facts, &catalog, code, frame, Phase::Suspended).is_err());
            frame.next = position;
            assert!(
                state
                    .reached_const_call(&facts, &catalog, selected[1].1)
                    .is_err()
            );
        } else {
            assert!(result.is_err(), "corrupt selector witness {corrupt}");
        }
        assert!(state.paid_call.is_none());
    }
    Ok(())
}

#[test]
fn source_selector_warm_body_executes_f_twice_without_selector_or_g_frames() -> TestResult {
    let (report, mut facts, bodies) = checked("repeated-empty", true)?;
    let caller = bodies.first().ok_or("caller")?;
    let callback = bodies.get(1).ok_or("callback")?;
    println!(
        "SelectorCall={} Machine={} State={} Frame={} Code={}",
        size_of::<Call<'_>>(),
        size_of::<Machine<'_, '_>>(),
        size_of::<crate::const_eval::State>(),
        size_of::<Frame>(),
        size_of::<crate::const_eval::code::Code>()
    );
    let mut catalog = Catalog::default();
    let root = catalog.insert(caller)?;
    let mut machine = Machine::catalog(
        Evaluator::new(&report, &mut facts),
        &catalog,
        Limits::default(),
    );
    machine.begin(root, &[]).map_err(|e| e.message)?;
    let Progress::Prepare(Requirement::FunctionBody(target)) =
        machine.advance().map_err(|e| e.message)?
    else {
        return Err("cold selector did not request its actual F".into());
    };
    let (state, _) = machine.suspend();
    assert_eq!(
        state.work.calls, 1,
        "no selector placeholder frame before F readiness"
    );
    let mut state = Some(state);
    let code = catalog.insert(callback)?;
    catalog.functions.insert(target, code);
    for cold in [true, false] {
        let mut machine = Machine::catalog(
            Evaluator::new(&report, &mut facts),
            &catalog,
            Limits::default(),
        );
        if cold {
            machine.restore(state.take().ok_or("suspended source caller")?);
        } else {
            machine.begin(root, &[]).map_err(|e| e.message)?;
        }
        let Progress::Complete(value) = machine.advance().map_err(|e| e.message)? else {
            return Err("warm ordinary F still required preparation".into());
        };
        assert_eq!(value.as_scalar()?, crate::sema::model::Scalar::Integer(7));
        assert_eq!(
            machine.work.calls, 3,
            "one caller and two actual F frames only"
        );
        assert_eq!(machine.work.peak_frames, 2);
    }
    Ok(())
}

#[test]
fn source_selector_attempted_work_rejects_fresh_and_warm_cached_entry() -> TestResult {
    let (report, mut facts, bodies) = checked("repeated-empty", true)?;
    let caller = bodies.first().ok_or("caller")?;
    let callback = bodies.get(1).ok_or("callback")?;
    let target = caller
        .values
        .iter()
        .find_map(|v| {
            let Rvalue::Call { callee, .. } = v.kind else {
                return None;
            };
            let raw = Instance::new(&facts.types, caller.values[callee.0 as usize].ty).ok()?;
            let call = source_select(&facts, raw).ok()??;
            Some(caller.callable_target(Instance::new(&facts.types, call.types[1]).ok()?))
        })
        .ok_or("actual F target")?;
    for warm in [false, true] {
        let mut catalog = Catalog::default();
        let root = catalog.insert(caller)?;
        if warm {
            let entry = catalog.insert(callback)?;
            catalog.functions.insert(target, entry);
        }
        let mut machine = Machine::catalog(
            Evaluator::new(&report, &mut facts),
            &catalog,
            Limits {
                steps: 128,
                ..Limits::default()
            },
        );
        machine.begin(root, &[]).map_err(|e| e.message)?;
        let error = machine
            .advance()
            .err()
            .ok_or("selector budget was bypassed")?;
        assert_eq!(error.message, "constant execution step limit exceeded");
        assert!(machine.work.steps > 128, "failed attempts remain charged");
        assert_eq!(
            machine.work.calls, 1,
            "no callback or placeholder frame after failed budget"
        );
    }
    Ok(())
}

#[test]
fn selector_preparation_visits_both_calls_after_early_or_owner_retention() -> TestResult {
    for name in ["same-item-instances", "conditional-owner-two-selectors"] {
        let (_, mut facts, bodies) = checked(name, false)?;
        let body = bodies.first().ok_or("caller")?;
        selectors(&facts, body)?;
        let before = facts.body_work.canonical_steps;
        assert!(crate::const_eval::use_inputs::Inputs::prepare(
            &mut facts, body
        )?);
        assert_eq!(
            facts.body_work.canonical_steps - before,
            body.values.len() + 10,
            "{name}"
        );
        let before = facts.body_work.canonical_steps;
        let catalog = facts.source_bindings.take();
        assert!(!crate::const_eval::use_inputs::Inputs::prepare(
            &mut facts, body
        )?);
        facts.source_bindings = catalog;
        let mut ordinary = body.clone();
        ordinary.const_context = false;
        assert!(!crate::const_eval::use_inputs::Inputs::prepare(
            &mut facts, &ordinary
        )?);
        assert_eq!(
            facts.body_work.canonical_steps, before,
            "fast paths must inspect no values"
        );
    }
    Ok(())
}

#[test]
fn selector_opponents_cannot_replace_the_actual_caller_owner_or_call_signatures() -> TestResult {
    let (report, mut facts, bodies) = checked("same-item-instances", false)?;
    let body = bodies.first().ok_or("caller")?;
    let mut catalog = Catalog::default();
    let root = catalog.insert(body)?;
    let mut machine = Machine::catalog(
        Evaluator::new(&report, &mut facts),
        &catalog,
        Limits::default(),
    );
    machine.begin(root, &[]).map_err(|e| e.message)?;
    let Progress::Prepare(Requirement::FunctionBody(_)) =
        machine.advance().map_err(|e| e.message)?
    else {
        return Err("missing reached selector".into());
    };
    let (state, _) = machine.suspend();
    let frame = state.frames.last().ok_or("caller frame")?;
    let call = validate(
        &facts,
        &catalog,
        catalog.code(root).ok_or("Code")?,
        frame,
        Phase::Suspended,
    )?
    .ok_or("Call")?;
    let id = call.value();
    let Owner::Function {
        definition,
        signature,
        ..
    } = body.owner
    else {
        return Err("owner".into());
    };
    let Rvalue::Call {
        callee, arguments, ..
    } = body.values[id.0 as usize].kind
    else {
        return Err("Call".into());
    };
    let byte = TypeId::primitive(crate::types::Primitive::U8);
    let parameters = facts.types.intern_list(&[byte])?;
    let wrong_signature = facts.types.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: byte,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    for corrupt in 0..5 {
        // Only opponents clone a proved source body and invalidate one field.
        // No positive permission is fabricated from these snapshots.
        let mut wrong = body.clone();
        match corrupt {
            0 => wrong.owner = Owner::AnonymousConstant,
            1 => {
                wrong.owner = Owner::Function {
                    definition,
                    signature: wrong_signature,
                    execution: None,
                }
            }
            2 => {
                wrong.owner = Owner::Function {
                    definition,
                    signature,
                    execution: Some(wrong_signature),
                }
            }
            3 => {
                wrong.values[id.0 as usize].kind = Rvalue::Call {
                    callee,
                    arguments,
                    signature: TypeId::UNIT,
                }
            }
            4 => wrong.values[id.0 as usize].ty = TypeId::UNIT,
            _ => return Err("opponent".into()),
        }
        let mut other = Catalog::default();
        let index = other.insert(&wrong)?;
        assert_eq!(index, root);
        assert!(
            validate(
                &facts,
                &other,
                other.code(index).ok_or("opponent Code")?,
                frame,
                Phase::Suspended
            )
            .is_err(),
            "opponent {corrupt}"
        );
    }
    Ok(())
}
