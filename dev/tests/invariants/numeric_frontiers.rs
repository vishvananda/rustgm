//! Numeric seeds respect real normalization barriers and shared graph occurrences.
use super::*;
use crate::types::infer::Value;
use crate::types::{
    ItemId, Kind, Primitive, TypeId, copy,
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
        let seen = &context.dependencies.as_ref().ok_or("owner")?.watches[owner.0 as usize].seen;
        println!(
            "numeric live width={width} seen_entries={} pair_capacity={} pair_bytes={}",
            seen.len(),
            seen.capacity(),
            crate::support::retained::map(seen)
        );
        assert!(!context.default_numbers()?);
        assert_eq!(context.unresolved_domain(numeric)?, Some(Domain::Integer));
        let known = context.known(&store, TypeId::primitive(Primitive::Usize))?;
        context.publish_projection(&store, output, known)?;
        drain(&mut context, &store)?;
        assert!(context.default_numbers()?);
        drain(&mut context, &store)?;
        assert_eq!(
            context.resolved(&mut store, numeric)?,
            TypeId::primitive(Primitive::I32)
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
    ready_equations_precede_numeric_fallback()?;
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

#[test]
fn child_hints_cannot_cross_a_parent_projection_barrier() -> TestResult {
    let mut context = Context::default();
    let mut store = Store::default();
    let input = context.variable(Domain::Float)?;
    let output = projection(&mut context, &mut store, input)?;
    let parent = context.watch_inputs(&[output], &[], true, Fallback::Predicate)?;
    let producer = context.watch_normalization(output)?;
    drain(&mut context, &store)?;
    assert_eq!(context.watch_float_hint(parent, input)?, None);
    assert!(context.default_numbers()?);
    assert_eq!(
        context.resolved(&mut store, input)?,
        TypeId::primitive(Primitive::F64)
    );
    context.cancel_watch(producer)?;
    context.cancel_watch(parent)?;
    Ok(())
}

fn ready_equations_precede_numeric_fallback() -> TestResult {
    use crate::types::infer::deferred::{Outcome, Relation};
    for watched in [false, true] {
        let mut context = Context::default();
        let mut store = crate::types::Store::default();
        let subject = context.known(&store, TypeId::primitive(Primitive::Usize))?;
        let projected = projection(&mut context, &mut store, subject)?;
        let expected = context.known(&store, TypeId::primitive(Primitive::I32))?;
        let Outcome::Pending(equation) =
            context.equate_deferred(&store, projected, expected, Relation::Equality)?
        else {
            return Err("actual projection equation did not suspend".into());
        };
        let numeric = context.variable(Domain::Integer)?;
        if watched {
            context.watch_inputs(&[numeric], &[], true, Fallback::Predicate)?;
        }
        context.apply_normalization(&mut store, projected, TypeId::primitive(Primitive::I32))?;
        drain(&mut context, &store)?;
        assert!(!context.default_numbers()?);
        assert_eq!(context.unresolved_domain(numeric)?, Some(Domain::Integer));
        assert_eq!(context.ready_equation()?, Some(equation));
        assert!(matches!(
            context.resume_equation(&store, equation)?,
            Outcome::Complete
        ));
        assert!(context.ready_equation()?.is_none());
        assert!(!context.has_deferred_equations());
        assert!(context.default_numbers()?);
        drain(&mut context, &store)?;
        assert_eq!(
            context.resolved(&mut store, numeric)?,
            TypeId::primitive(Primitive::I32)
        );
    }
    Ok(())
}

#[test]
fn scoped_integer_inputs_wait_without_losing_independent_equations() -> TestResult {
    for reverse in [false, true] {
        let mut context = Context::default();
        let mut store = Store::default();
        let subject_type = TypeId::primitive(Primitive::U8);
        let subject = context.known(&store, subject_type)?;
        let projected = projection(&mut context, &mut store, subject)?;
        let projected_type = context.resolved(&mut store, projected)?;
        let Some(Kind::Projection(application)) = store.kind(projected_type) else {
            return Err("named associated projection expected".into());
        };
        assert_eq!(application.item, ItemId(10));
        assert_eq!(application.subject, subject_type);
        assert_eq!(
            store
                .trait_application(application.trait_application)
                .ok_or("projection trait application")?
                .item,
            ItemId(1)
        );
        assert_eq!(context.head(&store, projected), Err(Error::Normalization));
        let projected_root = context.root(projected)?;
        assert_eq!(context.opaque_projection(projected_root), None);

        let integer = context.variable(Domain::Integer)?;
        let independent = context.variable(Domain::Any)?;
        let boolean = context.known(&store, TypeId::primitive(Primitive::Bool))?;
        let word_type = TypeId::primitive(Primitive::U32);
        let word = context.known(&store, word_type)?;
        assert_eq!(context.unresolved_domain(integer)?, Some(Domain::Integer));
        assert_eq!(context.head(&store, integer)?, None);
        let left = context.construct(Constructor::Tuple, &[integer, independent, integer])?;
        let right = context.construct(Constructor::Tuple, &[projected, boolean, word])?;
        let (left, right) = if reverse {
            (right, left)
        } else {
            (left, right)
        };

        let result = context.equate_scoped_inputs(&mut store, left, right);
        assert_eq!(result, Err(Error::Normalization), "reverse={reverse}");
        assert_eq!(
            context.resolved(&mut store, independent)?,
            TypeId::primitive(Primitive::Bool)
        );
        assert_eq!(context.resolved(&mut store, integer)?, word_type);
        assert_ne!(context.root(integer)?, context.root(projected)?);
        assert_eq!(context.resolved(&mut store, projected)?, projected_type);
        assert_eq!(context.head(&store, projected), Err(Error::Normalization));

        context.apply_normalization(&mut store, projected, word_type)?;
        context.equate_scoped_inputs(&mut store, left, right)?;
        assert_eq!(
            context.resolved(&mut store, left)?,
            context.resolved(&mut store, right)?
        );
    }
    Ok(())
}

#[test]
fn scoped_integer_inputs_preserve_closed_mismatches_and_ordinary_equations() -> TestResult {
    for reverse in [false, true] {
        for primitive in [Primitive::Bool, Primitive::F32] {
            let mut context = Context::default();
            let mut store = Store::default();
            let integer = context.variable(Domain::Integer)?;
            let other = context.known(&store, TypeId::primitive(primitive))?;
            let (left, right) = if reverse {
                (other, integer)
            } else {
                (integer, other)
            };
            assert_eq!(
                context.equate_scoped_inputs(&mut store, left, right),
                Err(Error::Mismatch)
            );
            assert_eq!(context.unresolved_domain(integer)?, Some(Domain::Integer));
            assert_ne!(context.root(integer)?, context.root(other)?);
        }
        let mut context = Context::default();
        let mut store = Store::default();
        let subject = context.known(&store, TypeId::primitive(Primitive::U8))?;
        let projected = projection(&mut context, &mut store, subject)?;
        let integer = context.variable(Domain::Integer)?;
        let (left, right) = if reverse {
            (projected, integer)
        } else {
            (integer, projected)
        };
        assert_eq!(
            context.equate_scoped_terms(&store, left, right),
            Err(Error::Mismatch)
        );
        assert_eq!(context.unresolved_domain(integer)?, Some(Domain::Integer));
        assert_ne!(context.root(integer)?, context.root(projected)?);
    }
    Ok(())
}

#[test]
fn scoped_integer_inputs_cannot_bind_a_marked_opaque_projection() -> TestResult {
    for reverse in [false, true] {
        let mut context = Context::default();
        let mut store = Store::default();
        let subject = context.known(&store, TypeId::primitive(Primitive::U8))?;
        let projected = projection(&mut context, &mut store, subject)?;
        let projected_type = context.resolved(&mut store, projected)?;
        context.apply_normalization(&mut store, projected, projected_type)?;
        let projected_root = context.root(projected)?;
        assert_eq!(
            context.opaque_projection(projected_root),
            Some(projected_type)
        );
        assert!(matches!(
            context.head(&store, projected)?,
            Some(Constructor::Projection(_))
        ));
        let integer = context.variable(Domain::Integer)?;
        let (left, right) = if reverse {
            (projected, integer)
        } else {
            (integer, projected)
        };
        assert_eq!(
            context.equate_scoped_inputs(&mut store, left, right),
            Err(Error::Mismatch)
        );
        assert_eq!(context.unresolved_domain(integer)?, Some(Domain::Integer));
        assert_ne!(context.root(integer)?, context.root(projected)?);
        assert_eq!(context.resolved(&mut store, projected)?, projected_type);
    }
    Ok(())
}
