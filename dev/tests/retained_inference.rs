//! Partial publication keeps query identities and debts available for later conditions.
use rustgm::support::hash::HashMap;
use rustgm::types::{
    Primitive, RegionId, Store, TypeId,
    arguments::Constant,
    infer::{Constructor, Context, Domain, Error, arguments::Argument, refinement::Refinement},
    substitute::Bound,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[test]
fn repeated_publication_reuses_fresh_type_and_const_holes_then_refreshes_the_query() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Any)?;
    let (mut query, copied) = Refinement::new(&store, &mut source, &[Argument::Type(input)])?;
    let Argument::Type(copied) = copied[0] else {
        return Err("missing input".into());
    };
    let element = query.context.variable(Domain::Any)?;
    let length = query
        .context
        .const_variable(&store, TypeId::primitive(Primitive::Usize))?;
    let array = query
        .context
        .construct(Constructor::ArrayInfer(length), &[element])?;
    query.context.equate(&store, copied, array)?;
    query
        .context
        .region_equalities
        .push((RegionId(41), RegionId(42)));
    assert!(query.publish(
        &mut store,
        &mut source,
        &mut Bound::restore_regions(HashMap::default())
    )?);
    assert_eq!(query.context.steps, 0);
    let sentinel = source.variable(Domain::Any)?;
    for _ in 0..8 {
        assert!(!query.publish(
            &mut store,
            &mut source,
            &mut Bound::restore_regions(HashMap::default())
        )?);
        assert_eq!(query.context.steps, 0);
        assert_eq!(
            query.context.region_equalities,
            [(RegionId(41), RegionId(42))]
        );
    }
    assert_eq!(source.variable(Domain::Any)?.0, sentinel.0 + 1);
    let byte = source.known(&store, TypeId::primitive(Primitive::U8))?;
    let fixed = source.construct(Constructor::Array(3), &[byte])?;
    source.equate(&store, input, fixed)?;
    query.refresh(&store, &mut source)?;
    assert_eq!(
        query.context.resolved(&mut store, element)?,
        TypeId::primitive(Primitive::U8)
    );
    let three = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 3,
    })?;
    assert_eq!(query.context.resolved_const(&mut store, length)?, three);
    let completed = query.commit(
        &mut store,
        &mut source,
        &mut Bound::restore_regions(HashMap::default()),
    )?;
    assert!(!completed.changed);
    assert_eq!(completed.equalities, [(RegionId(41), RegionId(42))]);
    Ok(())
}
#[test]
fn a_later_conflict_cannot_publish_new_caller_bindings() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let input = source.variable(Domain::Any)?;
    let (mut query, copied) = Refinement::new(&store, &mut source, &[Argument::Type(input)])?;
    let Argument::Type(copied) = copied[0] else {
        return Err("missing input".into());
    };
    let child = query.context.variable(Domain::Any)?;
    let pair = query
        .context
        .construct(Constructor::Tuple, &[child, child])?;
    query.context.equate(&store, copied, pair)?;
    assert!(query.publish(
        &mut store,
        &mut source,
        &mut Bound::restore_regions(HashMap::default())
    )?);
    let caller_child = source.child(&store, input, 0)?;
    let byte = source.known(&store, TypeId::primitive(Primitive::U8))?;
    source.equate(&store, caller_child, byte)?;
    let word = query
        .context
        .known(&store, TypeId::primitive(Primitive::U16))?;
    query.context.equate(&store, child, word)?;
    let sentinel = source.variable(Domain::Any)?;
    assert!(matches!(
        query.publish(
            &mut store,
            &mut source,
            &mut Bound::restore_regions(HashMap::default())
        ),
        Err(Error::Mismatch)
    ));
    assert_eq!(source.variable(Domain::Any)?.0, sentinel.0 + 1);
    assert_eq!(
        source.resolved(&mut store, caller_child)?,
        TypeId::primitive(Primitive::U8)
    );
    Ok(())
}
