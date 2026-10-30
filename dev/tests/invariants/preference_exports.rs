//! Numeric hints reuse committed graph mappings and never manufacture caller roots.
use super::*;
use crate::types::{
    Primitive, TypeId,
    infer::{Constructor, Value},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn input(
    store: &Store,
    source: &mut Context,
    domain: Domain,
) -> Result<(Refinement, TermId, TermId), Error> {
    let original = source.variable(domain)?;
    let (query, roots) = Refinement::new(store, source, &[super::super::Argument::Type(original)])?;
    let super::super::Argument::Type(copied) = roots[0] else {
        return Err(Error::InvalidIdentity);
    };
    Ok((query, original, copied))
}

#[test]
fn shared_query_aliases_map_once_to_the_actual_caller_float() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let (mut query, original, copied) = input(&store, &mut source, Domain::Float)?;
    let alias = query.context.variable(Domain::Float)?;
    query.context.equate(&store, copied, alias)?;
    let before = source.terms.len();
    let (changed, hints) = query.publish_with_numeric_hints(
        &mut store,
        &mut source,
        &mut Bound::new(Vec::new()),
        &[alias, copied, alias],
    )?;
    assert!(!changed);
    assert_eq!(hints, vec![source.root(original)?]);
    assert_eq!(source.terms.len(), before);
    assert_eq!(source.unresolved_domain(original)?, Some(Domain::Float));
    Ok(())
}

#[test]
fn explicit_caller_f64_constraints_remove_a_pending_float_preference() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let (mut query, original, copied) = input(&store, &mut source, Domain::Float)?;
    let f64_type = TypeId::primitive(Primitive::F64);
    let explicit = source.known(&store, f64_type)?;
    source.equate(&store, original, explicit)?;
    let (_, hints) = query.publish_with_numeric_hints(
        &mut store,
        &mut source,
        &mut Bound::new(Vec::new()),
        &[copied],
    )?;
    assert!(hints.is_empty());
    assert_eq!(source.resolved(&mut store, original)?, f64_type);
    Ok(())
}

#[test]
fn fresh_connected_numeric_leaves_reuse_the_publication_imports() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let (mut query, original, copied) = input(&store, &mut source, Domain::Any)?;
    let fresh = query.context.variable(Domain::Float)?;
    let tuple = query
        .context
        .construct(Constructor::Tuple, &[fresh, fresh])?;
    query.context.equate(&store, copied, tuple)?;
    let (changed, hints) = query.publish_with_numeric_hints(
        &mut store,
        &mut source,
        &mut Bound::new(Vec::new()),
        &[fresh, fresh],
    )?;
    assert!(changed);
    assert_eq!(hints.len(), 1);
    let root = source.root(original)?;
    let term = source.terms[root.0 as usize];
    assert!(matches!(term.value, Value::Construct(Constructor::Tuple)));
    let fields = source.edges[term.start as usize..term.end as usize].to_vec();
    assert_eq!(fields.len(), 2);
    assert_eq!(source.root(fields[0])?, hints[0]);
    assert_eq!(source.root(fields[1])?, hints[0]);
    assert_eq!(source.unresolved_domain(hints[0])?, Some(Domain::Float));
    Ok(())
}

#[test]
fn unconnected_query_floats_allocate_no_caller_roots_or_export_records() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let (mut query, original, _) = input(&store, &mut source, Domain::Float)?;
    let private = query.context.variable(Domain::Float)?;
    let before = source.terms.len();
    let (_, hints) = query.publish_with_numeric_hints(
        &mut store,
        &mut source,
        &mut Bound::new(Vec::new()),
        &[private],
    )?;
    assert!(hints.is_empty() && hints.capacity() == 0);
    assert_eq!(source.terms.len(), before);
    assert_eq!(source.unresolved_domain(original)?, Some(Domain::Float));
    let mut detached = Refinement::default();
    let private = detached.context.variable(Domain::Float)?;
    let (_, hints) = detached.publish_with_numeric_hints(
        &mut store,
        &mut source,
        &mut Bound::new(Vec::new()),
        &[private],
    )?;
    assert!(hints.is_empty() && hints.capacity() == 0);
    assert_eq!(source.terms.len(), before);
    Ok(())
}

#[test]
fn exhausted_hint_payment_leaves_caller_inputs_unpublished() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let (mut query, original, copied) = input(&store, &mut source, Domain::Any)?;
    let fresh = query.context.variable(Domain::Float)?;
    query.context.equate(&store, copied, fresh)?;
    let before = source.terms.len();
    source.steps = 1_000_000;
    for _ in 0..2 {
        assert!(matches!(
            query.publish_with_numeric_hints(
                &mut store,
                &mut source,
                &mut Bound::new(Vec::new()),
                &[fresh],
            ),
            Err(Error::Budget)
        ));
    }
    assert!(source.steps >= 1_000_002);
    assert_eq!(source.terms.len(), before);
    assert_eq!(source.unresolved_domain(original)?, Some(Domain::Any));
    Ok(())
}

#[test]
fn empty_hint_publication_keeps_ordinary_work_and_retention() -> TestResult {
    let mut observations = Vec::new();
    for requested in [false, true] {
        let mut store = Store::default();
        let mut source = Context::default();
        let (mut query, original, copied) = input(&store, &mut source, Domain::Float)?;
        let f32_type = TypeId::primitive(Primitive::F32);
        let selected = query.context.known(&store, f32_type)?;
        query.context.equate(&store, copied, selected)?;
        let changed = if requested {
            let (changed, hints) = query.publish_with_numeric_hints(
                &mut store,
                &mut source,
                &mut Bound::new(Vec::new()),
                &[],
            )?;
            assert!(hints.is_empty() && hints.capacity() == 0);
            changed
        } else {
            query.publish(&mut store, &mut source, &mut Bound::new(Vec::new()))?
        };
        assert_eq!(source.resolved(&mut store, original)?, f32_type);
        observations.push((changed, source.steps, source.retained_allocation()));
    }
    assert_eq!(observations[0], observations[1]);
    Ok(())
}
