use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId, infer::Context,
    substitute::Bound,
};

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
fn canonical_nominal_arguments_preserve_identity_and_pending_equalities()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let left_args = store.intern_regions(&[RegionId(11), RegionId::STATIC])?;
    let right_args = store.intern_regions(&[RegionId(12), RegionId::STATIC])?;
    assert_ne!(left_args, right_args);
    assert_eq!(store.intern_regions(&[])?, GenericArgs::EMPTY);
    for _ in 0..1000 {
        assert_eq!(
            store.intern_regions(&[RegionId(11), RegionId::STATIC])?,
            left_args
        );
    }
    let left = store.intern(Kind::Adt(ItemId(3), left_args))?;
    let right = store.intern(Kind::Adt(ItemId(3), right_args))?;
    let other = store.intern(Kind::Adt(ItemId(4), left_args))?;
    assert_ne!(left, right);
    assert_ne!(left, other);
    let mut infer = Context::default();
    let left = infer.known(&store, left)?;
    let right = infer.known(&store, right)?;
    let other = infer.known(&store, other)?;
    infer.equate(&store, left, right)?;
    assert_eq!(infer.region_equalities, [(RegionId(11), RegionId(12))]);
    assert!(infer.equate(&store, left, other).is_err());
    assert!(
        store
            .intern(Kind::Adt(ItemId(0), GenericArgs(u32::MAX)))
            .is_err()
    );
    Ok(())
}

#[test]
fn substitution_does_not_capture_nested_binders_or_share_argument_environments()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let local = reference(&mut store, RegionId::bound(0, 0)?, byte)?;
    let outer = reference(&mut store, RegionId::bound(1, 0)?, byte)?;
    let distant = reference(&mut store, RegionId::bound(2, 7)?, byte)?;
    let parameters = store.intern_list(&[local, outer, distant])?;
    let function = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: outer,
        unsafe_: false,
    })?;
    assert_eq!(store.escaping_binders(function), Some(2));
    let mut substitution = Bound::new(vec![RegionId::bound(0, 3)?]);
    let result = substitution.apply(&mut store, function)?;
    let Some(Kind::Function {
        parameters,
        result: output,
        ..
    }) = store.kind(result)
    else {
        return Err("expected function");
    };
    let shifted = reference(&mut store, RegionId::bound(1, 3)?, byte)?;
    let lowered = reference(&mut store, RegionId::bound(1, 7)?, byte)?;
    assert_eq!(
        store.list(parameters),
        Some([local, shifted, lowered].as_slice())
    );
    assert_eq!(output, shifted);
    let steps = substitution.steps;
    assert_eq!(substitution.apply(&mut store, function)?, result);
    assert_eq!(substitution.steps, steps);
    let mut other = Bound::new(vec![RegionId(17)]);
    assert_ne!(other.apply(&mut store, function)?, result);
    let instantiated = other.apply(&mut store, local)?;
    let Some(Kind::Reference { region, .. }) = store.kind(instantiated) else {
        return Err("expected reference");
    };
    assert_eq!(region, RegionId(17));
    Ok(())
}

#[test]
fn shared_deep_graphs_are_linear_and_ordinary_types_need_no_walk() -> Result<(), &'static str> {
    let mut store = Store::default();
    let mut ty = reference(&mut store, RegionId::bound(0, 0)?, TypeId::UNIT)?;
    for _ in 0..10_000 {
        let fields = store.intern_list(&[ty, ty])?;
        ty = store.intern(Kind::Tuple(fields))?;
    }
    let mut substitution = Bound::new(vec![RegionId::STATIC]);
    let result = substitution.apply(&mut store, ty)?;
    assert_eq!(store.escaping_binders(result), Some(0));
    assert!(substitution.steps < 40_010);
    let steps = substitution.steps;
    assert_eq!(substitution.apply(&mut store, result)?, result);
    assert_eq!(substitution.steps, steps);
    assert!(Bound::new(vec![]).apply(&mut store, ty).is_err());
    // Failure leaves no queued work contaminating another root.
    let mut invalid = Bound::new(vec![]);
    assert!(invalid.apply(&mut store, ty).is_err());
    assert_eq!(invalid.apply(&mut store, TypeId::UNIT)?, TypeId::UNIT);
    Ok(())
}

#[test]
fn nested_nominal_arguments_and_function_items_substitute_without_changing_owners()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let args = store.intern_regions(&[RegionId::bound(1, 0)?, RegionId::STATIC])?;
    let aggregate = store.intern(Kind::Adt(ItemId(33), args))?;
    let parameters = store.intern_list(&[aggregate])?;
    let signature = store.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: aggregate,
        unsafe_: false,
    })?;
    let item = store.intern(Kind::FunctionItem {
        arguments: rustgm::types::GenericArgs::EMPTY,
        item: ItemId(12),
        signature,
    })?;
    let transformed = Bound::new(vec![RegionId(19)]).apply(&mut store, item)?;
    let Some(Kind::FunctionItem {
        arguments: _,
        item: owner,
        signature,
    }) = store.kind(transformed)
    else {
        return Err("expected item");
    };
    assert_eq!(owner, ItemId(12));
    let Some(Kind::Function { result, .. }) = store.kind(signature) else {
        return Err("expected signature");
    };
    let Some(Kind::Adt(owner, args)) = store.kind(result) else {
        return Err("expected aggregate");
    };
    assert_eq!(owner, ItemId(33));
    assert_eq!(
        store.region_args(args),
        Some([RegionId(19), RegionId::STATIC].as_slice())
    );
    Ok(())
}

#[test]
fn substitution_limits_reject_before_materializing_unbounded_argument_work()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let free = RegionId::bound(0, 0)?;
    let arguments = store.intern_regions(&vec![free; 1_000_001])?;
    let nominal = store.intern(Kind::Adt(ItemId(0), arguments))?;
    let mut substitution = Bound::new(vec![RegionId::STATIC]);
    assert_eq!(
        substitution.apply(&mut store, nominal),
        Err("lifetime substitution work budget exceeded")
    );
    let outside = reference(&mut store, RegionId::bound(1, 0)?, TypeId::UNIT)?;
    let parameters = store.intern_list(&[outside])?;
    let nested = store.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: outside,
        unsafe_: false,
    })?;
    let mut overflowing = Bound::new(vec![RegionId::bound(0x7fff, 0)?]);
    assert_eq!(
        overflowing.apply(&mut store, nested),
        Err("region binder nesting exceeds 32767")
    );
    assert!(
        Bound::new(vec![])
            .apply(&mut store, TypeId(u32::MAX))
            .is_err()
    );
    Ok(())
}
