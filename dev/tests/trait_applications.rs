//! Trait arguments remain part of identity, binder substitution and inference obligations.
#[path = "trait_applications/binders.rs"]
mod binders;
#[path = "trait_applications/equations.rs"]
mod equations;
#[path = "trait_applications/impl_equations.rs"]
mod impl_equations;
use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, ConstKind, Constant},
    bound_uses,
    infer::{self, Context, Domain, Instantiation},
    parameters::{
        Parameter, ParameterId, ParameterKind,
        environment::{Abilities, Environment},
        predicates::{Bound, Predicate},
        uses::Uses,
    },
    substitute,
    traits::{Application, ApplicationId},
};

fn application(
    store: &mut Store,
    item: u32,
    arguments: &[Argument],
) -> Result<ApplicationId, &'static str> {
    let arguments = store.intern_arguments(arguments)?;
    store.intern_trait_application(Application {
        item: ItemId(item),
        arguments,
    })
}
fn parameter(
    store: &mut Store,
    index: u32,
    kind: ParameterKind,
) -> Result<ParameterId, &'static str> {
    store.intern_parameter(Parameter {
        owner: ItemId(100),
        index,
        kind,
    })
}
fn reference(
    store: &mut Store,
    referent: TypeId,
    region: RegionId,
) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        referent,
        region,
        mutable: false,
    })
}

#[test]
fn canonical_trait_identity_retains_argument_kinds_order_and_values()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let value = store.intern_constant(Constant { ty: byte, bits: 7 })?;
    let args = [Argument::Type(byte), Argument::Const(value)];
    let first = application(&mut store, 200, &args)?;
    assert_eq!(first, application(&mut store, 200, &args)?);
    assert_ne!(first, application(&mut store, 201, &args)?);
    assert_ne!(first, application(&mut store, 200, &[args[1], args[0]])?);
    let other = store.intern_constant(Constant { ty: byte, bits: 9 })?;
    assert_ne!(
        first,
        application(&mut store, 200, &[args[0], Argument::Const(other)])?
    );
    let a = application(&mut store, 200, &[Argument::Lifetime(RegionId::STATIC)])?;
    let b = application(
        &mut store,
        200,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    assert_ne!(a, b);
    assert!(
        store
            .intern_trait_application(Application {
                item: ItemId(200),
                arguments: GenericArgs(u32::MAX)
            })
            .is_err()
    );
    assert!(store.trait_application(ApplicationId(u32::MAX)).is_none());
    assert!(std::mem::size_of::<Bound>() <= 8);
    assert!(std::mem::size_of::<Predicate>() <= 16);
    Ok(())
}

