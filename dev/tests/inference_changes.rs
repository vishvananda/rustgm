//! Selection retries follow input changes, never unrelated body equations.
use rustgm::types::{
    Primitive, Store, TypeId,
    infer::{Constructor, Context, Domain, Error, dependencies::WatchId},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);

fn drain(context: &mut Context, store: &Store) -> Result<Vec<WatchId>, Error> {
    let mut events = Vec::new();
    while let Some(id) = context.ready_resolution(store)? {
        events.push(id);
    }
    Ok(events)
}

#[test]
fn published_normal_forms_retry_queries_without_completing_their_input_tickets() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let input = context.variable(Domain::Any)?;
    let watch = context.watch_changes(&[input], &[])?;
    assert_eq!(drain(&mut context, &store)?, [watch]);
    context.await_resolution(watch)?;
    context.retry_watch(watch)?;
    context.retry_watch(watch)?;
    assert_eq!(drain(&mut context, &store)?, [watch]);
    assert!(!context.watch_resolved(watch)?);
    let byte = context.known(&store, BYTE)?;
    context.equate(&store, input, byte)?;
    assert_eq!(drain(&mut context, &store)?, [watch]);
    assert!(context.watch_resolved(watch)?);
    context.retry_watch(watch)?;
    assert!(drain(&mut context, &store)?.is_empty());
    context.cancel_watch(watch)?;
    context.retry_watch(watch)?;
    assert!(drain(&mut context, &store)?.is_empty());
    Ok(())
}

#[test]
fn initial_partial_event_is_once_per_query_and_updates_coalesce() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let a = context.variable(Domain::Any)?;
    let b = context.variable(Domain::Any)?;
    let watch = context.watch_changes(&[a, b, a], &[])?;
    assert_eq!(drain(&mut context, &store)?, [watch]);
    assert!(!context.watch_resolved(watch)?);
    let byte = context.known(&store, BYTE)?;
    context.equate(&store, a, byte)?;
    context.equate(&store, b, byte)?;
    assert_eq!(context.ready_resolution(&store)?, Some(watch));
    // The notification includes all queued updates and already knows readiness.
    assert!(context.watch_resolved(watch)?);
    assert!(drain(&mut context, &store)?.is_empty());
    Ok(())
}

#[test]
fn canonical_completion_order_precedes_queued_partial_notifications() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let a = context.variable(Domain::Any)?;
    let b = context.variable(Domain::Any)?;
    let mixed = context.watch_changes(&[a, b], &[])?;
    let first = context.watch_resolution(&[a], &[])?;
    assert_eq!(drain(&mut context, &store)?, [mixed]);
    let byte = context.known(&store, BYTE)?;
    context.equate(&store, a, byte)?;
    context.equate(&store, b, byte)?;
    assert_eq!(context.ready_resolution(&store)?, Some(first));
    assert_eq!(context.ready_resolution(&store)?, Some(mixed));
    assert!(context.watch_resolved(mixed)?);
    assert!(drain(&mut context, &store)?.is_empty());
    Ok(())
}

#[test]
fn domain_narrowing_and_root_merges_notify_without_defaulting() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let a = context.variable(Domain::Any)?;
    let b = context.variable(Domain::Integer)?;
    let watch = context.watch_changes(&[a, b], &[])?;
    assert_eq!(drain(&mut context, &store)?, [watch]);
    context.equate(&store, a, b)?;
    assert_eq!(drain(&mut context, &store)?, [watch]);
    assert!(!context.watch_resolved(watch)?);
    let c = context.variable(Domain::Integer)?;
    context.equate(&store, c, a)?;
    assert_eq!(drain(&mut context, &store)?, [watch]);
    let byte = context.known(&store, BYTE)?;
    context.equate(&store, b, byte)?;
    assert_eq!(context.ready_resolution(&store)?, Some(watch));
    context.cancel_watch(watch)?;
    assert!(drain(&mut context, &store)?.is_empty());
    assert_eq!(context.watch_resolved(watch), Err(Error::InvalidIdentity));
    Ok(())
}

