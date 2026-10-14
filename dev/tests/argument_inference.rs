use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument as Canonical, ConstKind, Constant},
    infer::{Constructor, Context, Domain, Error, arguments::Argument, constants::ConstTermId},
    parameters::{Parameter, ParameterKind},
};

fn signature(types: &mut Store, result: TypeId) -> Result<TypeId, &'static str> {
    let parameters = types.intern_list(&[])?;
    types.intern(Kind::Function {
        parameters,
        result,
        bound_regions: 0,
        unsafe_: false,
    })
}

#[test]
fn const_variables_are_typed_and_never_use_numeric_fallback()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let usize_ = TypeId::primitive(Primitive::Usize);
    let byte = TypeId::primitive(Primitive::U8);
    let mut context = Context::default();
    let a = context.const_variable(&types, usize_)?;
    let b = context.const_variable(&types, usize_)?;
    assert_eq!(
        context.resolved_const(&mut types, a),
        Err(Error::Unresolved)
    );
    context.equate_consts(a, b)?;
    let four = types.intern_constant(Constant {
        ty: usize_,
        bits: 4,
    })?;
    let value = context.known_const(&types, four)?;
    context.equate_consts(b, value)?;
    assert_eq!(context.resolved_const(&mut types, a)?, four);
    let wrong = context.const_variable(&types, byte)?;
    assert_eq!(context.equate_consts(wrong, a), Err(Error::Mismatch));
    let five = types.intern_constant(Constant {
        ty: usize_,
        bits: 5,
    })?;
    let five = context.known_const(&types, five)?;
    assert_eq!(context.equate_consts(a, five), Err(Error::Mismatch));
    assert!(
        context
            .const_variable(&types, TypeId::primitive(Primitive::F32))
            .is_err()
    );
    assert!(
        context
            .resolved_const(&mut types, ConstTermId(u32::MAX))
            .is_err()
    );
    Ok(())
}

#[test]
fn array_shapes_infer_lengths_and_preserve_rigid_const_parameters()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let usize_ = TypeId::primitive(Primitive::Usize);
    let concrete = types.intern(Kind::Array {
        element: byte,
        length: 7,
    })?;
    let mut context = Context::default();
    let ty = context.variable(Domain::Any)?;
    let length = context.const_variable(&types, usize_)?;
    let array = context.construct(Constructor::ArrayInfer(length), &[ty])?;
    let known = context.known(&types, concrete)?;
    context.equate(&types, array, known)?;
    assert_eq!(context.resolved(&mut types, array)?, concrete);
    let id = context.resolved_const(&mut types, length)?;
    assert_eq!(
        types.constant(id),
        Some(Constant {
            ty: usize_,
            bits: 7
        })
    );
    let p = types.intern_parameter(Parameter {
        owner: ItemId(5),
        index: 0,
        kind: ParameterKind::Const { ty: usize_ },
    })?;
    let p = types.intern_const(ConstKind::Parameter(p))?;
    let symbolic = types.intern(Kind::ArrayTerm {
        element: byte,
        length: p,
    })?;
    let length = context.const_variable(&types, usize_)?;
    let inferred = context.construct(Constructor::ArrayInfer(length), &[ty])?;
    let known = context.known(&types, symbolic)?;
    context.equate(&types, inferred, known)?;
    assert_eq!(context.resolved(&mut types, inferred)?, symbolic);
    let concrete = context.known(&types, concrete)?;
    assert_eq!(
        context.equate(&types, inferred, concrete),
        Err(Error::Mismatch)
    );
    Ok(())
}

#[test]
fn erased_parameters_remain_in_function_item_identity() -> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let sig = signature(&mut types, TypeId::UNIT)?;
    let mut context = Context::default();
    let signature = context.known(&types, sig)?;
    let byte = context.known(&types, TypeId::primitive(Primitive::U8))?;
    let word = context.known(&types, TypeId::primitive(Primitive::U16))?;
    let a = context.function_item(ItemId(3), signature, &[Argument::Type(byte)])?;
    let b = context.function_item(ItemId(3), signature, &[Argument::Type(word)])?;
    assert_eq!(context.equate(&types, a, b), Err(Error::Mismatch));
    let a = context.resolved(&mut types, a)?;
    let b = context.resolved(&mut types, b)?;
    assert_ne!(a, b);
    let Some(Kind::FunctionItem {
        arguments,
        signature,
        ..
    }) = types.kind(a)
    else {
        return Err("missing item".into());
    };
    assert_eq!(signature, sig);
    assert_eq!(
        types
            .arguments(arguments)
            .ok_or("missing arguments")?
            .collect::<Vec<_>>(),
        [Canonical::Type(TypeId::primitive(Primitive::U8))]
    );
    let mut context = Context::default();
    let signature = context.known(&types, sig)?;
    let unknown = context.variable(Domain::Any)?;
    let item = context.function_item(ItemId(3), signature, &[Argument::Type(unknown)])?;
    assert_eq!(context.resolved(&mut types, item), Err(Error::Unresolved));
    assert!(matches!(context.finish(&mut types), Err(Error::Unresolved)));
    Ok(())
}

