//! Cold unsizing state scales with selected requirements, never unrelated types.
use super::super::{mechanical, tests as helpers};
use super::*;
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{Kind, Primitive, TypeId, arguments::Argument as Typed},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fixture(width: usize) -> Result<(crate::resolve::expansion::Report, Declarations)> {
    let source = crate::test_source::fixture(&format!(
        "pa31/tests/spec/unsizing-roles/dyn-width-{width}.rs"
    ))?;
    let report = crate::test_source::source_provider(&source, "coercion_frontier")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    Ok((report, facts))
}
fn predicate(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
) -> Result<crate::types::parameters::predicates::Predicate> {
    let source = facts.definitions[helpers::definition(report, "A")?.0 as usize].ok_or("A")?;
    let target =
        facts.definitions[helpers::definition(report, "Target")?.0 as usize].ok_or("Target")?;
    let args = facts.types.intern_arguments(&[Typed::Type(target)])?;
    Ok(helpers::predicate(
        facts,
        helpers::definition(report, "Unsize")?,
        source,
        args,
    )?)
}

#[test]
fn unsizing_cold_capacity_and_query_work_follow_width_and_calls() -> Result {
    let mut previous_work = None;
    for width in [8, 32, 128] {
        let (report, mut facts) = fixture(width)?;
        let predicate = predicate(&report, &mut facts)?;
        let assumptions = Assumptions::default();
        let mut selected = None;
        let mut cold_capacities = None;
        let (mut context, goal) = helpers::instantiate(&mut facts, predicate)?;
        let mut cold = Search::new(helpers::caller(&assumptions, None), Availability::Validated);
        let frame = Frame::new(&facts, &mut context, &goal, 1)?;
        assert!(matches!(cold.run(&mut facts, frame)?, Decision::Proven(_)));
        println!("width={width} cold_preparation_steps={}", cold.query.steps);
        for noise in [0, 8192] {
            for n in 0..noise {
                facts.types.intern(Kind::Array {
                    element: TypeId::primitive(Primitive::U16),
                    length: n,
                })?;
            }
            let (mut context, goal) = helpers::instantiate(&mut facts, predicate)?;
            let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
            assert!(matches!(
                Search::source_unsizing_result(&mut facts, &mut frame)?,
                Some(Action::Descend)
            ));
            let pending = frame.unsizing()?;
            let capacities = (pending.goals.capacity(), pending.children.capacity());
            if let Some(expected) = cold_capacities {
                assert_eq!(capacities, expected);
            }
            cold_capacities = Some(capacities);
            let retained = size_of::<mechanical::Pending>()
                + capacities.0 * size_of::<Goal>()
                + capacities.1 * size_of::<usize>();
            println!(
                "width={width} noise={noise} Pending={} Mechanical={} goals={} goal_capacity={} child_capacity={} cold_bytes={retained}",
                size_of::<Pending>(),
                size_of::<mechanical::Pending>(),
                pending.goals.len(),
                capacities.0,
                capacities.1
            );
            assert_eq!(pending.goals.len(), width + 5);
            assert!(capacities.0 <= 2 * (width + 5));
            assert_eq!(capacities.1, 0);
            for calls in [8, 32, 128] {
                let mut total = 0;
                for _ in 0..calls {
                    let (mut context, goal) = helpers::instantiate(&mut facts, predicate)?;
                    let mut search =
                        Search::new(helpers::caller(&assumptions, None), Availability::Validated);
                    let frame = Frame::new(&facts, &mut context, &goal, 1)?;
                    let Decision::Proven(root) = search.run(&mut facts, frame)? else {
                        return Err("dyn Unsize failed".into());
                    };
                    assert!(search.query.nodes[root].children.len() >= width);
                    assert!(search.query.suspended.is_empty());
                    assert!(search.query.steps <= 4096 * (width + 1));
                    let current = (
                        search.query.steps,
                        search.query.nodes[root].children.capacity(),
                        search.query.suspended.capacity(),
                    );
                    if let Some(expected) = selected {
                        assert_eq!(current, expected);
                    }
                    selected = Some(current);
                    total += search.query.steps;
                }
                println!("width={width} noise={noise} calls={calls} steps={total}");
                assert!(total <= calls * 4096 * (width + 1));
                let (steps, children, suspended) = selected.ok_or("selected work")?;
                assert_eq!(total, steps * calls);
                println!(
                    "width={width} calls={calls} selected_children_capacity={children} selected_children_bytes={} suspended_capacity={suspended}",
                    children * size_of::<usize>()
                );
            }
        }
        let work = selected.ok_or("selected work")?.0;
        if let Some(previous) = previous_work {
            assert!(work <= 6 * previous);
        }
        previous_work = Some(work);
    }
    Ok(())
}

#[test]
fn unsizing_plan_errors_charge_work_and_plain_goals_allocate_no_language_state() -> Result {
    let (report, mut facts) = fixture(8)?;
    let predicate = predicate(&report, &mut facts)?;
    let (mut context, goal) = helpers::instantiate(&mut facts, predicate)?;
    let frame = Frame::new(&facts, &mut context, &goal, 1)?;
    assert!(frame.language.is_none());
    let source = context.known(&facts.types, TypeId::primitive(Primitive::U8))?;
    let target = context.known(&facts.types, TypeId::primitive(Primitive::Bool))?;
    let first = context.steps;
    assert!(matches!(
        plan::requirements(&mut facts, &mut context, source, target, &mut Vec::new())?,
        plan::Outcome::Absent
    ));
    assert!(context.steps > first);
    context.steps = 1_000_000;
    assert!(matches!(
        plan::requirements(&mut facts, &mut context, source, target, &mut Vec::new()),
        Err(infer::Error::Budget)
    ));
    let assumptions = Assumptions::default();
    let plain = crate::types::parameters::predicates::Predicate {
        ty: TypeId::primitive(Primitive::U8),
        bound: crate::types::parameters::predicates::Bound::Sized,
    };
    let (_, mut facts) = helpers::declarations("fn f(){}")?;
    assert!(facts.source_bindings.is_none());
    let (mut context, goal) = helpers::instantiate(&mut facts, plain)?;
    let mut frame = Frame::new(&facts, &mut context, &goal, 1)?;
    assert!(Search::source_unsizing_result(&mut facts, &mut frame)?.is_none());
    assert!(frame.language.is_none());
    let mut search = Search::new(helpers::caller(&assumptions, None), Availability::Validated);
    assert!(matches!(
        search.run(&mut facts, frame)?,
        Decision::Proven(_)
    ));
    assert!(search.query.suspended.is_empty());
    Ok(())
}
