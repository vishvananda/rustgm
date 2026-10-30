//! Numeric preferences share changed-root waiters without a second root cache.
use super::*;
use crate::types::{
    ItemId, Primitive, TypeId,
    infer::{Domain, Value, arguments::Argument, predicates::Predicate},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn watch(context: &mut Context, store: &Store, input: TermId) -> TestResult<WatchId> {
    let subject = context.known(store, TypeId::primitive(Primitive::F32))?;
    let goal = Predicate::trait_application(subject, ItemId(99), vec![Argument::Type(input)]);
    Ok(context.watch_predicate(&goal)?)
}
fn drain(context: &mut Context, store: &Store) -> TestResult {
    while context.ready_resolution(store)?.is_some() {}
    Ok(())
}
#[test]
fn shared_root_preferences_preserve_order_explicit_types_and_cancellation() -> TestResult {
    for reverse in [false, true] {
        for cancel in [false, true] {
            let mut context = Context::default();
            let mut store = Store::default();
            let input = context.variable(Domain::Float)?;
            let first = watch(&mut context, &store, input)?;
            let second = watch(&mut context, &store, input)?;
            let selected = if reverse { first } else { second };
            context.prefer_f32(selected, input)?;
            drain(&mut context, &store)?;
            if cancel {
                context.cancel_watch(selected)?;
            }
            assert!(context.default_numbers()?);
            assert_eq!(
                context.resolved(&mut store, input)?,
                TypeId::primitive(if cancel {
                    Primitive::F64
                } else {
                    Primitive::F32
                })
            );
        }
    }
    let mut context = Context::default();
    let mut store = Store::default();
    let input = context.variable(Domain::Float)?;
    let id = watch(&mut context, &store, input)?;
    context.prefer_f32(id, input)?;
    let explicit = context.known(&store, TypeId::primitive(Primitive::F64))?;
    context.equate(&store, input, explicit)?;
    drain(&mut context, &store)?;
    assert!(!context.default_numbers()?);
    assert_eq!(
        context.resolved(&mut store, input)?,
        TypeId::primitive(Primitive::F64)
    );
    Ok(())
}
#[test]
fn late_alias_preferences_do_not_change_independent_float_roots() -> TestResult {
    let mut context = Context::default();
    let mut store = Store::default();
    let input = context.variable(Domain::Float)?;
    let alias = context.variable(Domain::Any)?;
    let other = context.variable(Domain::Float)?;
    let id = watch(&mut context, &store, alias)?;
    watch(&mut context, &store, other)?;
    drain(&mut context, &store)?;
    context.equate(&store, alias, input)?;
    context.prefer_f32(id, alias)?;
    drain(&mut context, &store)?;
    while context.default_numbers()? {
        drain(&mut context, &store)?;
    }
    assert_eq!(
        context.resolved(&mut store, input)?,
        TypeId::primitive(Primitive::F32)
    );
    assert_eq!(
        context.resolved(&mut store, other)?,
        TypeId::primitive(Primitive::F64)
    );
    Ok(())
}
#[test]
fn failed_preference_payment_and_cancelled_watches_publish_nothing() -> TestResult {
    let mut context = Context::default();
    let store = Store::default();
    let input = context.variable(Domain::Float)?;
    let id = watch(&mut context, &store, input)?;
    context.steps = 1_000_000;
    assert_eq!(context.prefer_f32(id, input), Err(Error::Budget));
    assert!(matches!(
        context.dependencies.as_ref().ok_or("dependencies")?.watches[id.0 as usize].fallback,
        Fallback::Predicate
    ));
    let mut context = Context::default();
    let input = context.variable(Domain::Float)?;
    let id = watch(&mut context, &store, input)?;
    context.cancel_watch(id)?;
    assert_eq!(context.prefer_f32(id, input), Err(Error::InvalidIdentity));
    Ok(())
}
#[test]
fn shared_fanout_registration_and_binding_keep_linear_work_and_compact_payloads() -> TestResult {
    assert_eq!(size_of::<Fallback>(), 8);
    for count in [8, 32, 128] {
        let mut context = Context::default();
        let mut store = Store::default();
        let input = context.variable(Domain::Float)?;
        for index in 0..count {
            let id = watch(&mut context, &store, input)?;
            if index + 1 == count {
                context.prefer_f32(id, input)?;
            }
            drain(&mut context, &store)?;
        }
        assert!(context.default_numbers()?);
        drain(&mut context, &store)?;
        assert_eq!(
            context.resolved(&mut store, input)?,
            TypeId::primitive(Primitive::F32)
        );
        eprintln!(
            "float fanout={count} work={} Watch={} Fallback={}",
            context.steps,
            size_of::<Watch>(),
            size_of::<Fallback>()
        );
        assert!(
            context.steps <= 128 * count,
            "shared fanout {count}: {}",
            context.steps
        );
    }
    Ok(())
}

#[test]
fn shared_blocked_frontiers_are_linear_and_cancellation_releases_the_root() -> TestResult {
    for (mixed, reverse) in [(false, false), (true, false), (true, true)] {
        for count in [8, 32, 128] {
            let mut context = Context::default();
            let mut store = Store::default();
            let first = context.variable(Domain::Float)?;
            let second = context.variable(Domain::Float)?;
            let (input, ready) = if reverse {
                (second, first)
            } else {
                (first, second)
            };
            let unresolved = context.variable(Domain::Any)?;
            let blocker =
                context.watch_inputs(&[input, unresolved], &[], true, Fallback::Predicate)?;
            for index in 0..count {
                let id = if mixed {
                    context.watch_inputs(&[ready, input], &[], true, Fallback::Predicate)?
                } else {
                    watch(&mut context, &store, input)?
                };
                if index + 1 == count {
                    context.prefer_f32(id, input)?;
                }
            }
            drain(&mut context, &store)?;
            let before = context.steps;
            assert!(!context.default_numbers()?);
            let blocked_work = context.steps - before;
            assert_eq!(context.unresolved_domain(input)?, Some(Domain::Float));
            assert_eq!(context.unresolved_domain(ready)?, Some(Domain::Float));
            assert!(
                blocked_work <= 24 * count + 128,
                "blocked fanout {count}/{mixed}/{reverse}: {blocked_work}"
            );
            context.cancel_watch(blocker)?;
            drain(&mut context, &store)?;
            assert!(context.default_numbers()?);
            drain(&mut context, &store)?;
            assert_eq!(
                context.resolved(&mut store, input)?,
                TypeId::primitive(Primitive::F32)
            );
            if mixed {
                assert_eq!(
                    context.resolved(&mut store, ready)?,
                    TypeId::primitive(Primitive::F64)
                );
            }
            eprintln!(
                "blocked float fanout={count} mixed={mixed} reverse={reverse} sweep_work={blocked_work} total_work={}",
                context.steps
            );
            assert!(context.steps <= 192 * count + 512);
        }
    }
    Ok(())
}

#[test]
fn a_silent_hint_never_seeds_fallback_or_emits_a_foreign_notification() -> TestResult {
    let mut context = Context::default();
    let mut store = Store::default();
    let input = context.variable(Domain::Float)?;
    let parent = watch(&mut context, &store, input)?;
    drain(&mut context, &store)?;
    let hint = context
        .watch_float_hint(parent, input)?
        .ok_or("missing visible hint")?;
    assert_eq!(context.ready_resolution(&store)?, None);
    context.cancel_watch(parent)?;
    assert!(context.default_numbers()?);
    assert_eq!(
        context.resolved(&mut store, input)?,
        TypeId::primitive(Primitive::F64)
    );
    assert_eq!(context.ready_resolution(&store)?, None);
    context.cancel_watch(hint)?;
    Ok(())
}

#[test]
fn failed_hint_payment_leaves_the_parent_and_float_unchanged() -> TestResult {
    let mut context = Context::default();
    let store = Store::default();
    let input = context.variable(Domain::Float)?;
    let parent = watch(&mut context, &store, input)?;
    drain(&mut context, &store)?;
    let count = context
        .dependencies
        .as_ref()
        .ok_or("dependencies")?
        .watches
        .len();
    context.steps = 1_000_000;
    assert_eq!(context.watch_float_hint(parent, input), Err(Error::Budget));
    let storage = context.dependencies.as_ref().ok_or("dependencies")?;
    assert_eq!(storage.watches.len(), count);
    assert!(matches!(
        storage.watches[parent.0 as usize].fallback,
        Fallback::Predicate
    ));
    assert!(matches!(
        context.terms[input.0 as usize].value,
        Value::Variable(Domain::Float)
    ));
    Ok(())
}

#[test]
fn reobserved_ordinary_conditions_remove_the_old_direct_preference() -> TestResult {
    let mut context = Context::default();
    let mut store = Store::default();
    let input = context.variable(Domain::Float)?;
    let id = watch(&mut context, &store, input)?;
    context.prefer_f32(id, input)?;
    drain(&mut context, &store)?;
    context.clear_float_preference(id)?;
    assert!(context.default_numbers()?);
    assert_eq!(
        context.resolved(&mut store, input)?,
        TypeId::primitive(Primitive::F64)
    );
    Ok(())
}
