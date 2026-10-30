//! Numeric preferences share changed-root waiters without a second root cache.
use super::*;
use crate::types::{
    ItemId, Primitive, TypeId,
    infer::{Domain, arguments::Argument, predicates::Predicate},
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
