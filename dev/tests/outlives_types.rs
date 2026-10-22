use crate::semantic;
use rustgm::types::{
    ItemId, Kind, RegionId, Store, TypeId,
    aggregate::Definitions,
    outlives::{Bounds, Queries},
};

#[test]
fn recursive_nominal_bounds_propagate_through_argument_permutations_once() -> Result<(), String> {
    let source = "struct A<'a,'b>{next:*const B<'b,'a>}struct B<'a,'b>{next:*const A<'b,'a>,value:&'a &'b u8}enum E<'a,'b>{Empty,Full(A<'a,'b>)}";
    let (report, mut facts) = semantic::analyze(source)?;
    let bounds = Bounds::infer(&mut facts.types, &facts.aggregates, &[])?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    for (name, pair) in [("A", (a, b)), ("B", (b, a)), ("E", (a, b))] {
        let (index, _) = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find(|(_, d)| report.package.names.get(d.name) == name)
            .ok_or("missing nominal definition")?;
        let item = ItemId(u32::try_from(index).map_err(|e| e.to_string())?);
        assert_eq!(bounds.get(item), Some([pair].as_slice()), "{name}");
    }
    assert!(bounds.steps < 100, "{}", bounds.steps);
    Ok(())
}

fn reference(
    store: &mut Store,
    region: RegionId,
    referent: TypeId,
) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region,
        referent,
    })
}

#[test]
fn nested_binders_export_only_free_requirements() -> Result<(), &'static str> {
    let mut store = Store::default();
    let a = RegionId::bound(1, 0)?;
    let b = RegionId::bound(1, 1)?;
    let inner = reference(&mut store, b, TypeId::UNIT)?;
    let outer = reference(&mut store, a, inner)?;
    let local = reference(&mut store, RegionId::bound(0, 0)?, inner)?;
    let parameters = store.intern_list(&[outer, local])?;
    let function = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let bounds = Bounds::default();
    let callables = rustgm::types::callable::Templates::default();
    let mut query = Queries::new(&store, &bounds, &callables);
    assert_eq!(
        query.requirements(function)?,
        &[(RegionId::bound(0, 1)?, RegionId::bound(0, 0)?)]
    );
    Ok(())
}

#[test]
fn deep_shared_graphs_are_iterative_and_cache_complete_queries() -> Result<(), &'static str> {
    let mut store = Store::default();
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let inner = reference(&mut store, b, TypeId::UNIT)?;
    let mut ty = reference(&mut store, a, inner)?;
    for _ in 0..10_000 {
        let fields = store.intern_list(&[ty, ty])?;
        ty = store.intern(Kind::Tuple(fields))?;
    }
    let bounds = Bounds::infer(&mut store, &Definitions::default(), &[])?;
    let callables = rustgm::types::callable::Templates::default();
    let mut query = Queries::new(&store, &bounds, &callables);
    for _ in 0..100 {
        assert_eq!(query.requirements(ty)?, &[(b, a)]);
    }
    assert!(query.requirements(TypeId(u32::MAX)).is_err());
    Ok(())
}

#[test]
fn explicit_template_bounds_use_the_same_finite_nominal_propagation() -> Result<(), String> {
    let (report, mut facts) = semantic::analyze("struct A<'a>(&'a u8);struct B<'b>(A<'b>);")?;
    let items: Vec<_> = ["A", "B"]
        .iter()
        .map(|name| {
            let index = report
                .resolution
                .definitions
                .iter()
                .position(|d| report.package.names.get(d.name) == *name)
                .ok_or("missing nominal definition")?;
            Ok(ItemId(u32::try_from(index).map_err(|e| e.to_string())?))
        })
        .collect::<Result<_, String>>()?;
    let pair = (RegionId::bound(0, 0)?, RegionId::STATIC);
    let bounds = Bounds::infer(&mut facts.types, &facts.aggregates, &[(items[0], pair)])?;
    for item in items {
        assert_eq!(bounds.get(item), Some([pair].as_slice()));
    }
    Ok(())
}

#[test]
fn live_components_include_nested_free_regions_but_skip_static_bound_and_empty_values()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let a = RegionId(1);
    let b = RegionId(2);
    let inner = reference(&mut store, b, TypeId::UNIT)?;
    let outer = reference(&mut store, a, inner)?;
    let static_ = reference(&mut store, RegionId::STATIC, TypeId::UNIT)?;
    let empty = store.intern(Kind::Array {
        element: outer,
        length: 0,
    })?;
    let nonempty = store.intern(Kind::Array {
        element: outer,
        length: 1,
    })?;
    let captured = reference(&mut store, RegionId::bound(1, 0)?, TypeId::UNIT)?;
    let bound = reference(&mut store, RegionId::bound(0, 0)?, TypeId::UNIT)?;
    let parameters = store.intern_list(&[captured, bound, static_])?;
    let function = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let mut query = rustgm::types::outlives::LiveRegions::default();
    assert_eq!(query.get(&store, outer)?, [a, b]);
    assert_eq!(query.get(&store, nonempty)?, [a, b]);
    assert!(query.get(&store, empty)?.is_empty());
    assert!(query.get(&store, static_)?.is_empty());
    assert_eq!(query.get(&store, function)?, [RegionId::bound(0, 0)?]);
    let steps = query.steps();
    for _ in 0..100 {
        assert_eq!(query.get(&store, outer)?, [a, b]);
    }
    assert_eq!(query.steps(), steps);
    assert!(query.get(&store, TypeId(u32::MAX)).is_err());
    Ok(())
}
