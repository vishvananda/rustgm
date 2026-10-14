use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, ConstKind, Constant},
    parameters::{Parameter, ParameterKind},
    substitute::Bound,
};

fn parameter(store: &mut Store, owner: ItemId, index: u32) -> Result<TypeId, &'static str> {
    let id = store.intern_parameter(Parameter {
        owner,
        index,
        kind: ParameterKind::Type { sized: true },
    })?;
    store.intern(Kind::Parameter(id))
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
fn joint_substitution_does_not_capture_inserted_types_or_reprocess_their_regions()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let owner = ItemId(4);
    let t = parameter(&mut store, owner, 1)?;
    let byte = TypeId::primitive(Primitive::U8);
    let local = reference(&mut store, RegionId::bound(0, 0)?, t)?;
    let early = reference(&mut store, RegionId::bound(1, 0)?, t)?;
    let outer = reference(&mut store, RegionId::bound(2, 7)?, t)?;
    let parameters = store.intern_list(&[local, early, outer])?;
    let function = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: t,
        unsafe_: false,
    })?;
    let argument = reference(&mut store, RegionId::bound(0, 3)?, byte)?;
    let args = store.intern_arguments(&[
        Argument::Lifetime(RegionId::bound(0, 5)?),
        Argument::Type(argument),
    ])?;
    let mut substitution = Bound::joint(&store, owner, args)?;
    let result = substitution.apply(&mut store, function)?;
    let argument = reference(&mut store, RegionId::bound(1, 3)?, byte)?;
    let local = reference(&mut store, RegionId::bound(0, 0)?, argument)?;
    let early = reference(&mut store, RegionId::bound(1, 5)?, argument)?;
    let outer = reference(&mut store, RegionId::bound(1, 7)?, argument)?;
    let parameters = store.intern_list(&[local, early, outer])?;
    let expected = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: argument,
        unsafe_: false,
    })?;
    assert_eq!(result, expected);
    let steps = substitution.steps;
    assert_eq!(substitution.apply(&mut store, function)?, result);
    assert_eq!(substitution.steps, steps);
    assert_eq!(store.contains_parameters(result), Some(false));
    Ok(())
}

#[test]
fn parameter_replacement_is_simultaneous_and_owner_scoped() -> Result<(), &'static str> {
    let mut store = Store::default();
    let owner = ItemId(2);
    let a = parameter(&mut store, owner, 0)?;
    let b = parameter(&mut store, owner, 1)?;
    let foreign = parameter(&mut store, ItemId(3), 0)?;
    let fields = store.intern_list(&[a, b, foreign])?;
    let tuple = store.intern(Kind::Tuple(fields))?;
    let args = store.intern_arguments(&[Argument::Type(b), Argument::Type(a)])?;
    let result = Bound::joint(&store, owner, args)?.apply(&mut store, tuple)?;
    let fields = store.intern_list(&[b, a, foreign])?;
    assert_eq!(store.kind(result), Some(Kind::Tuple(fields)));
    let args = store.intern_arguments(&[Argument::Type(TypeId::UNIT)])?;
    assert_eq!(
        Bound::joint(&store, ItemId(3), args)?.apply(&mut store, foreign)?,
        TypeId::UNIT
    );
    Ok(())
}

