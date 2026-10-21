use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    infer::{Constructor, Context, Domain, Error},
};

#[test]
fn empty_instantiation_preserves_rigid_shared_graphs_and_later_substitution()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{
        arguments::{Argument, ConstKind, Constant},
        infer::arguments::Argument as Inferred,
        parameters::{Parameter, ParameterKind},
        substitute::Bound,
    };
    let mut store = Store::default();
    let owner = ItemId(7);
    let t = store.intern_parameter(Parameter {
        owner,
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let usize_ = TypeId::primitive(Primitive::Usize);
    let n = store.intern_parameter(Parameter {
        owner,
        index: 1,
        kind: ParameterKind::Const { ty: usize_ },
    })?;
    let t_ty = store.intern(Kind::Parameter(t))?;
    let length = store.intern_const(ConstKind::Parameter(n))?;
    let array = store.intern(Kind::ArrayTerm {
        element: t_ty,
        length,
    })?;
    let local = store.intern(Kind::Reference {
        region: RegionId::bound(0, 0)?,
        mutable: false,
        referent: array,
    })?;
    let captured = store.intern(Kind::Reference {
        region: RegionId::bound(1, 3)?,
        mutable: false,
        referent: t_ty,
    })?;
    let parameters = store.intern_list(&[local, captured])?;
    let mut template = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: t_ty,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    for _ in 0..12 {
        let fields = store.intern_list(&[template, template])?;
        template = store.intern(Kind::Tuple(fields))?;
    }
    let mut context = Context::default();
    for _ in 0..128 {
        let root = context.instantiate(&mut store, template, &[])?;
        assert_eq!(context.resolved(&mut store, root)?, template);
    }
    assert!(
        context.steps < 1024,
        "{} steps for identity substitution",
        context.steps
    );
    let byte = TypeId::primitive(Primitive::U8);
    let count = store.intern_constant(Constant {
        ty: usize_,
        bits: 3,
    })?;
    let actual_t = context.known(&store, byte)?;
    let actual_n = context.known_const(&store, count)?;
    let result = context.instantiate(
        &mut store,
        template,
        &[
            (t, Inferred::Type(actual_t)),
            (n, Inferred::Const(actual_n)),
        ],
    )?;
    let expected = Bound::parameters(
        &store,
        &[(t, Argument::Type(byte)), (n, Argument::Const(count))],
    )?
    .apply(&mut store, template)?;
    assert_eq!(context.resolved(&mut store, result)?, expected);
    let unchanged = context.instantiate(&mut store, template, &[])?;
    assert_eq!(context.resolved(&mut store, unchanged)?, template);
    Ok(())
}

#[test]
fn partial_resolution_does_not_poison_later_requests() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let mut cx = Context::default();
    let byte = TypeId::primitive(Primitive::U8);
    let known = cx.known(&store, byte)?;
    let missing = cx.variable(Domain::Any)?;
    let tuple = cx.construct(Constructor::Tuple, &[known, missing])?;
    let independent = cx.construct(Constructor::Array(2), &[known])?;
    let expected_array = store.intern(Kind::Array {
        element: byte,
        length: 2,
    })?;
    for _ in 0..4 {
        assert_eq!(cx.resolved(&mut store, tuple), Err(Error::Unresolved));
        assert_eq!(cx.resolved(&mut store, independent)?, expected_array);
    }
    let boolean = TypeId::primitive(Primitive::Bool);
    let resolved = cx.known(&store, boolean)?;
    cx.equate(&store, missing, resolved)?;
    let fields = store.intern_list(&[byte, boolean])?;
    let expected = store.intern(Kind::Tuple(fields))?;
    assert_eq!(cx.resolved(&mut store, tuple)?, expected);
    Ok(())
}

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
        bound_regions: 0,
        parameters: list,
        result: i32_,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let a = store.intern(Kind::FunctionItem {
        arguments: rustgm::types::GenericArgs::EMPTY,
        item: ItemId(0),
        signature,
    })?;
    let b = store.intern(Kind::FunctionItem {
        arguments: rustgm::types::GenericArgs::EMPTY,
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
                arguments: rustgm::types::GenericArgs::EMPTY,
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

#[test]
fn constant_pattern_equality_preserves_expression_constraints()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let mut cx = Context::default();
    let variable = cx.variable(Domain::Any)?;
    let byte = cx.known(&store, TypeId::primitive(Primitive::U8))?;
    let mut reference = |region, mutable, referent| {
        cx.construct(Constructor::Reference { mutable, region }, &[referent])
    };
    let input = reference(RegionId(1), false, variable)?;
    let constant = reference(RegionId::STATIC, false, byte)?;
    let other = reference(RegionId(2), false, byte)?;
    cx.equate(&store, input, other)?;
    cx.equate_pattern(&store, input, constant)?;
    assert_eq!(cx.region_equalities, [(RegionId(1), RegionId(2))]);
    let solved = cx.finish(&mut store)?;
    assert_eq!(
        solved.types[variable.0 as usize],
        TypeId::primitive(Primitive::U8)
    );
    assert_ne!(
        solved.types[input.0 as usize],
        solved.types[constant.0 as usize]
    );
    Ok(())
}
