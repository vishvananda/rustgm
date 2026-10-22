//! Interface views retain source ownership while sharing one lifetime fold.
use super::{Argument, Bound, Context, Refinement, Store, map_types};
use crate::support::hash::HashMap;
use crate::types::{Kind, RegionId, TypeId, callable::Convention, infer::Domain};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn reference(store: &mut Store, region: RegionId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        referent: TypeId::UNIT,
        mutable: false,
        region,
    })
}
#[test]
fn incoming_views_open_prepared_aliases_without_rebinding_source_roots() -> TestResult {
    let mut store = Store::default();
    let mut parent = Context::default();
    let original = parent.variable(Domain::Any)?;
    let (mut child, _) = Refinement::new(&store, &mut parent, &[Argument::Type(original)])?;
    let slot = RegionId::bound(0, 0)?;
    let alias = RegionId(11);
    let canonical = reference(&mut store, slot)?;
    let aliased = reference(&mut store, alias)?;
    let prototype = store.intern_list(&[canonical, canonical])?;
    let prototype = store.intern(Kind::Tuple(prototype))?;
    let Argument::Type(child_root) = child.roots(true)[0] else {
        return Err("missing child root".into());
    };
    let expected = child.context.known(&store, prototype)?;
    child.context.equate(&store, child_root, expected)?;
    let fields = store.intern_list(&[aliased, aliased])?;
    let tuple = store.intern(Kind::Tuple(fields))?;
    let known = parent.known(&store, tuple)?;
    parent.equate(&store, original, known)?;
    let mut opening = Bound::restore_regions(HashMap::from_iter([(alias, slot)]));
    let pairs = child.incoming_regions(&mut store, &mut parent, &mut opening)?;
    let roots = pairs.into_iter().flat_map(|(a, b)| [a, b]).collect();
    let roots = map_types(
        &mut child.context,
        &mut store,
        roots,
        &mut Bound::new(vec![RegionId(21)]),
    )?;
    let pairs = roots
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[0], p[1]))
        .collect::<Vec<_>>();
    child.context.equate_input_pairs(&store, &pairs)?;
    let Argument::Type(result) = pairs[0].0 else {
        return Err("missing imported type".into());
    };
    let element = reference(&mut store, RegionId(21))?;
    let fields = store.intern_list(&[element, element])?;
    let expected = store.intern(Kind::Tuple(fields))?;
    assert_eq!(child.context.resolved(&mut store, result)?, expected);
    assert_eq!(parent.resolved(&mut store, original)?, tuple);
    assert!(parent.region_equalities.is_empty());
    assert!(child.context.region_equalities.is_empty());
    Ok(())
}
#[test]
fn receiving_fold_retains_distinct_slots_and_local_callable_binders() -> TestResult {
    let mut store = Store::default();
    let local = reference(&mut store, RegionId::bound(0, 0)?)?;
    let first = reference(&mut store, RegionId::bound(1, 0)?)?;
    let second = reference(&mut store, RegionId::bound(1, 1)?)?;
    let parameters = store.intern_list(&[local, first, second])?;
    let function = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: local,
        unsafe_: false,
        convention: Convention::RUST,
    })?;
    let a = reference(&mut store, RegionId(11))?;
    let b = reference(&mut store, RegionId(12))?;
    let parameters = store.intern_list(&[local, a, b])?;
    let expected = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: local,
        unsafe_: false,
        convention: Convention::RUST,
    })?;
    let mut context = Context::default();
    let original = context.known(&store, function)?;
    let roots = map_types(
        &mut context,
        &mut store,
        vec![Argument::Type(original)],
        &mut Bound::new(vec![RegionId(11), RegionId(12)]),
    )?;
    let Argument::Type(result) = roots[0] else {
        return Err("missing mapped callable".into());
    };
    assert_eq!(context.resolved(&mut store, result)?, expected);
    assert_ne!(a, b);
    assert_ne!(local, a);
    assert!(context.region_equalities.is_empty());
    assert_eq!(context.resolved(&mut store, original)?, function);
    Ok(())
}
#[test]
fn shared_interface_roots_use_one_mapping_per_refresh() -> TestResult {
    let mut observations = Vec::new();
    for width in [16, 64, 256, 1024] {
        let mut store = Store::default();
        let original = reference(&mut store, RegionId::bound(0, 0)?)?;
        let mut context = Context::default();
        let term = context.known(&store, original)?;
        let before = context.steps;
        let roots = map_types(
            &mut context,
            &mut store,
            vec![Argument::Type(term); width],
            &mut Bound::new(vec![RegionId(11)]),
        )?;
        assert!(roots.windows(2).all(|pair| pair[0] == pair[1]));
        observations.push(context.steps - before);
        assert!(context.region_equalities.is_empty());
    }
    for (width, work) in [16, 64, 256, 1024].into_iter().zip(&observations) {
        assert!(*work <= width * 3 + 20, "{width}: {observations:?}");
    }
    Ok(())
}