#[test]
fn const_substitution_normalizes_symbolic_arrays_and_preserves_type_identity()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let owner = ItemId(1);
    let element = parameter(&mut store, owner, 0)?;
    let usize_ = TypeId::primitive(Primitive::Usize);
    let n = store.intern_parameter(Parameter {
        owner,
        index: 1,
        kind: ParameterKind::Const { ty: usize_ },
    })?;
    let length = store.intern_const(ConstKind::Parameter(n))?;
    let array = store.intern(Kind::ArrayTerm { element, length })?;
    let concrete = store.intern_constant(Constant {
        ty: usize_,
        bits: 37,
    })?;
    let args = store.intern_arguments(&[
        Argument::Type(TypeId::primitive(Primitive::U16)),
        Argument::Const(concrete),
    ])?;
    let mut substitution = Bound::joint(&store, owner, args)?;
    let result = substitution.apply(&mut store, array)?;
    let expected = store.intern(Kind::Array {
        element: TypeId::primitive(Primitive::U16),
        length: 37,
    })?;
    assert_eq!(result, expected);
    assert_eq!(store.contains_parameters(result), Some(false));
    assert_eq!(
        store.intern(Kind::ArrayTerm {
            element: TypeId::primitive(Primitive::U16),
            length: concrete
        })?,
        expected
    );
    let nominal_args =
        store.intern_arguments(&[Argument::Type(element), Argument::Const(length)])?;
    let nominal = store.intern(Kind::Adt(ItemId(9), nominal_args))?;
    let result = substitution.apply(&mut store, nominal)?;
    assert_eq!(store.kind(result), Some(Kind::Adt(ItemId(9), args)));
    let bad = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::U8),
        bits: 37,
    })?;
    assert!(
        store
            .intern(Kind::ArrayTerm {
                element,
                length: bad
            })
            .is_err()
    );
    let args = store.intern_arguments(&[Argument::Type(TypeId::UNIT), Argument::Const(bad)])?;
    assert!(
        Bound::joint(&store, owner, args)?
            .apply(&mut store, array)
            .is_err()
    );
    Ok(())
}

#[test]
fn inserted_callable_binders_stay_local_and_shifted_arguments_share_work()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let owner = ItemId(1);
    let a = parameter(&mut store, owner, 0)?;
    let b = parameter(&mut store, owner, 1)?;
    let byte = TypeId::primitive(Primitive::U8);
    let own = reference(&mut store, RegionId::bound(0, 0)?, byte)?;
    let outer = reference(&mut store, RegionId::bound(1, 3)?, byte)?;
    let parameters = store.intern_list(&[own, outer])?;
    let argument = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: outer,
        unsafe_: false,
    })?;
    let parameters = store.intern_list(&[a, b])?;
    let template = store.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: a,
        unsafe_: false,
    })?;
    let args = store.intern_arguments(&[Argument::Type(argument), Argument::Type(argument)])?;
    let result = Bound::joint(&store, owner, args)?.apply(&mut store, template)?;
    let outer = reference(&mut store, RegionId::bound(2, 3)?, byte)?;
    let parameters = store.intern_list(&[own, outer])?;
    let shifted = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: outer,
        unsafe_: false,
    })?;
    let parameters = store.intern_list(&[shifted, shifted])?;
    assert_eq!(
        store.kind(result),
        Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result: shifted,
            unsafe_: false
        })
    );
    Ok(())
}

#[test]
fn missing_arguments_and_binder_overflow_fail_without_fabricating_results()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let owner = ItemId(1);
    let t = parameter(&mut store, owner, 0)?;
    assert!(
        Bound::joint(&store, owner, GenericArgs::EMPTY)?
            .apply(&mut store, t)
            .is_err()
    );
    let wrong = store.intern_regions(&[RegionId::STATIC])?;
    assert!(
        Bound::joint(&store, owner, wrong)?
            .apply(&mut store, t)
            .is_err()
    );
    let argument = reference(&mut store, RegionId::bound(0x7fff, 0)?, TypeId::UNIT)?;
    let parameters = store.intern_list(&[t])?;
    let function = store.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
    })?;
    let args = store.intern_arguments(&[Argument::Type(argument)])?;
    assert!(
        Bound::joint(&store, owner, args)?
            .apply(&mut store, function)
            .is_err()
    );
    Ok(())
}

#[test]
fn generic_substitution_is_linear_in_shared_structural_nodes() -> Result<(), &'static str> {
    let mut store = Store::default();
    let owner = ItemId(1);
    let mut root = parameter(&mut store, owner, 0)?;
    for _ in 0..4096 {
        let children = store.intern_list(&[root, root])?;
        root = store.intern(Kind::Tuple(children))?;
    }
    let args = store.intern_arguments(&[Argument::Type(TypeId::primitive(Primitive::U8))])?;
    let mut substitution = Bound::joint(&store, owner, args)?;
    let result = substitution.apply(&mut store, root)?;
    assert_eq!(store.contains_parameters(result), Some(false));
    assert!(substitution.steps < 4096 * 5);
    let steps = substitution.steps;
    assert_eq!(substitution.apply(&mut store, root)?, result);
    assert_eq!(substitution.steps, steps);
    Ok(())
}
