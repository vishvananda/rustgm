use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    infer::{Constructor, Context, Domain, Error},
};

#[test]
fn canonical_shapes_share_lists_but_preserve_semantic_identity() -> Result<(), &'static str> {
    let mut store = Store::default();
    for primitive in Primitive::ALL {
        assert_eq!(
            store.kind(TypeId::primitive(primitive)),
            Some(Kind::Primitive(primitive))
        );
    }
    let i32_ = TypeId::primitive(Primitive::I32);
    let list = store.intern_list(&[i32_, i32_])?;
    for _ in 0..10_000 {
        assert_eq!(store.intern_list(&[i32_, i32_])?, list);
    }
    assert_eq!(store.edge_count(), 2);
    let tuple = store.intern(Kind::Tuple(list))?;
    assert_eq!(store.intern(Kind::Tuple(list))?, tuple);
    let signature = store.intern(Kind::Function {
        parameters: list,
        result: i32_,
        unsafe_: false,
    })?;
    let a = store.intern(Kind::FunctionItem {
        item: ItemId(0),
        signature,
    })?;
    let b = store.intern(Kind::FunctionItem {
        item: ItemId(1),
        signature,
    })?;
    assert_ne!(a, b);
    assert_ne!(a, signature);
    let a = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId(1),
        referent: i32_,
    })?;
    let b = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId(2),
        referent: i32_,
    })?;
    assert_ne!(a, b);
    let before = (store.len(), store.edge_count());
    assert!(store.intern_list(&[TypeId(u32::MAX)]).is_err());
    assert!(
        store
            .intern(Kind::FunctionItem {
                item: ItemId(2),
                signature: i32_
            })
            .is_err()
    );
    assert_eq!((store.len(), store.edge_count()), before);
    Ok(())
}

#[test]
fn numeric_constraints_merge_domains_and_fallback_only_after_constraints() -> Result<(), Error> {
    let mut store = Store::default();
    let mut cx = Context::default();
    let x = cx.variable(Domain::Any)?;
    let y = cx.variable(Domain::Integer)?;
    let z = cx.variable(Domain::Integer)?;
    let f = cx.variable(Domain::Float)?;
    let i16_ = cx.known(&store, TypeId::primitive(Primitive::I16))?;
    cx.equate(&store, x, y)?;
    cx.equate(&store, i16_, x)?;
    let result = cx.finish(&mut store)?;
    assert_eq!(
        result.types[x.0 as usize],
        TypeId::primitive(Primitive::I16)
    );
    assert_eq!(
        result.types[y.0 as usize],
        TypeId::primitive(Primitive::I16)
    );
    assert_eq!(
        result.types[z.0 as usize],
        TypeId::primitive(Primitive::I32)
    );
    assert_eq!(
        result.types[f.0 as usize],
        TypeId::primitive(Primitive::F64)
    );
    Ok(())
}

#[test]
fn incompatible_domains_never_and_unresolved_variables_are_not_coercions() -> Result<(), Error> {
    let mut store = Store::default();
    let mut cx = Context::default();
    let integer = cx.variable(Domain::Integer)?;
    let float = cx.variable(Domain::Float)?;
    assert_eq!(cx.equate(&store, integer, float), Err(Error::Mismatch));
    let never = cx.known(&store, TypeId::NEVER)?;
    let unit = cx.known(&store, TypeId::UNIT)?;
    assert_eq!(cx.equate(&store, never, unit), Err(Error::Mismatch));
    let mut cx = Context::default();
    cx.variable(Domain::Any)?;
    assert!(matches!(cx.finish(&mut store), Err(Error::Unresolved)));
    Ok(())
}

#[test]
fn occurs_checks_follow_indirect_bindings_and_shared_graphs() -> Result<(), Error> {
    let store = Store::default();
    let mut cx = Context::default();
    let x = cx.variable(Domain::Any)?;
    let y = cx.variable(Domain::Any)?;
    let pair = cx.construct(Constructor::Tuple, &[x, x])?;
    cx.equate(&store, y, pair)?;
    assert_eq!(cx.equate(&store, x, y), Err(Error::Occurs));
    let mut cx = Context::default();
    let leaf = cx.variable(Domain::Any)?;
    let mut shared = cx.known(&store, TypeId::UNIT)?;
    for _ in 0..100 {
        shared = cx.construct(Constructor::Tuple, &[shared, shared])?;
    }
    cx.equate(&store, leaf, shared)?;
    assert!(
        cx.steps < 300,
        "shared subtrees must not cause exponential traversal"
    );
    Ok(())
}

#[test]
fn deep_structures_unify_and_finalize_without_recursive_ownership() -> Result<(), Error> {
    let mut store = Store::default();
    let mut cx = Context::default();
    let x = cx.variable(Domain::Integer)?;
    let y = cx.known(&store, TypeId::primitive(Primitive::U64))?;
    let (mut a, mut b) = (x, y);
    for _ in 0..10_000 {
        a = cx.construct(Constructor::Tuple, &[a])?;
        b = cx.construct(Constructor::Tuple, &[b])?;
    }
    cx.equate(&store, a, b)?;
    let result = cx.finish(&mut store)?;
    assert_eq!(result.types[a.0 as usize], result.types[b.0 as usize]);
    assert_eq!(
        result.types[x.0 as usize],
        TypeId::primitive(Primitive::U64)
    );
    assert!(result.steps < 100_000);
    Ok(())
}

#[test]
fn canonical_structures_constrain_terms_and_retain_region_obligations()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let ty = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::STATIC,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let mut cx = Context::default();
    let variable = cx.variable(Domain::Any)?;
    let reference = cx.construct(
        Constructor::Reference {
            mutable: false,
            region: RegionId(1),
        },
        &[variable],
    )?;
    let known = cx.known(&store, ty)?;
    cx.equate(&store, reference, known)?;
    let result = cx.finish(&mut store)?;
    assert_eq!(
        result.types[variable.0 as usize],
        TypeId::primitive(Primitive::U8)
    );
    assert_eq!(result.region_equalities, [(RegionId(1), RegionId::STATIC)]);
    assert_ne!(result.types[reference.0 as usize], ty);
    Ok(())
}