#[test]
fn new_compound_children_remain_watched_with_linear_graph_work() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let mut leaf = context.variable(Domain::Any)?;
    let watch = context.watch_changes(&[leaf], &[])?;
    assert_eq!(drain(&mut context, &store)?, [watch]);
    let start = context.steps;
    for _ in 0..512 {
        let child = context.variable(Domain::Any)?;
        let tuple = context.construct(Constructor::Tuple, &[child, child])?;
        context.equate(&store, leaf, tuple)?;
        assert_eq!(drain(&mut context, &store)?, [watch]);
        assert!(!context.watch_resolved(watch)?);
        leaf = child;
    }
    assert!(context.steps - start < 512 * 24);
    let byte = context.known(&store, BYTE)?;
    context.equate(&store, leaf, byte)?;
    drain(&mut context, &store)?;
    assert!(context.watch_resolved(watch)?);
    Ok(())
}

#[test]
fn changes_before_leaf_discovery_are_seen_and_cancelled_queries_stay_inert() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let a = context.variable(Domain::Any)?;
    let b = context.variable(Domain::Any)?;
    let count = context.const_variable(&store, TypeId::primitive(Primitive::Usize))?;
    let watch = context.watch_changes(&[a, b], &[count])?;
    let byte = context.known(&store, BYTE)?;
    context.equate(&store, b, byte)?;
    let array = context.construct(Constructor::ArrayInfer(count), &[byte])?;
    let fixed = context.construct(Constructor::Array(3), &[byte])?;
    context.equate(&store, array, fixed)?;
    assert_eq!(context.ready_resolution(&store)?, Some(watch));
    assert!(!context.watch_resolved(watch)?);
    assert!(drain(&mut context, &store)?.is_empty());
    context.equate(&store, a, byte)?;
    drain(&mut context, &store)?;
    assert!(context.watch_resolved(watch)?);
    let cancelled = context.watch_changes(&[array], &[])?;
    context.cancel_watch(cancelled)?;
    assert!(drain(&mut context, &store)?.is_empty());
    let empty = context.watch_changes(&[], &[])?;
    context.cancel_watch(empty)?;
    assert!(drain(&mut context, &store)?.is_empty());
    Ok(())
}

#[test]
fn const_leaves_notify_and_unrelated_queries_do_not_rescan() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let mut inputs = Vec::new();
    for _ in 0..4_096 {
        let constant = context.const_variable(&store, TypeId::primitive(Primitive::Usize))?;
        inputs.push((constant, context.watch_changes(&[], &[constant])?));
    }
    assert_eq!(drain(&mut context, &store)?.len(), inputs.len());
    let byte = context.known(&store, BYTE)?;
    for index in [3_000, 1, 2_000, 4_095] {
        let start = context.steps;
        let (constant, watch) = inputs[index];
        let array = context.construct(Constructor::ArrayInfer(constant), &[byte])?;
        let fixed = context.construct(Constructor::Array(3), &[byte])?;
        context.equate(&store, array, fixed)?;
        assert_eq!(context.ready_resolution(&store)?, Some(watch));
        context.cancel_watch(watch)?;
        assert!(drain(&mut context, &store)?.is_empty());
        assert!(context.steps - start < 40);
    }
    Ok(())
}

#[test]
fn certified_queries_can_wait_for_resolution_without_repeated_partial_events() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let inputs = (0..512)
        .map(|_| context.variable(Domain::Any))
        .collect::<Result<Vec<_>, _>>()?;
    let watch = context.watch_changes(&inputs, &[])?;
    assert_eq!(context.await_resolution(watch), Err(Error::InvalidIdentity));
    assert_eq!(context.ready_resolution(&store)?, Some(watch));
    context.await_resolution(watch)?;
    assert!(drain(&mut context, &store)?.is_empty());
    let byte = context.known(&store, BYTE)?;
    let start = context.steps;
    for (index, input) in inputs.into_iter().enumerate() {
        context.equate(&store, input, byte)?;
        assert_eq!(
            context.ready_resolution(&store)?,
            (index == 511).then_some(watch)
        );
    }
    assert!(context.steps - start < 512 * 12);
    assert!(context.watch_resolved(watch)?);
    assert!(drain(&mut context, &store)?.is_empty());
    Ok(())
}
