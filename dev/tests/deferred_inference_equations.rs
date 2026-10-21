//! Semantic normalization resumes result equations without inferring noninjective inputs.
use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    infer::{
        Constructor, Context, Domain, Error,
        deferred::{EquationId, Outcome, Relation},
    },
    projection::Schema,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
const WORD: TypeId = TypeId::primitive(Primitive::U16);
const ITEM: ItemId = ItemId(7);

fn store() -> Result<Store, &'static str> {
    let mut store = Store::default();
    store.declare_projection(
        ITEM,
        Schema {
            trait_item: ItemId(6),
            sized: true,
            copies: rustgm::types::copy::Properties::default(),
        },
    )?;
    Ok(store)
}
fn pending(outcome: Outcome) -> Result<EquationId, &'static str> {
    match outcome {
        Outcome::Pending(id) => Ok(id),
        Outcome::Complete => Err("equation completed before normalization"),
    }
}

#[test]
fn symbolic_results_refine_shared_inputs_without_erasing_the_original_application() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let input = context.variable(Domain::Any)?;
    let projection = context.projected(&store, ITEM, input, &[], &[])?;
    let byte = context.known(&store, BYTE)?;
    let equation =
        pending(context.equate_deferred(&store, projection, byte, Relation::Equality)?)?;
    let watch = context.watch_changes(&[projection], &[])?;
    assert_eq!(context.ready_resolution(&store)?, Some(watch));
    assert!(!context.watch_resolved(watch)?);
    let retained = context.publish_projection(&store, projection, input)?;
    assert_eq!(context.ready_equation()?, Some(equation));
    assert_eq!(
        context.resume_equation(&store, equation)?,
        Outcome::Complete
    );
    assert_eq!(context.ready_resolution(&store)?, Some(watch));
    assert!(context.watch_resolved(watch)?);
    assert_eq!(context.resolved(&mut store, projection)?, BYTE);
    let application = context.projection_application(&store, retained)?;
    assert_eq!(
        context.resolved(&mut store, application.predicate.subject())?,
        BYTE
    );
    assert!(matches!(
        context.head(&store, retained),
        Err(Error::Normalization)
    ));
    context.apply_normalization(&mut store, retained, BYTE)?;
    context.finish(&mut store)?;
    Ok(())
}

#[test]
fn erased_symbolic_results_keep_const_input_dependencies_and_reject_cycles() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let byte = context.known(&store, BYTE)?;
    let length = context.const_variable(&store, TypeId::primitive(Primitive::Usize))?;
    let projection = context.projected(
        &store,
        ITEM,
        byte,
        &[rustgm::types::infer::arguments::Argument::Const(length)],
        &[],
    )?;
    let recursive = context.construct(Constructor::Tuple, &[projection])?;
    assert_eq!(
        context.publish_projection(&store, projection, recursive),
        Err(Error::Occurs)
    );
    let watch = context.watch_changes(&[projection], &[])?;
    assert_eq!(context.ready_resolution(&store)?, Some(watch));
    let retained = context.publish_projection(&store, projection, byte)?;
    assert_eq!(context.resolved(&mut store, projection)?, BYTE);
    assert_eq!(
        context.resolved(&mut store, retained),
        Err(Error::Unresolved)
    );
    assert_eq!(context.ready_resolution(&store)?, None);
    assert!(!context.watch_resolved(watch)?);
    Ok(())
}

#[test]
fn ordinary_siblings_infer_inputs_independently_of_projection_order() -> TestResult {
    for reverse in [false, true] {
        let mut store = store()?;
        let mut context = Context::default();
        let input = context.variable(Domain::Any)?;
        let projection = context.projected(&store, ITEM, input, &[], &[])?;
        let byte = context.known(&store, BYTE)?;
        let word = context.known(&store, WORD)?;
        let (left, right) = if reverse {
            ([projection, input], [byte, word])
        } else {
            ([input, projection], [word, byte])
        };
        let left = context.construct(Constructor::Tuple, &left)?;
        let right = context.construct(Constructor::Tuple, &right)?;
        let id = pending(context.equate_deferred(&store, left, right, Relation::Equality)?)?;
        assert_eq!(context.equation_roots(id)?, (left, right));
        assert_eq!(context.resolved(&mut store, input)?, WORD);
        assert_eq!(context.ready_equation()?, None);
        assert_eq!(context.resume_equation(&store, id)?, Outcome::Pending(id));
        let normalized = context.resolved(&mut store, right)?;
        context.apply_normalization(&mut store, left, normalized)?;
        assert_eq!(context.ready_equation()?, Some(id));
        assert_eq!(context.resume_equation(&store, id)?, Outcome::Complete);
        assert!(!context.has_deferred_equations());
        assert_eq!(context.ready_equation()?, None);
        let solved = context.finish(&mut store)?;
        assert_eq!(solved.types[left.0 as usize], normalized);
    }
    Ok(())
}