#[test]
fn predicate_substitution_preserves_nested_binders_and_const_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let index = TypeId::primitive(Primitive::Usize);
    let t = parameter(&mut store, 1, ParameterKind::Type { sized: true })?;
    let n = parameter(&mut store, 2, ParameterKind::Const { ty: index })?;
    let ty = store.intern(Kind::Parameter(t))?;
    let length = store.intern_const(ConstKind::Parameter(n))?;
    let inner = reference(&mut store, byte, RegionId::bound(0, 0)?)?;
    let parameters = store.intern_list(&[ty, inner])?;
    let function = store.intern(Kind::Function {
        parameters,
        result: ty,
        unsafe_: false,
        bound_regions: 1,
    })?;
    let trait_ = application(
        &mut store,
        200,
        &[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Type(function),
            Argument::Const(length),
        ],
    )?;
    let predicate = Predicate {
        ty,
        bound: Bound::Trait(trait_),
    };
    let region = RegionId::bound(0, 9)?;
    let inserted = reference(&mut store, byte, region)?;
    let constant = store.intern_constant(Constant { ty: index, bits: 3 })?;
    let mut fold = substitute::Bound::parameters_captured(
        &store,
        &[
            (t, Argument::Type(inserted)),
            (n, Argument::Const(constant)),
        ],
        vec![region],
    )?;
    let actual = fold.predicate(&mut store, predicate)?;
    assert_eq!(actual.ty, inserted);
    let Bound::Trait(id) = actual.bound else {
        return Err("missing substituted trait".into());
    };
    let args = store
        .arguments(
            store
                .trait_application(id)
                .ok_or("missing application")?
                .arguments,
        )
        .ok_or("missing arguments")?
        .collect::<Vec<_>>();
    assert_eq!(args[0], Argument::Lifetime(region));
    assert_eq!(args[2], Argument::Const(constant));
    let Argument::Type(function) = args[1] else {
        return Err("missing callable argument".into());
    };
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions: 1,
        ..
    }) = store.kind(function)
    else {
        return Err("lost callable binder".into());
    };
    let parameters = store
        .list(parameters)
        .ok_or("missing callable parameters")?;
    assert_eq!(parameters[1], inner);
    assert_eq!(parameters[0], result);
    assert!(
        matches!(store.kind(result), Some(Kind::Reference { region: actual, .. }) if actual == RegionId::bound(1, 9)?)
    );
    let mut uses = bound_uses::Uses::default();
    uses.predicate(&store, actual)?;
    assert_eq!(uses.indices.into_iter().collect::<Vec<_>>(), vec![9]);
    assert!(!actual.contains_parameters(&store)?);
    assert_eq!(actual.escaping_binders(&store)?, 1);
    Ok(())
}

#[test]
fn traits_do_not_invent_builtin_permissions_or_hide_foreign_parameters()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let t = parameter(&mut store, 0, ParameterKind::Type { sized: false })?;
    let u = parameter(&mut store, 1, ParameterKind::Type { sized: true })?;
    let n = parameter(
        &mut store,
        2,
        ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    )?;
    let ty = store.intern(Kind::Parameter(t))?;
    let argument = store.intern(Kind::Parameter(u))?;
    let constant = store.intern_const(ConstKind::Parameter(n))?;
    let trait_ = application(
        &mut store,
        200,
        &[Argument::Type(argument), Argument::Const(constant)],
    )?;
    let predicate = Predicate {
        ty,
        bound: Bound::Trait(trait_),
    };
    let missing = Environment::new(&store, &[(t, Abilities::default())])?;
    assert!(
        missing
            .with_const_parameters(&store, &[n])?
            .with_predicates(&store, &[predicate])
            .is_err()
    );
    let base = Environment::new(
        &store,
        &[(t, Abilities::default()), (u, Abilities::default())],
    )?;
    assert!(base.clone().with_predicates(&store, &[predicate]).is_err());
    let complete = base
        .clone()
        .with_const_parameters(&store, &[n])?
        .with_predicates(&store, &[predicate, predicate])?;
    assert!(complete.assumes_trait(ty, trait_));
    assert!(!base.assumes_trait(ty, trait_));
    assert_eq!(complete.trait_predicates().count(), 1);
    assert_eq!(complete.assumed(ty), Abilities::default());
    assert!(!complete.assumes_sized(ty));
    assert!(!complete.has_shape_assumptions());
    assert!(base.with_const_parameters(&store, &[t]).is_err());
    let mut uses = Uses::default();
    uses.predicate(&store, predicate)?;
    assert_eq!(uses.parameters.len(), 2);
    assert!(uses.constants.contains(&n));
    Ok(())
}

