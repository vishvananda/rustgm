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
    events.observe(&mut context, &store, 0, &a)?;
    events.observe(&mut context, &store, 1, &b)?;
    let unit = context.known(&store, TypeId::UNIT)?;
    context.equate(&store, projection, unit)?;
    context.equate(&store, second, unit)?;
    events.observe(&mut context, &store, 0, &a)?;
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
            events.observe(&mut context, &store, index, &predicate)?;
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
    events.observe(&mut context, &store, 0, &first_goal)?;
    events.observe(&mut context, &store, 1, &second_goal)?;
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
