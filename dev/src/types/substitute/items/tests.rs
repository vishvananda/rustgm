//! Callable substitution must distinguish surrounding binders and reuse shared DAGs.
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId, arguments::Argument,
    substitute::Bound,
};
use std::collections::HashMap;

fn callable(store: &mut Store, item: u32, region: RegionId) -> Result<TypeId, &'static str> {
    let reference = store.intern(Kind::Reference {
        mutable: false,
        region,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let signature = signature(store, reference)?;
    let arguments = store.intern_arguments(&[Argument::Lifetime(region)])?;
    store.intern(Kind::FunctionItem {
        item: ItemId(item),
        arguments,
        signature,
    })
}
fn signature(store: &mut Store, parameter: TypeId) -> Result<TypeId, &'static str> {
    let parameters = store.intern_list(&[parameter])?;
    store.intern(Kind::Function {
        parameters,
        result: TypeId::primitive(Primitive::U8),
        bound_regions: 1,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })
}

#[test]
fn selected_items_preserve_local_binders_and_shift_outer_arguments() -> Result<(), &'static str> {
    let mut store = Store::default();
    let source = callable(&mut store, 7, RegionId::bound(0, 0)?)?;
    let target = callable(&mut store, 8, RegionId::bound(0, 0)?)?;
    let shifted_source = Bound::shift(1).apply(&mut store, source)?;
    let shifted_target = Bound::shift(1).apply(&mut store, target)?;
    let outer = signature(&mut store, shifted_source)?;
    let local = signature(&mut store, source)?;
    let expected_outer = signature(&mut store, shifted_target)?;
    let elements = store.intern_list(&[source, outer, local])?;
    let root = store.intern(Kind::Tuple(elements))?;
    let expected = store.intern_list(&[target, expected_outer, local])?;
    let expected = store.intern(Kind::Tuple(expected))?;
    let mut fold = Bound::selected_items(&store, HashMap::from([(source, target)]))?;
    assert_eq!(fold.apply(&mut store, root)?, expected);
    assert_eq!(fold.apply(&mut store, local)?, local);
    Ok(())
}

#[test]
fn selected_items_visit_shared_subtrees_once() -> Result<(), &'static str> {
    let mut store = Store::default();
    let signature = signature(&mut store, TypeId::primitive(Primitive::U8))?;
    let source = store.intern(Kind::FunctionItem {
        item: ItemId(1),
        arguments: GenericArgs::EMPTY,
        signature,
    })?;
    let target = store.intern(Kind::FunctionItem {
        item: ItemId(2),
        arguments: GenericArgs::EMPTY,
        signature,
    })?;
    let mut root = source;
    let mut expected = target;
    for _ in 0..128 {
        let children = store.intern_list(&[root, root])?;
        root = store.intern(Kind::Tuple(children))?;
        let children = store.intern_list(&[expected, expected])?;
        expected = store.intern(Kind::Tuple(children))?;
    }
    let mut fold = Bound::selected_items(&store, HashMap::from([(source, target)]))?;
    assert_eq!(fold.apply(&mut store, root)?, expected);
    assert!(
        fold.steps < 2_000,
        "shared replacement DAG was expanded: {}",
        fold.steps
    );
    let steps = fold.steps;
    for _ in 0..128 {
        assert_eq!(fold.apply(&mut store, root)?, expected);
    }
    assert_eq!(fold.steps, steps);
    Ok(())
}
