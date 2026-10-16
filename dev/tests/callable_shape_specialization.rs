//! Shape inference opens enclosing binders without erasing an entire inferred callable.
use rustgm::types::{
    Kind, Primitive, RegionId, Store, TypeId,
    infer::{Constructor, Context, Domain, Error, TermId},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn signature(store: &mut Store, parameter: TypeId, binders: u32) -> Result<TypeId, &'static str> {
    let parameters = store.intern_list(&[parameter])?;
    store.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: binders,
        unsafe_: false,
    })
}

fn polymorphic(store: &mut Store) -> Result<TypeId, &'static str> {
    let reference = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    signature(store, reference, 1)
}

fn partial(context: &mut Context, store: &Store) -> Result<(TermId, TermId), Error> {
    let hole = context.variable(Domain::Any)?;
    let unit = context.known(store, TypeId::UNIT)?;
    let shape = context.construct(
        Constructor::Function {
            parameters: 1,
            bound_regions: 0,
            unsafe_: false,
        },
        &[hole, unit],
    )?;
    Ok((shape, hole))
}

fn parameter(store: &Store, ty: TypeId) -> Result<TypeId, &'static str> {
    let Some(Kind::Function { parameters, .. }) = store.kind(ty) else {
        return Err("missing callable");
    };
    store
        .list(parameters)
        .and_then(|p| p.first())
        .copied()
        .ok_or("missing parameter")
}

#[test]
fn an_entire_inferred_higher_ranked_type_keeps_its_binders() -> TestResult {
    let mut store = Store::default();
    let poly = polymorphic(&mut store)?;
    let outer = signature(&mut store, poly, 0)?;
    let mut context = Context::default();
    let (expected, hole) = partial(&mut context, &store)?;
    let result = context.specialize_shape(&mut store, outer, expected, 10)?;
    assert_eq!(result.ty, outer);
    assert_eq!(result.regions, 0);
    assert_eq!(context.resolved(&mut store, hole), Err(Error::Unresolved));
    let actual = context.known(&store, result.ty)?;
    context.equate_shape(&store, actual, expected)?;
    assert_eq!(context.resolved(&mut store, hole)?, poly);
    Ok(())
}

#[test]
fn a_hole_inside_a_callable_uses_an_opened_source_binder_without_inferring_it() -> TestResult {
    let mut store = Store::default();
    let source = polymorphic(&mut store)?;
    let mut context = Context::default();
    let (expected, hole) = partial(&mut context, &store)?;
    let result = context.specialize_shape(&mut store, source, expected, 10)?;
    assert_eq!(result.regions, 1);
    assert!(matches!(
        store.kind(source),
        Some(Kind::Function {
            bound_regions: 1,
            ..
        })
    ));
    assert!(matches!(
        store.kind(result.ty),
        Some(Kind::Function {
            bound_regions: 0,
            ..
        })
    ));
    assert!(matches!(
        store.kind(parameter(&store, result.ty)?),
        Some(Kind::Reference {
            region: RegionId(10),
            ..
        })
    ));
    assert_eq!(context.resolved(&mut store, hole), Err(Error::Unresolved));
    Ok(())
}

#[test]
fn identical_sibling_callable_occurrences_receive_independent_instances() -> TestResult {
    let mut store = Store::default();
    let source = polymorphic(&mut store)?;
    let fields = store.intern_list(&[source, source])?;
    let pair = store.intern(Kind::Tuple(fields))?;
    let mut context = Context::default();
    let (shape, _) = partial(&mut context, &store)?;
    let expected = context.construct(Constructor::Tuple, &[shape, shape])?;
    let result = context.specialize_shape(&mut store, pair, expected, 10)?;
    assert_eq!(result.regions, 2);
    let Some(Kind::Tuple(fields)) = store.kind(result.ty) else {
        return Err("missing specialized pair".into());
    };
    let fields = store.list(fields).ok_or("missing specialized fields")?;
    assert_ne!(fields[0], fields[1]);
    for (&ty, expected) in fields.iter().zip([RegionId(10), RegionId(11)]) {
        assert!(
            matches!(store.kind(parameter(&store, ty)?), Some(Kind::Reference { region, .. }) if region == expected)
        );
    }
    Ok(())
}

#[test]
fn unchanged_shared_shapes_are_visited_once_without_collapsing_binders() -> TestResult {
    let mut store = Store::default();
    let mut source = polymorphic(&mut store)?;
    let mut context = Context::default();
    let mut expected = context.variable(Domain::Any)?;
    for _ in 0..512 {
        let fields = store.intern_list(&[source, source])?;
        source = store.intern(Kind::Tuple(fields))?;
        expected = context.construct(Constructor::Tuple, &[expected, expected])?;
    }
    let before = context.steps;
    let result = context.specialize_shape(&mut store, source, expected, 10)?;
    assert_eq!(result.ty, source);
    assert_eq!(result.regions, 0);
    assert_eq!(result.steps, context.steps - before);
    assert!(result.steps < 20_000);
    Ok(())
}

#[test]
fn binder_detection_observes_new_bindings_without_interning_partial_shapes() -> TestResult {
    let mut store = Store::default();
    let source = polymorphic(&mut store)?;
    let mut context = Context::default();
    let hole = context.variable(Domain::Any)?;
    let mut expected = hole;
    for _ in 0..512 {
        expected = context.construct(Constructor::Tuple, &[expected, expected])?;
    }
    let count = store.len();
    let before = context.steps;
    assert!(!context.contains_late_callables(&store, expected)?);
    assert!(context.steps - before < 20_000);
    assert_eq!(store.len(), count);
    assert_eq!(context.resolved(&mut store, hole), Err(Error::Unresolved));
    let actual = context.known(&store, source)?;
    context.equate_shape(&store, hole, actual)?;
    let before = context.steps;
    assert!(context.contains_late_callables(&store, expected)?);
    assert!(context.steps - before < 20_000);
    assert_eq!(store.len(), count);
    Ok(())
}