#[test]
fn mixed_item_arguments_unify_with_completed_canonical_items()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let sig = signature(&mut types, TypeId::UNIT)?;
    let count = types.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 9,
    })?;
    let args = types.intern_arguments(&[
        Canonical::Lifetime(RegionId(1)),
        Canonical::Const(count),
        Canonical::Type(TypeId::primitive(Primitive::U8)),
    ])?;
    let item = types.intern(Kind::FunctionItem {
        item: ItemId(3),
        signature: sig,
        arguments: args,
    })?;
    let mut context = Context::default();
    let signature = context.known(&types, sig)?;
    let length = context.const_variable(&types, TypeId::primitive(Primitive::Usize))?;
    let ty = context.variable(Domain::Any)?;
    let inferred = context.function_item(
        ItemId(3),
        signature,
        &[
            Argument::Lifetime(RegionId(2)),
            Argument::Const(length),
            Argument::Type(ty),
        ],
    )?;
    let known = context.known(&types, item)?;
    context.equate(&types, inferred, known)?;
    assert_eq!(context.resolved_const(&mut types, length)?, count);
    let solved = context.finish(&mut types)?;
    assert_eq!(
        solved.types[ty.0 as usize],
        TypeId::primitive(Primitive::U8)
    );
    assert!(
        solved
            .region_equalities
            .contains(&(RegionId(2), RegionId(1)))
    );
    Ok(())
}

#[test]
fn occurs_check_includes_generic_arguments_absent_from_signatures()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let sig = signature(&mut types, TypeId::UNIT)?;
    let mut context = Context::default();
    let signature = context.known(&types, sig)?;
    let variable = context.variable(Domain::Any)?;
    let item = context.function_item(ItemId(0), signature, &[Argument::Type(variable)])?;
    assert_eq!(context.equate(&types, variable, item), Err(Error::Occurs));
    let variable = context.variable(Domain::Any)?;
    let nominal = context.nominal(ItemId(0), &[Argument::Type(variable)])?;
    assert_eq!(
        context.equate(&types, variable, nominal),
        Err(Error::Occurs)
    );
    Ok(())
}

#[test]
fn explicit_empty_argument_lists_match_closed_items() -> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let sig = signature(&mut types, TypeId::UNIT)?;
    let canonical = types.intern(Kind::FunctionItem {
        item: ItemId(7),
        signature: sig,
        arguments: GenericArgs::EMPTY,
    })?;
    let mut context = Context::default();
    let signature = context.known(&types, sig)?;
    let inferred = context.function_item(ItemId(7), signature, &[])?;
    let known = context.known(&types, canonical)?;
    assert!(!context.incompatible(&types, inferred, known)?);
    context.equate(&types, inferred, known)?;
    assert_eq!(context.resolved(&mut types, inferred)?, canonical);
    Ok(())
}

#[test]
fn inspecting_compatibility_does_not_bind_type_or_const_variables()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let mut context = Context::default();
    let ty = context.variable(Domain::Any)?;
    let length = context.const_variable(&types, TypeId::primitive(Primitive::Usize))?;
    let inferred = context.construct(Constructor::ArrayInfer(length), &[ty])?;
    for (element, length) in [(Primitive::U8, 3), (Primitive::U16, 4)] {
        let known = types.intern(Kind::Array {
            element: TypeId::primitive(element),
            length,
        })?;
        let known = context.known(&types, known)?;
        assert!(!context.incompatible(&types, inferred, known)?);
    }
    assert_eq!(context.resolved(&mut types, ty), Err(Error::Unresolved));
    assert_eq!(
        context.resolved_const(&mut types, length),
        Err(Error::Unresolved)
    );
    Ok(())
}

#[test]
fn typed_template_instantiation_shares_constraints_without_mutating_the_template()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let t = types.intern_parameter(Parameter {
        owner: ItemId(8),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let n = types.intern_parameter(Parameter {
        owner: ItemId(8),
        index: 1,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let ty = types.intern(Kind::Parameter(t))?;
    let length = types.intern_const(ConstKind::Parameter(n))?;
    let array = types.intern(Kind::ArrayTerm {
        element: ty,
        length,
    })?;
    let list = types.intern_list(&[array, array])?;
    let template = types.intern(Kind::Tuple(list))?;
    let mut context = Context::default();
    for primitive in [Primitive::U8, Primitive::U16] {
        let type_argument = context.variable(Domain::Any)?;
        let const_argument = context.const_variable(&types, TypeId::primitive(Primitive::Usize))?;
        let term = context.instantiate(
            &mut types,
            template,
            &[
                (t, Argument::Type(type_argument)),
                (n, Argument::Const(const_argument)),
            ],
        )?;
        let expected = types.intern(Kind::Array {
            element: TypeId::primitive(primitive),
            length: 3,
        })?;
        let known = context.known(&types, expected)?;
        let first = context.child(&types, term, 0)?;
        context.equate(&types, first, known)?;
        let second = context.child(&types, term, 1)?;
        assert_eq!(context.resolved(&mut types, second)?, expected);
    }
    assert_eq!(
        types.kind(array),
        Some(Kind::ArrayTerm {
            element: ty,
            length
        })
    );
    Ok(())
}
