use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, ConstKind, Constant},
    parameters::{Parameter, ParameterKind},
    substitute::Bound,
};

fn reference(store: &mut Store, region: RegionId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region,
        referent: TypeId::UNIT,
    })
}

#[test]
fn explicit_parameter_ids_support_sparse_slots_and_multiple_owners() -> Result<(), &'static str> {
    let mut types = Store::default();
    let a = types.intern_parameter(Parameter {
        owner: ItemId(5),
        index: 3,
        kind: ParameterKind::Type { sized: true },
    })?;
    let b = types.intern_parameter(Parameter {
        owner: ItemId(9),
        index: 7,
        kind: ParameterKind::Type { sized: true },
    })?;
    let n = types.intern_parameter(Parameter {
        owner: ItemId(5),
        index: 11,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let a_ty = types.intern(Kind::Parameter(a))?;
    let b_ty = types.intern(Kind::Parameter(b))?;
    let n_const = types.intern_const(ConstKind::Parameter(n))?;
    let array = types.intern(Kind::ArrayTerm {
        element: a_ty,
        length: n_const,
    })?;
    let list = types.intern_list(&[array, b_ty])?;
    let root = types.intern(Kind::Tuple(list))?;
    let length = types.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 7,
    })?;
    let bindings = [
        (b, Argument::Type(a_ty)),
        (n, Argument::Const(length)),
        (a, Argument::Type(b_ty)),
    ];
    let mut bound = Bound::parameters(&types, &bindings)?;
    let result = bound.apply(&mut types, root)?;
    let expected_array = types.intern(Kind::Array {
        element: b_ty,
        length: 7,
    })?;
    let expected = types.intern_list(&[expected_array, a_ty])?;
    assert_eq!(types.kind(result), Some(Kind::Tuple(expected)));
    let steps = bound.steps;
    assert_eq!(bound.apply(&mut types, root)?, result);
    assert_eq!(bound.steps, steps);
    let count = types.len();
    assert!(
        Bound::parameters(
            &types,
            &[(a, Argument::Type(a_ty)), (a, Argument::Type(b_ty))]
        )
        .is_err()
    );
    assert!(Bound::parameters(&types, &[(n, Argument::Type(TypeId::UNIT))]).is_err());
    assert!(Bound::parameters(&types, &[(a, Argument::Const(length))]).is_err());
    assert_eq!(count, types.len());
    Ok(())
}

#[test]
fn argument_universals_preserve_nested_binders_static_and_repeated_free_regions()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let caller = reference(&mut types, RegionId(17))?;
    let caller_bound = reference(&mut types, RegionId::bound(0, 9)?)?;
    let static_ = reference(&mut types, RegionId::STATIC)?;
    let local = reference(&mut types, RegionId::bound(0, 0)?)?;
    let captured = reference(&mut types, RegionId::bound(1, 9)?)?;
    let parameters = types.intern_list(&[local, caller, captured, static_])?;
    let function = types.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: captured,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let mut bound = Bound::universals(4);
    let transformed = bound.apply(&mut types, caller)?;
    assert_eq!(transformed, reference(&mut types, RegionId::bound(0, 4)?)?);
    assert_eq!(
        bound.apply(&mut types, caller_bound)?,
        reference(&mut types, RegionId::bound(0, 5)?)?
    );
    let result = bound.apply(&mut types, function)?;
    let caller_nested = reference(&mut types, RegionId::bound(1, 4)?)?;
    let captured_nested = reference(&mut types, RegionId::bound(1, 5)?)?;
    let parameters = types.intern_list(&[local, caller_nested, captured_nested, static_])?;
    assert_eq!(
        types.kind(result),
        Some(Kind::Function {
            bound_regions: 1,
            parameters,
            result: captured_nested,
            unsafe_: false,
            convention: rustgm::types::callable::Convention::RUST,
        })
    );
    assert_eq!(bound.universal_count(), 2);
    let steps = bound.steps;
    assert_eq!(bound.apply(&mut types, function)?, result);
    assert_eq!(steps, bound.steps);
    assert!(Bound::universals(65_536).apply(&mut types, caller).is_err());
    assert_eq!(
        Bound::universals(65_536).apply(&mut types, static_)?,
        static_
    );
    Ok(())
}

#[test]
fn mapped_types_shift_under_callable_binders_without_replacing_inserted_parameters()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let id = types.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 8,
        kind: ParameterKind::Type { sized: true },
    })?;
    let parameter = types.intern(Kind::Parameter(id))?;
    let local = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: parameter,
    })?;
    let early = reference(&mut types, RegionId::bound(1, 1)?)?;
    let parameters = types.intern_list(&[local, early])?;
    let function = types.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: parameter,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let argument = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 7)?,
        referent: parameter,
    })?;
    let mut bound = Bound::parameters_captured(
        &types,
        &[(id, Argument::Type(argument))],
        vec![RegionId::STATIC, RegionId(23)],
    )?;
    let result = bound.apply(&mut types, function)?;
    let argument = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(1, 7)?,
        referent: parameter,
    })?;
    let local = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: argument,
    })?;
    let early = reference(&mut types, RegionId(23))?;
    let parameters = types.intern_list(&[local, early])?;
    assert_eq!(
        types.kind(result),
        Some(Kind::Function {
            bound_regions: 1,
            parameters,
            result: argument,
            unsafe_: false,
            convention: rustgm::types::callable::Convention::RUST,
        })
    );
    Ok(())
}