#[test]
fn equal_associated_outputs_never_merge_distinct_inputs() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let t = context.variable(Domain::Any)?;
    let u = context.variable(Domain::Any)?;
    let first = context.projected(&store, ITEM, t, &[], &[])?;
    let second = context.projected(&store, ITEM, u, &[], &[])?;
    let id = pending(context.equate_deferred(&store, first, second, Relation::Shape)?)?;
    assert_ne!(context.root(t)?, context.root(u)?);
    assert_eq!(
        context.apply_normalization(&mut store, first, BYTE),
        Err(Error::Unresolved)
    );
    let byte = context.known(&store, BYTE)?;
    let word = context.known(&store, WORD)?;
    context.equate(&store, t, byte)?;
    context.equate(&store, u, word)?;
    context.apply_normalization(&mut store, first, BYTE)?;
    assert_eq!(context.ready_equation()?, Some(id));
    assert_eq!(context.resume_equation(&store, id)?, Outcome::Pending(id));
    context.apply_normalization(&mut store, second, BYTE)?;
    assert_eq!(context.ready_equation()?, Some(id));
    assert_eq!(context.resume_equation(&store, id)?, Outcome::Complete);
    assert_eq!(context.resolved(&mut store, t)?, BYTE);
    assert_eq!(context.resolved(&mut store, u)?, WORD);
    assert_eq!(context.ready_equation()?, None);
    Ok(())
}

#[test]
fn a_self_dependent_projection_equation_waits_without_an_ordinary_occurs_cycle() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let input = context.variable(Domain::Any)?;
    let projected = context.projected(&store, ITEM, input, &[], &[])?;
    let id = pending(context.equate_deferred(&store, input, projected, Relation::Equality)?)?;
    assert_eq!(context.unresolved_domain(input)?, Some(Domain::Any));
    let byte = context.known(&store, BYTE)?;
    context.equate(&store, input, byte)?;
    context.apply_normalization(&mut store, projected, BYTE)?;
    assert_eq!(context.ready_equation()?, Some(id));
    assert_eq!(context.resume_equation(&store, id)?, Outcome::Complete);
    context.finish(&mut store)?;
    Ok(())
}

#[test]
fn known_inputs_do_not_discharge_an_unresumed_equation_at_completion() -> TestResult {
    for normalize in [false, true] {
        let mut store = store()?;
        let mut context = Context::default();
        let byte = context.known(&store, BYTE)?;
        let projected = context.projected(&store, ITEM, byte, &[], &[])?;
        pending(context.equate_deferred(&store, projected, byte, Relation::Equality)?)?;
        if normalize {
            context.apply_normalization(&mut store, projected, BYTE)?;
        }
        assert!(matches!(
            context.finish(&mut store),
            Err(Error::Normalization)
        ));
    }
    Ok(())
}

#[test]
fn numeric_results_wait_for_normalization_before_fallback() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let byte = context.known(&store, BYTE)?;
    let projected = context.projected(&store, ITEM, byte, &[], &[])?;
    let number = context.variable(Domain::Integer)?;
    let id = pending(context.equate_deferred(&store, projected, number, Relation::Equality)?)?;
    assert_eq!(context.unresolved_domain(number)?, Some(Domain::Integer));
    context.apply_normalization(&mut store, projected, WORD)?;
    assert_eq!(context.ready_equation()?, Some(id));
    assert_eq!(context.resume_equation(&store, id)?, Outcome::Complete);
    assert_eq!(context.resolved(&mut store, number)?, WORD);
    Ok(())
}

