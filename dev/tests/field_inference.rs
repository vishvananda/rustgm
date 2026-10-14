use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{ConstKind, Constant},
    infer::{Context, Domain, Error, Instantiation, arguments::Argument},
    parameters::{Parameter, ParameterKind},
};

#[test]
fn field_templates_share_unresolved_arguments_and_completed_subgraphs()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let parameter = types.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let template = types.intern(Kind::Parameter(parameter))?;
    let mut context = Context::default();
    let argument = context.variable(Domain::Any)?;
    let mut substitution = Instantiation::new(
        &mut context,
        &types,
        &[(parameter, Argument::Type(argument))],
    )?;
    let mut nested = template;
    for _ in 0..4096 {
        let list = types.intern_list(&[nested, template])?;
        nested = types.intern(Kind::Tuple(list))?;
        let term = context.instantiate_with(&mut types, nested, &mut substitution)?;
        assert_eq!(context.child(&types, term, 1)?, argument);
        let before = context.steps;
        assert_eq!(
            context.instantiate_with(&mut types, nested, &mut substitution)?,
            term
        );
        assert!(context.steps - before <= 2);
    }
    assert!(context.steps < 40_000, "{}", context.steps);
    assert_eq!(
        context.resolved(&mut types, argument),
        Err(Error::Unresolved)
    );
    let byte = context.known(&types, TypeId::primitive(Primitive::U8))?;
    context.equate(&types, argument, byte)?;
    let term = context.instantiate_with(&mut types, template, &mut substitution)?;
    assert_eq!(
        context.resolved(&mut types, term)?,
        TypeId::primitive(Primitive::U8)
    );
    assert_eq!(types.kind(template), Some(Kind::Parameter(parameter)));
    Ok(())
}

#[test]
fn item_arguments_preserve_unsolved_type_and_const_slots() -> Result<(), Box<dyn std::error::Error>>
{
    let mut types = Store::default();
    let size = TypeId::primitive(Primitive::Usize);
    let parameter = types.intern_parameter(Parameter {
        owner: ItemId(2),
        index: 2,
        kind: ParameterKind::Const { ty: size },
    })?;
    let constant = types.intern_const(ConstKind::Parameter(parameter))?;
    let mut context = Context::default();
    let length = context.const_variable(&types, size)?;
    let ty = context.variable(Domain::Any)?;
    let arguments = [
        Argument::Lifetime(RegionId(3)),
        Argument::Type(ty),
        Argument::Const(length),
    ];
    let receiver = context.nominal(ItemId(2), &arguments)?;
    assert_eq!(context.item_arguments(&types, receiver)?, arguments);
    assert_eq!(
        context.resolved(&mut types, receiver),
        Err(Error::Unresolved)
    );
    let substitution = Instantiation::new(
        &mut context,
        &types,
        &[(parameter, Argument::Const(length))],
    )?;
    assert_eq!(
        context.instantiate_const(&types, constant, &substitution)?,
        length
    );
    let three = types.intern_constant(Constant { ty: size, bits: 3 })?;
    let three_term = context.known_const(&types, three)?;
    context.equate_consts(length, three_term)?;
    assert_eq!(context.resolved_const(&mut types, length)?, three);
    assert!(Instantiation::new(&mut context, &types, &[(parameter, Argument::Type(ty))]).is_err());
    assert!(
        Instantiation::new(
            &mut context,
            &types,
            &[(parameter, Argument::Const(length)); 2]
        )
        .is_err()
    );
    assert_eq!(
        context.item_arguments(&types, ty),
        Err(Error::InvalidIdentity)
    );
    Ok(())
}

#[test]
fn parameter_insertion_shifts_only_lifetimes_outside_the_argument()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let parameter = types.intern_parameter(Parameter {
        owner: ItemId(9),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let formal = types.intern(Kind::Parameter(parameter))?;
    let parameters = types.intern_list(&[formal])?;
    let template = types.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: formal,
        unsafe_: false,
    })?;
    let reference = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let parameters = types.intern_list(&[reference])?;
    let inner = types.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: reference,
        unsafe_: false,
    })?;
    let elements = types.intern_list(&[reference, inner])?;
    let argument = types.intern(Kind::Tuple(elements))?;
    let shifted = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(1, 0)?,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let elements = types.intern_list(&[shifted, inner])?;
    let shifted = types.intern(Kind::Tuple(elements))?;
    let parameters = types.intern_list(&[shifted])?;
    let expected = types.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: shifted,
        unsafe_: false,
    })?;
    let mut context = Context::default();
    let argument = context.known(&types, argument)?;
    let inferred = context.instantiate(
        &mut types,
        template,
        &[(parameter, Argument::Type(argument))],
    )?;
    assert_eq!(context.resolved(&mut types, inferred)?, expected);
    Ok(())
}
