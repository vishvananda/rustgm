//! Numeric seeds respect real normalization barriers and shared graph occurrences.
use super::*;
use crate::types::infer::Value;
use crate::types::{
    ItemId, Primitive, TypeId, copy,
    infer::{Constructor, Domain, arguments::ArgumentsId, projections::Projection},
    projection::Schema,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn projection(
    context: &mut Context,
    store: &mut crate::types::Store,
    subject: TermId,
) -> TestResult<TermId> {
    store.declare_projection(
        ItemId(10),
        Schema {
            trait_item: ItemId(1),
            sized: true,
            copies: copy::Properties::default(),
        },
    )?;
    let arguments: ArgumentsId = context.argument_layout(&[], &mut Vec::new())?;
    Ok(context.construct(
        Constructor::Projection(Projection {
            item: ItemId(10),
            arguments,
            trait_arguments: 0,
        }),
        &[subject],
    )?)
}
fn drain(context: &mut Context, store: &crate::types::Store) -> TestResult {
    while context.ready_resolution(store)?.is_some() {}
    Ok(())
}
#[test]
fn producer_defaulting_yields_before_blocked_consumers_and_reversed_roots() -> TestResult {
    for reverse in [false, true] {
        let mut context = Context::default();
        let mut store = crate::types::Store::default();
        let outer = context.variable(Domain::Integer)?;
        let inner = context.variable(Domain::Integer)?;
        let output = projection(&mut context, &mut store, inner)?;
        let roots = if reverse {
            [outer, output]
        } else {
            [output, outer]
        };
        let consumer = context.watch_inputs(&roots, &[], true, Fallback::Predicate)?;
        let producer = context.watch_normalization(output)?;
        drain(&mut context, &store)?;
        assert!(context.default_numbers()?);
        assert_eq!(
            context.resolved(&mut store, inner)?,
            TypeId::primitive(Primitive::I32)
        );
        assert_eq!(context.unresolved_domain(outer)?, Some(Domain::Integer));
        drain(&mut context, &store)?;
        context.apply_normalization(&mut store, output, TypeId::primitive(Primitive::Usize))?;
        let expected = context.known(&store, TypeId::primitive(Primitive::Usize))?;
        context.equate(&store, outer, expected)?;
        drain(&mut context, &store)?;
        context.cancel_watch(producer)?;
        context.cancel_watch(consumer)?;
        assert!(!context.default_numbers()?);
    }
    Ok(())
}
#[test]
fn hidden_first_diamonds_upgrade_once_and_shared_blockers_remain_unresolved() -> TestResult {
    for width in [1, 8, 32, 128] {
        let mut context = Context::default();
        let mut store = crate::types::Store::default();
        let numeric = context.variable(Domain::Integer)?;
        let alias = context.variable(Domain::Any)?;
        let output = projection(&mut context, &mut store, numeric)?;
        let diamond = context.construct(Constructor::Tuple, &vec![output; width])?;
        let owner = context.watch_inputs(&[diamond, alias], &[], true, Fallback::Predicate)?;
        let producer = context.watch_normalization(output)?;
        drain(&mut context, &store)?;
        context.equate(&store, alias, numeric)?;
        drain(&mut context, &store)?;
        let watch = &context.dependencies.as_ref().ok_or("owner")?.watches[owner.0 as usize];
        assert_eq!(watch.numeric, 1);
        assert_eq!(watch.blockers, 1);
        assert!(!context.default_numbers()?);
        assert_eq!(context.unresolved_domain(numeric)?, Some(Domain::Integer));
        context.apply_normalization(&mut store, output, TypeId::primitive(Primitive::Usize))?;
        drain(&mut context, &store)?;
        assert!(context.default_numbers()?);
        drain(&mut context, &store)?;
        assert_eq!(
            context.resolved(&mut store, numeric)?,
            TypeId::primitive(Primitive::I32)
        );
        let seen = &context.dependencies.as_ref().ok_or("owner")?.watches[owner.0 as usize].seen;
        println!(
            "numeric live width={width} seen_entries={} pair_capacity={} pair_bytes={}",
            seen.len(),
            seen.capacity(),
            crate::support::retained::map(seen)
        );
        context.cancel_watch(producer)?;
        context.cancel_watch(owner)?;
        let storage = context.dependencies.as_ref().ok_or("owner")?;
        println!(
            "numeric width={width} work={} Watch={} Storage={} VisitPair={} OldVisit={} QueueEntry={} watch_capacity={} wait_capacity={} numeric_capacity={} retained={:?}",
            context.steps,
            size_of::<Watch>(),
            size_of::<Storage>(),
            size_of::<(Dependency, Visit)>(),
            size_of::<Dependency>(),
            size_of::<(WatchId, Dependency, bool)>(),
            storage.watches.capacity(),
            storage.waiting.capacity(),
            storage.numeric_ready.capacity(),
            context.retained_allocation()
        );
        assert!(context.steps < 1500 + width * 100);
    }
    Ok(())
}
#[test]
fn ordinary_watches_and_failed_numeric_growth_keep_existing_authority() -> TestResult {
    let mut context = Context::default();
    let store = crate::types::Store::default();
    assert!(context.dependencies.is_none());
    let ordinary = context.variable(Domain::Integer)?;
    let watch = context.watch_resolution(&[ordinary], &[])?;
    drain(&mut context, &store)?;
    assert!(context.default_numbers()?);
    drain(&mut context, &store)?;
    assert!(context.watch_resolved(watch)?);
    context.cancel_watch(watch)?;
    let pending = context.variable(Domain::Integer)?;
    context.watch_inputs(&[pending], &[], true, Fallback::Predicate)?;
    drain(&mut context, &store)?;
    context.steps = 1_000_000;
    assert_eq!(context.default_numbers(), Err(Error::Budget));
    assert!(matches!(
        context.terms[pending.0 as usize].value,
        Value::Variable(Domain::Integer)
    ));
    let first = context.steps;
    assert_eq!(context.default_numbers(), Err(Error::Budget));
    assert!(context.steps > first);
    Ok(())
}

#[test]
fn final_publication_rejects_only_live_numeric_frontiers() -> TestResult {
    for mode in ["pending", "settled", "cancelled", "ordinary"] {
        let mut context = Context::default();
        let mut store = crate::types::Store::default();
        let numeric = context.variable(Domain::Integer)?;
        if mode != "ordinary" {
            let output = projection(&mut context, &mut store, numeric)?;
            let watch = context.watch_inputs(&[output, numeric], &[], true, Fallback::Predicate)?;
            drain(&mut context, &store)?;
            if mode == "cancelled" {
                context.cancel_watch(watch)?;
            } else if mode == "settled" {
                let ty = TypeId::primitive(Primitive::I32);
                let value = context.known(&store, ty)?;
                context.equate(&store, numeric, value)?;
                context.apply_normalization(&mut store, output, ty)?;
                drain(&mut context, &store)?;
            }
        }
        let result = context.finish(&mut store);
        if mode == "pending" {
            assert!(matches!(result, Err(Error::Normalization)));
        } else {
            assert!(result.is_ok(), "{mode}: {:?}", result.err());
        }
    }
    Ok(())
}
