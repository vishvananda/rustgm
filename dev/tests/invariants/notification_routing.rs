//! Body trait watches share an inference context without consuming projection notifications.
use super::Notifications;
use crate::types::{
    Store, TypeId,
    infer::{Context, Domain, Instantiation, TermId, arguments::Argument, predicates::Predicate},
    parameters::predicates::{Bound, Predicate as Canonical},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn goal(context: &mut Context, store: &mut Store, input: TermId) -> TestResult<Predicate> {
    let mut substitution = Instantiation::new(context, store, &[])?;
    let goal = context.instantiate_predicate(
        store,
        Canonical {
            ty: TypeId::UNIT,
            bound: Bound::Sized,
        },
        &mut substitution,
    )?;
    Ok(goal.map_arguments(|_| Ok(Argument::Type(input)))?)
}
#[test]
fn projection_and_trait_events_reach_their_original_owners() -> TestResult {
    let mut store = Store::default();
    let mut context = Context::default();
    let mut events = Notifications::default();
    let projection = context.variable(Domain::Any)?;
    let projection_watch = context.watch_resolution(&[projection], &[])?;
    let first = context.variable(Domain::Any)?;
    let second = context.variable(Domain::Any)?;
    let a = goal(&mut context, &mut store, first)?;
    let b = goal(&mut context, &mut store, second)?;
    events.observe(&mut context, &store, 0, &a, None)?;
    events.observe(&mut context, &store, 1, &b, None)?;
    let unit = context.known(&store, TypeId::UNIT)?;
    context.equate(&store, projection, unit)?;
    context.equate(&store, second, unit)?;
    events.observe(&mut context, &store, 0, &a, None)?;
    assert_eq!(
        events.next_projection(&mut context, &store)?,
        Some(projection_watch)
    );
    assert_eq!(events.next_goal(&mut context)?, Some(1));
    assert_eq!(events.next_goal(&mut context)?, None);
    assert_eq!(events.next_projection(&mut context, &store)?, None);
    events.complete(&mut context, 1)?;
    assert_eq!(events.first_pending(), Some(0));
    context.equate(&store, first, unit)?;
    assert_eq!(events.next_projection(&mut context, &store)?, None);
    assert_eq!(events.next_goal(&mut context)?, Some(0));
    events.complete(&mut context, 0)?;
    assert!(!events.waiting());
    Ok(())
}
#[test]
fn independent_pending_goals_have_linear_registration_and_delivery_work() -> TestResult {
    for count in [16, 128, 1024, 4096] {
        let mut store = Store::default();
        let mut context = Context::default();
        let mut events = Notifications::default();
        let mut inputs = Vec::new();
        for index in 0..count {
            let input = context.variable(Domain::Any)?;
            let predicate = goal(&mut context, &mut store, input)?;
            events.observe(&mut context, &store, index, &predicate, None)?;
            inputs.push(input);
        }
        let registration = context.steps;
        let unit = context.known(&store, TypeId::UNIT)?;
        for (index, input) in inputs.into_iter().enumerate().rev() {
            context.equate(&store, input, unit)?;
            assert_eq!(events.next_projection(&mut context, &store)?, None);
            assert_eq!(events.next_goal(&mut context)?, Some(index));
            events.complete(&mut context, index)?;
        }
        eprintln!(
            "body notifications count={count} registration={registration} total={}",
            context.steps
        );
        assert!(registration <= 32 * count);
        assert!(context.steps <= 64 * count);
        assert!(!events.waiting());
    }
    Ok(())
}

#[test]
fn owned_prefix_events_do_not_consume_projection_or_foreign_goal_events() -> TestResult {
    use crate::syntax::ast::NodeId;
    let mut store = Store::default();
    let mut context = Context::default();
    let mut events = Notifications::default();
    let owner = NodeId::new(10);
    let foreign = NodeId::new(20);
    events.own(owner, &[0])?;
    events.own(foreign, &[1])?;
    assert_eq!(events.next_owned(&mut context, owner)?, Some(0));
    assert_eq!(events.next_owned(&mut context, foreign)?, Some(1));
    let first = context.variable(Domain::Any)?;
    let second = context.variable(Domain::Any)?;
    let first_goal = goal(&mut context, &mut store, first)?;
    let second_goal = goal(&mut context, &mut store, second)?;
    events.observe(&mut context, &store, 0, &first_goal, None)?;
    events.observe(&mut context, &store, 1, &second_goal, None)?;
    let projection = context.variable(Domain::Any)?;
    let watch = context.watch_resolution(&[projection], &[])?;
    let unit = context.known(&store, TypeId::UNIT)?;
    context.equate(&store, second, unit)?;
    context.equate(&store, projection, unit)?;
    assert_eq!(events.next_projection(&mut context, &store)?, Some(watch));
    assert_eq!(events.next_projection(&mut context, &store)?, None);
    assert_eq!(events.next_owned(&mut context, owner)?, None);
    assert_eq!(events.next_owned(&mut context, foreign)?, Some(1));
    events.complete(&mut context, 1)?;
    assert_eq!(events.next_goal(&mut context)?, None);
    context.equate(&store, first, unit)?;
    assert_eq!(events.next_projection(&mut context, &store)?, None);
    assert_eq!(events.next_owned(&mut context, owner)?, Some(0));
    events.complete(&mut context, 0)?;
    assert!(!events.waiting());
    Ok(())
}
#[test]
fn completed_prefix_seed_without_watch_cannot_leave_an_active_owner() -> TestResult {
    use crate::syntax::ast::NodeId;
    let mut events = Notifications::default();
    let mut context = Context::default();
    events.own(NodeId::new(1), &[0])?;
    assert!(events.own(NodeId::new(2), &[0]).is_err());
    events.complete(&mut context, 0)?;
    assert_eq!(events.next_owned(&mut context, NodeId::new(1))?, None);
    events.own(NodeId::new(2), &[0])?;
    assert_eq!(events.next_owned(&mut context, NodeId::new(2))?, Some(0));
    Ok(())
}

#[test]
fn selected_float_hints_are_silent_and_complete_one_real_frontier() -> TestResult {
    use crate::types::{Primitive, infer::Constructor};
    for cancel in [false, true] {
        let mut store = Store::default();
        let mut context = Context::default();
        let mut events = Notifications::default();
        let first = context.variable(Domain::Float)?;
        let second = context.variable(Domain::Float)?;
        let inputs = context.construct(Constructor::Tuple, &[first, second])?;
        let predicate = goal(&mut context, &mut store, inputs)?;
        events.observe(&mut context, &store, 0, &predicate, None)?;
        events.prefer_floats(&mut context, &store, 0, &[first, second])?;
        assert_eq!(events.next_projection(&mut context, &store)?, None);
        assert_eq!(events.next_goal(&mut context)?, None);
        if cancel {
            events.clear_hints(&mut context, 0)?;
        }
        assert!(context.default_numbers()?);
        let expected = TypeId::primitive(if cancel {
            Primitive::F64
        } else {
            Primitive::F32
        });
        assert_eq!(context.resolved(&mut store, first)?, expected);
        assert_eq!(context.resolved(&mut store, second)?, expected);
        assert_eq!(events.next_projection(&mut context, &store)?, None);
        assert_eq!(events.next_goal(&mut context)?, Some(0));
        assert_eq!(events.next_goal(&mut context)?, None);
        events.complete(&mut context, 0)?;
        assert!(!events.waiting());
    }
    Ok(())
}

#[test]
fn replacement_and_completion_remove_every_selected_child_hint() -> TestResult {
    use crate::types::Primitive;
    let mut store = Store::default();
    let mut context = Context::default();
    let mut events = Notifications::default();
    let input = context.variable(Domain::Float)?;
    let predicate = goal(&mut context, &mut store, input)?;
    events.observe(&mut context, &store, 0, &predicate, None)?;
    events.prefer_floats(&mut context, &store, 0, &[input])?;
    events.prefer_floats(&mut context, &store, 0, &[])?;
    assert!(context.default_numbers()?);
    assert_eq!(
        context.resolved(&mut store, input)?,
        TypeId::primitive(Primitive::F64)
    );
    assert_eq!(events.next_projection(&mut context, &store)?, None);
    events.complete(&mut context, 0)?;
    let input = context.variable(Domain::Float)?;
    let predicate = goal(&mut context, &mut store, input)?;
    events.observe(&mut context, &store, 1, &predicate, None)?;
    events.prefer_floats(&mut context, &store, 1, &[input])?;
    events.complete(&mut context, 1)?;
    assert!(context.default_numbers()?);
    assert_eq!(
        context.resolved(&mut store, input)?,
        TypeId::primitive(Primitive::F64)
    );
    assert_eq!(events.next_projection(&mut context, &store)?, None);
    assert!(!events.waiting());
    Ok(())
}

#[test]
fn projection_parent_hints_preserve_the_original_projection_ticket() -> TestResult {
    use crate::types::Primitive;
    let mut store = Store::default();
    let mut context = Context::default();
    let mut events = Notifications::default();
    let input = context.variable(Domain::Float)?;
    let watch = context.watch_normalization(input)?;
    events.prefer_projection_floats(&mut context, &store, watch, &[input])?;
    assert_eq!(events.next_projection(&mut context, &store)?, None);
    assert!(context.default_numbers()?);
    assert_eq!(
        context.resolved(&mut store, input)?,
        TypeId::primitive(Primitive::F32)
    );
    assert_eq!(events.next_projection(&mut context, &store)?, Some(watch));
    assert_eq!(events.next_projection(&mut context, &store)?, None);
    events.clear_projection_hints(&mut context, watch)?;
    context.cancel_watch(watch)?;
    assert!(!events.waiting());
    Ok(())
}

#[test]
fn selected_child_fanouts_default_in_one_linear_parent_frontier() -> TestResult {
    use crate::types::{Primitive, infer::Constructor};
    for count in [8, 32, 128] {
        let mut store = Store::default();
        let mut context = Context::default();
        let mut events = Notifications::default();
        let inputs = (0..count)
            .map(|_| context.variable(Domain::Float))
            .collect::<Result<Vec<_>, _>>()?;
        let tuple = context.construct(Constructor::Tuple, &inputs)?;
        let predicate = goal(&mut context, &mut store, tuple)?;
        events.observe(&mut context, &store, 0, &predicate, None)?;
        events.prefer_floats(&mut context, &store, 0, &inputs)?;
        assert!(context.default_numbers()?);
        for input in inputs {
            assert_eq!(
                context.resolved(&mut store, input)?,
                TypeId::primitive(Primitive::F32)
            );
        }
        assert_eq!(events.next_projection(&mut context, &store)?, None);
        assert_eq!(events.next_goal(&mut context)?, Some(0));
        assert_eq!(events.next_goal(&mut context)?, None);
        events.complete(&mut context, 0)?;
        eprintln!("selected child fanout={count} work={}", context.steps);
        assert!(context.steps <= 192 * count + 512);
    }
    Ok(())
}