#[test]
fn repeated_wide_trait_arguments_share_substitution_and_occurrence_work()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let t = parameter(&mut store, 0, ParameterKind::Type { sized: true })?;
    let ty = store.intern(Kind::Parameter(t))?;
    let trait_ = application(&mut store, 200, &vec![Argument::Type(ty); 2048])?;
    let predicate = Predicate {
        ty: TypeId::UNIT,
        bound: Bound::Trait(trait_),
    };
    let mut fold = substitute::Bound::parameter_only(
        &store,
        t,
        Argument::Type(TypeId::primitive(Primitive::U8)),
    )?;
    let actual = fold.predicate(&mut store, predicate)?;
    let steps = fold.steps;
    let mut uses = Uses::default();
    uses.predicate(&store, predicate)?;
    let initial_steps = uses.steps;
    for _ in 0..1024 {
        assert_eq!(fold.predicate(&mut store, predicate)?, actual);
        uses.predicate(&store, predicate)?;
    }
    assert_eq!(fold.steps, steps);
    assert_eq!(uses.steps, initial_steps + 1024);
    assert_eq!(uses.parameters.len(), 1);
    let mut exhausted = substitute::Bound::new(Vec::new());
    exhausted.steps = 1_000_000;
    let empty = application(&mut store, 201, &[])?;
    assert!(exhausted.trait_application(&mut store, empty).is_err());
    assert!(
        substitute::Bound::new(Vec::new())
            .trait_application(&mut store, ApplicationId(u32::MAX))
            .is_err()
    );
    Ok(())
}

#[test]
fn inferred_predicates_resolve_type_and_const_equivalence_classes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let t = parameter(&mut store, 0, ParameterKind::Type { sized: true })?;
    let n = parameter(
        &mut store,
        1,
        ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    )?;
    let ty = store.intern(Kind::Parameter(t))?;
    let constant = store.intern_const(ConstKind::Parameter(n))?;
    let trait_ = application(
        &mut store,
        200,
        &[
            Argument::Type(ty),
            Argument::Type(ty),
            Argument::Const(constant),
        ],
    )?;
    let mut context = Context::default();
    let type_hole = context.variable(Domain::Any)?;
    let const_hole = context.const_variable(&store, TypeId::primitive(Primitive::Usize))?;
    let bindings = [
        (t, infer::arguments::Argument::Type(type_hole)),
        (n, infer::arguments::Argument::Const(const_hole)),
    ];
    let mut substitution = Instantiation::new(&mut context, &store, &bindings)?;
    let predicate = context.instantiate_predicate(
        &mut store,
        Predicate {
            ty,
            bound: Bound::Trait(trait_),
        },
        &mut substitution,
    )?;
    assert!(matches!(
        context.resolved_predicate(&mut store, &predicate),
        Err(infer::Error::Unresolved)
    ));
    let byte = TypeId::primitive(Primitive::U8);
    let actual = context.known(&store, byte)?;
    context.equate(&store, type_hole, actual)?;
    let value = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 3,
    })?;
    let actual = context.known_const(&store, value)?;
    context.equate_consts(const_hole, actual)?;
    let expected = application(
        &mut store,
        200,
        &[
            Argument::Type(byte),
            Argument::Type(byte),
            Argument::Const(value),
        ],
    )?;
    let direct = context.resolved_predicate(&mut store, &predicate)?;
    assert_eq!(
        direct,
        Predicate {
            ty: byte,
            bound: Bound::Trait(expected)
        }
    );
    let (_, solved) = context.finish_with_predicates(&mut store, std::iter::once(&predicate))?;
    assert_eq!(solved, vec![direct]);
    Ok(())
}

#[test]
fn constants_only_in_obligations_cannot_escape_inference_completion()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let index = TypeId::primitive(Primitive::Usize);
    let n = parameter(&mut store, 0, ParameterKind::Const { ty: index })?;
    let constant = store.intern_const(ConstKind::Parameter(n))?;
    let trait_ = application(&mut store, 200, &[Argument::Const(constant)])?;
    let mut context = Context::default();
    let variable = context.const_variable(&store, index)?;
    let mut substitution = Instantiation::new(
        &mut context,
        &store,
        &[(n, infer::arguments::Argument::Const(variable))],
    )?;
    let predicate = context.instantiate_predicate(
        &mut store,
        Predicate {
            ty: TypeId::UNIT,
            bound: Bound::Trait(trait_),
        },
        &mut substitution,
    )?;
    assert!(matches!(
        context.finish_with_predicates(&mut store, std::iter::once(&predicate)),
        Err(infer::Error::Unresolved)
    ));
    Ok(())
}