#[test]
fn selected_results_must_satisfy_the_original_result_equation() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let byte = context.known(&store, BYTE)?;
    let projected = context.projected(&store, ITEM, byte, &[], &[])?;
    let id = pending(context.equate_deferred(&store, projected, byte, Relation::Equality)?)?;
    context.apply_normalization(&mut store, projected, WORD)?;
    assert_eq!(context.ready_equation()?, Some(id));
    assert_eq!(context.resume_equation(&store, id), Err(Error::Mismatch));
    Ok(())
}

#[test]
fn unchanged_siblings_do_not_repeat_region_equalities_on_resume() -> TestResult {
    for relation in [Relation::Equality, Relation::Shape] {
        let mut store = store()?;
        let mut context = Context::default();
        let byte = context.known(&store, BYTE)?;
        let projected = context.projected(&store, ITEM, byte, &[], &[])?;
        let a = context.construct(
            Constructor::Reference {
                mutable: false,
                region: RegionId(1),
            },
            &[byte],
        )?;
        let b = context.construct(
            Constructor::Reference {
                mutable: false,
                region: RegionId(2),
            },
            &[byte],
        )?;
        let left = context.construct(Constructor::Tuple, &[a, projected])?;
        let right = context.construct(Constructor::Tuple, &[b, byte])?;
        let id = pending(context.equate_deferred(&store, left, right, relation)?)?;
        let expected = if relation == Relation::Equality {
            vec![(RegionId(1), RegionId(2))]
        } else {
            vec![]
        };
        assert_eq!(context.region_equalities, expected);
        context.apply_normalization(&mut store, projected, BYTE)?;
        assert_eq!(context.ready_equation()?, Some(id));
        assert_eq!(context.resume_equation(&store, id)?, Outcome::Complete);
        assert_eq!(context.region_equalities, expected);
    }
    Ok(())
}

#[test]
fn a_selected_composite_result_exposes_remaining_projected_equations() -> TestResult {
    for second_first in [false, true] {
        let mut store = store()?;
        let mut context = Context::default();
        let byte = context.known(&store, BYTE)?;
        let first = context.projected(&store, ITEM, byte, &[], &[])?;
        let second = context.projected(&store, ITEM, byte, &[], &[])?;
        let target = context.construct(Constructor::Tuple, &[byte, second])?;
        let id = pending(context.equate_deferred(&store, first, target, Relation::Equality)?)?;
        if second_first {
            context.apply_normalization(&mut store, second, BYTE)?;
            assert_eq!(context.ready_equation()?, None);
        }
        let children = store.intern_list(&[BYTE, BYTE])?;
        let normal = store.intern(Kind::Tuple(children))?;
        context.apply_normalization(&mut store, first, normal)?;
        assert_eq!(context.ready_equation()?, Some(id));
        let outcome = context.resume_equation(&store, id)?;
        if second_first {
            assert_eq!(outcome, Outcome::Complete);
        } else {
            assert_eq!(outcome, Outcome::Pending(id));
            context.apply_normalization(&mut store, second, BYTE)?;
            assert_eq!(context.ready_equation()?, Some(id));
            assert_eq!(context.resume_equation(&store, id)?, Outcome::Complete);
        }
        context.finish(&mut store)?;
    }
    Ok(())
}

#[test]
fn normalization_cannot_rewrite_ordinary_lifetimes_or_fill_unknown_inputs() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let input = context.variable(Domain::Any)?;
    assert_eq!(
        context.apply_normalization(&mut store, input, BYTE),
        Err(Error::Unresolved)
    );
    let byte = context.known(&store, BYTE)?;
    let reference = context.construct(
        Constructor::Reference {
            mutable: false,
            region: RegionId(1),
        },
        &[byte],
    )?;
    let replacement = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId(2),
        referent: BYTE,
    })?;
    assert_eq!(
        context.apply_normalization(&mut store, reference, replacement),
        Err(Error::Mismatch)
    );
    assert!(context.region_equalities.is_empty());
    Ok(())
}

#[test]
fn resuming_a_wide_equation_visits_only_the_projection_that_changed() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let byte = context.known(&store, BYTE)?;
    let mut inputs = Vec::new();
    for _ in 0..4_096 {
        inputs.push(context.projected(&store, ITEM, byte, &[], &[])?);
    }
    let left = context.construct(Constructor::Tuple, &inputs)?;
    let right = context.construct(Constructor::Tuple, &vec![byte; inputs.len()])?;
    let id = pending(context.equate_deferred(&store, left, right, Relation::Equality)?)?;
    for (index, input) in inputs.iter().enumerate() {
        let before = context.steps;
        context.apply_normalization(&mut store, *input, BYTE)?;
        assert_eq!(context.ready_equation()?, Some(id));
        let expected = if index + 1 == inputs.len() {
            Outcome::Complete
        } else {
            Outcome::Pending(id)
        };
        assert_eq!(context.resume_equation(&store, id)?, expected);
        assert_eq!(context.ready_equation()?, None);
        assert!(context.steps - before < 64);
    }
    assert!(!context.has_deferred_equations());
    context.finish(&mut store)?;
    Ok(())
}

#[test]
fn selected_results_can_determine_inputs_of_a_later_normalization_query() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let byte = context.known(&store, BYTE)?;
    let argument = context.variable(Domain::Any)?;
    let output = context.variable(Domain::Integer)?;
    let first = context.projected(&store, ITEM, byte, &[], &[])?;
    let second = context.projected(&store, ITEM, argument, &[], &[])?;
    let a = pending(context.equate_deferred(&store, first, argument, Relation::Equality)?)?;
    let b = pending(context.equate_deferred(&store, second, output, Relation::Equality)?)?;
    let watch = context.watch_resolution(&[second], &[])?;
    assert_eq!(context.ready_resolution(&store)?, None);
    context.apply_normalization(&mut store, first, WORD)?;
    assert_eq!(context.ready_equation()?, Some(a));
    assert_eq!(context.resume_equation(&store, a)?, Outcome::Complete);
    assert_eq!(context.ready_resolution(&store)?, Some(watch));
    assert_eq!(context.resolved(&mut store, argument)?, WORD);
    context.apply_normalization(&mut store, second, BYTE)?;
    assert_eq!(context.ready_equation()?, Some(b));
    assert_eq!(context.resume_equation(&store, b)?, Outcome::Complete);
    assert_eq!(context.resolved(&mut store, output)?, BYTE);
    context.finish(&mut store)?;
    Ok(())
}

#[test]
fn simultaneous_normalization_coalesces_wakeups_without_dropping_leaves() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let byte = context.known(&store, BYTE)?;
    let first = context.projected(&store, ITEM, byte, &[], &[])?;
    let second = context.projected(&store, ITEM, byte, &[], &[])?;
    let left = context.construct(Constructor::Tuple, &[first, second, first])?;
    let right = context.construct(Constructor::Tuple, &[byte, byte, byte])?;
    let id = pending(context.equate_deferred(&store, left, right, Relation::Shape)?)?;
    context.apply_normalization(&mut store, first, BYTE)?;
    context.apply_normalization(&mut store, second, BYTE)?;
    assert_eq!(context.ready_equation()?, Some(id));
    assert_eq!(context.ready_equation()?, None);
    assert_eq!(context.resume_equation(&store, id)?, Outcome::Complete);
    assert_eq!(context.ready_equation()?, None);
    assert_eq!(
        context.resume_equation(&store, id),
        Err(Error::InvalidIdentity)
    );
    context.finish(&mut store)?;
    Ok(())
}

#[test]
fn normalization_preserves_callable_binders_around_a_selected_result() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let byte = context.known(&store, BYTE)?;
    let projected = context.projected(&store, ITEM, byte, &[], &[])?;
    let unit = context.known(&store, TypeId::UNIT)?;
    let callable = context.construct(
        Constructor::Function {
            bound_regions: 1,
            parameters: 1,
            unsafe_: false,
            convention: rustgm::types::callable::Convention::RUST,
        },
        &[projected, unit],
    )?;
    let reference = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: BYTE,
    })?;
    let parameters = store.intern_list(&[reference])?;
    let normalized = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let known = context.known(&store, normalized)?;
    let id = pending(context.equate_deferred(&store, callable, known, Relation::Shape)?)?;
    context.apply_normalization(&mut store, callable, normalized)?;
    assert_eq!(context.ready_equation()?, Some(id));
    assert_eq!(context.resume_equation(&store, id)?, Outcome::Complete);
    assert_eq!(context.resolved(&mut store, callable)?, normalized);
    assert_eq!(store.escaping_binders(normalized), Some(0));
    assert!(context.region_equalities.is_empty());
    Ok(())
}
