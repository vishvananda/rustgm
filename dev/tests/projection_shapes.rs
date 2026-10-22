//! Canonical projections preserve inputs without granting properties of their unknown result.
use crate::qualify as rustc;
use rustgm::support::hash::HashMap;
use rustgm::{
    abi::layout::Layouts,
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
        aggregate::{self, Definitions, Repr, Variant},
        arguments::{Argument, ConstKind, Constant},
        bound_uses,
        copy::Copies,
        destruction::NeedsDrop,
        equality::Equality,
        infer::{Context, Error},
        outlives::LiveRegions,
        parameters::{self, Parameter, ParameterKind, environment::Environment, shape::Shapes},
        projection::{Application, Schema},
        substitute::Bound,
        traits,
        variance::{Variance, Variances},
    },
};
use std::sync::Arc;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn projected_input_occurrences_do_not_constrain_pointer_outputs() -> TestResult {
    let mut store = Store::default();
    schema(&mut store, 11, true)?;
    let direct = borrow(
        &mut store,
        RegionId::bound(0, 0)?,
        TypeId::primitive(Primitive::U8),
    )?;
    let kind = application(&mut store, 11, direct, &[], &[])?;
    let projected = store.intern(kind)?;
    let cases = [
        (
            vec![projected],
            direct,
            false,
            "fn(<&'a u8 as A>::Out)->&'a u8",
        ),
        (
            vec![projected],
            projected,
            false,
            "fn(<&'a u8 as A>::Out)-><&'a u8 as A>::Out",
        ),
        (vec![], projected, false, "fn()-><&'a u8 as A>::Out"),
        (
            vec![direct, projected],
            direct,
            true,
            "fn(&'a u8,<&'a u8 as A>::Out)->&'a u8",
        ),
        (
            vec![direct],
            projected,
            true,
            "fn(&'a u8)-><&'a u8 as A>::Out",
        ),
        (
            vec![projected],
            TypeId::UNIT,
            true,
            "fn(<&'a u8 as A>::Out)",
        ),
    ];
    for (index, (inputs, result, accepted, source)) in cases.into_iter().enumerate() {
        let source = format!(
            "trait A{{type Out;}}impl<'a>A for &'a u8{{type Out=&'a u8;}}type F=for<'a>{source};"
        );
        rustc::qualify(&format!("projection-binder-{index}"), &source, accepted)?;
        let ty = signature(&mut store, &inputs, result, 1)?;
        assert_eq!(bound_uses::check_pointer(&store, ty).is_ok(), accepted);
    }
    // Normalization would erase the distinction, so a caller must check the
    // unnormalized signature before publishing this otherwise valid normal form.
    let normalized = signature(&mut store, &[direct], direct, 1)?;
    bound_uses::check_pointer(&store, normalized)?;
    Ok(())
}

#[test]
fn opaque_pointer_inputs_still_require_valid_lifetime_slots() -> TestResult {
    let mut store = Store::default();
    schema(&mut store, 11, true)?;
    let kind = application(
        &mut store,
        11,
        TypeId::UNIT,
        &[Argument::Lifetime(RegionId::bound(0, 1)?)],
        &[],
    )?;
    let projected = store.intern(kind)?;
    let invalid = signature(&mut store, &[projected], TypeId::UNIT, 1)?;
    assert_eq!(
        bound_uses::check_pointer(&store, invalid),
        Err("function pointer references an unbound lifetime")
    );
    let valid = signature(&mut store, &[projected], TypeId::UNIT, 2)?;
    bound_uses::check_pointer(&store, valid)?;
    Ok(())
}

#[test]
fn projection_occurrences_do_not_constrain_impl_parameters() -> TestResult {
    let mut store = Store::default();
    schema(&mut store, 11, true)?;
    let a = parameter(&mut store, 20, 0)?;
    let b = parameter(&mut store, 20, 1)?;
    let c = store.intern_parameter(Parameter {
        owner: ItemId(20),
        index: 2,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let constant = store.intern_const(ConstKind::Parameter(c))?;
    let subject = borrow(&mut store, RegionId::bound(0, 0)?, a)?;
    let kind = application(
        &mut store,
        11,
        subject,
        &[Argument::Type(b)],
        &[
            Argument::Const(constant),
            Argument::Lifetime(RegionId::bound(0, 1)?),
        ],
    )?;
    let projected = store.intern(kind)?;
    let mut occurrences = parameters::uses::Uses::default();
    occurrences.collect(&store, projected)?;
    assert_eq!(occurrences.parameters.len(), 2);
    assert_eq!(occurrences.constants, [c].into_iter().collect());
    let mut constrained = parameters::uses::Uses::constraining();
    constrained.collect(&store, projected)?;
    assert!(constrained.parameters.is_empty() && constrained.constants.is_empty());
    let mut regions = bound_uses::Uses::default();
    regions.collect(&store, projected)?;
    assert_eq!(regions.indices, [0, 1].into_iter().collect());
    let mut fixed = bound_uses::Uses::constraining();
    fixed.collect(&store, projected)?;
    assert!(fixed.indices.is_empty());
    let direct = store.intern(Kind::ArrayTerm {
        element: a,
        length: constant,
    })?;
    let direct = borrow(&mut store, RegionId::bound(0, 1)?, direct)?;
    let list = store.intern_list(&[projected, direct])?;
    let root = store.intern(Kind::Tuple(list))?;
    constrained.collect(&store, root)?;
    assert_eq!(constrained.parameters.len(), 1);
    assert_eq!(constrained.constants, [c].into_iter().collect());
    fixed.collect(&store, root)?;
    assert_eq!(fixed.indices, [1].into_iter().collect());
    Ok(())
}

fn schema(store: &mut Store, item: u32, sized: bool) -> Result<(), &'static str> {
    store.declare_projection(
        ItemId(item),
        Schema {
            trait_item: ItemId(10),
            sized,
            copies: rustgm::types::copy::Properties::default(),
        },
    )
}
fn application(
    store: &mut Store,
    item: u32,
    subject: TypeId,
    trait_arguments: &[Argument],
    family_arguments: &[Argument],
) -> Result<Kind, &'static str> {
    let arguments = store.intern_arguments(trait_arguments)?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(10),
        arguments,
    })?;
    let arguments = store.intern_arguments(family_arguments)?;
    Ok(Kind::Projection(Application {
        item: ItemId(item),
        subject,
        trait_application,
        arguments,
    }))
}
fn parameter(store: &mut Store, owner: u32, index: u32) -> Result<TypeId, &'static str> {
    let id = store.intern_parameter(Parameter {
        owner: ItemId(owner),
        index,
        kind: ParameterKind::Type { sized: true },
    })?;
    store.intern(Kind::Parameter(id))
}
fn borrow(store: &mut Store, region: RegionId, referent: TypeId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region,
        referent,
    })
}
fn signature(
    store: &mut Store,
    parameters: &[TypeId],
    result: TypeId,
    bound_regions: u32,
) -> Result<TypeId, &'static str> {
    let parameters = store.intern_list(parameters)?;
    store.intern(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })
}

#[test]
fn associated_declarations_and_complete_applications_have_distinct_identities() -> TestResult {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let kind = application(&mut store, 11, byte, &[], &[])?;
    assert!(store.intern(kind).is_err());
    schema(&mut store, 11, true)?;
    schema(&mut store, 12, true)?;
    schema(&mut store, 13, false)?;
    let first = store.intern(kind)?;
    let count = store.len();
    for _ in 0..100 {
        schema(&mut store, 11, true)?;
        assert_eq!(store.intern(kind)?, first);
    }
    assert_eq!(store.len(), count);
    assert!(schema(&mut store, 11, false).is_err());
    assert!(
        store
            .declare_projection(
                ItemId(11),
                Schema {
                    trait_item: ItemId(99),
                    sized: true,
                    copies: rustgm::types::copy::Properties::default(),
                }
            )
            .is_err()
    );
    for (item, subject, trait_args, family_args) in [
        (12, byte, vec![], vec![]),
        (11, TypeId::UNIT, vec![], vec![]),
        (11, byte, vec![Argument::Type(byte)], vec![]),
        (11, byte, vec![], vec![Argument::Type(byte)]),
        (11, byte, vec![Argument::Lifetime(RegionId(9))], vec![]),
    ] {
        let changed = application(&mut store, item, subject, &trait_args, &family_args)?;
        assert_ne!(store.intern(changed)?, first);
    }
    let Kind::Projection(mut wrong) = kind else {
        return Err("missing projection".into());
    };
    wrong.trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(99),
        arguments: GenericArgs::EMPTY,
    })?;
    assert!(store.intern(Kind::Projection(wrong)).is_err());
    let relaxed = application(&mut store, 13, byte, &[], &[])?;
    let relaxed = store.intern(relaxed)?;
    assert_eq!(store.is_sized(first), Some(true));
    assert_eq!(store.is_sized(relaxed), Some(false));
    assert_eq!(store.contains_parameters(first), Some(false));
    assert_eq!(store.contains_projections(first), Some(true));
    let tuple = store.intern_list(&[first, byte])?;
    let tuple = store.intern(Kind::Tuple(tuple))?;
    assert_eq!(store.contains_projections(tuple), Some(true));
    assert_eq!(store.contains_projections(byte), Some(false));
    assert_eq!(std::mem::size_of::<Application>(), 16);
    assert_eq!(std::mem::size_of::<Kind>(), 24);
    Ok(())
}

#[test]
fn substitution_reaches_self_trait_and_family_arguments_without_binder_capture() -> TestResult {
    let mut store = Store::default();
    schema(&mut store, 11, true)?;
    let t = parameter(&mut store, 20, 0)?;
    let u = parameter(&mut store, 21, 0)?;
    let Kind::Parameter(t_id) = store.kind(t).ok_or("missing T")? else {
        return Err("T kind".into());
    };
    let Kind::Parameter(u_id) = store.kind(u).ok_or("missing U")? else {
        return Err("U kind".into());
    };
    let n_id = store.intern_parameter(Parameter {
        owner: ItemId(20),
        index: 1,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let n = store.intern_const(ConstKind::Parameter(n_id))?;
    let local = borrow(&mut store, RegionId::bound(0, 0)?, u)?;
    let captured = borrow(&mut store, RegionId::bound(1, 1)?, t)?;
    let function = signature(&mut store, &[local], captured, 1)?;
    let kind = application(
        &mut store,
        11,
        t,
        &[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Type(u),
            Argument::Const(n),
        ],
        &[
            Argument::Type(function),
            Argument::Lifetime(RegionId::bound(0, 1)?),
        ],
    )?;
    let original = store.intern(kind)?;
    let mut uses = parameters::uses::Uses::default();
    uses.collect(&store, original)?;
    assert_eq!(uses.parameters, [t_id, u_id].into_iter().collect());
    assert_eq!(uses.constants, [n_id].into_iter().collect());
    let mut regions = bound_uses::Uses::default();
    regions.collect(&store, original)?;
    assert_eq!(regions.indices, [0, 1].into_iter().collect());
    let inserted = borrow(
        &mut store,
        RegionId::bound(0, 3)?,
        TypeId::primitive(Primitive::U16),
    )?;
    let three = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 3,
    })?;
    let boolean = TypeId::primitive(Primitive::Bool);
    let mut fold = Bound::parameters_captured(
        &store,
        &[
            (t_id, Argument::Type(inserted)),
            (u_id, Argument::Type(boolean)),
            (n_id, Argument::Const(three)),
        ],
        vec![RegionId(10), RegionId(11)],
    )?;
    let result = fold.apply(&mut store, original)?;
    let Kind::Projection(value) = store.kind(result).ok_or("missing folded projection")? else {
        return Err("folded kind".into());
    };
    assert_eq!(value.subject, inserted);
    let args = store
        .trait_application(value.trait_application)
        .ok_or("missing folded trait")?
        .arguments;
    assert_eq!(
        store
            .arguments(args)
            .ok_or("missing arguments")?
            .collect::<Vec<_>>(),
        [
            Argument::Lifetime(RegionId(10)),
            Argument::Type(boolean),
            Argument::Const(three)
        ]
    );
    let local = borrow(&mut store, RegionId::bound(0, 0)?, boolean)?;
    let shifted = borrow(
        &mut store,
        RegionId::bound(1, 3)?,
        TypeId::primitive(Primitive::U16),
    )?;
    let result = borrow(&mut store, RegionId(11), shifted)?;
    let expected = signature(&mut store, &[local], result, 1)?;
    assert_eq!(
        store
            .arguments(value.arguments)
            .ok_or("missing family")?
            .collect::<Vec<_>>(),
        [Argument::Type(expected), Argument::Lifetime(RegionId(11))]
    );
    assert_eq!(store.contains_parameters(value.subject), Some(false));
    Ok(())
}

#[test]
fn projections_do_not_invent_concrete_properties_or_reverse_inference() -> TestResult {
    let mut store = Store::default();
    schema(&mut store, 11, true)?;
    let byte = TypeId::primitive(Primitive::U8);
    let first = application(&mut store, 11, byte, &[], &[])?;
    let first = store.intern(first)?;
    let second = application(&mut store, 11, TypeId::UNIT, &[], &[])?;
    let second = store.intern(second)?;
    let definitions = Definitions::default();
    let mut layouts = Layouts::default();
    assert!(layouts.demand(&store, &definitions, first).is_err());
    assert!(layouts.demand(&store, &definitions, byte).is_ok());
    let mut copies = Copies::default();
    assert!(
        copies
            .demand(&store, &definitions, &HashMap::default(), first)
            .is_err()
    );
    assert!(
        copies
            .demand(&store, &definitions, &HashMap::default(), byte)?
            .copy
    );
    let mut equality = Equality::default();
    assert!(
        equality
            .demand(&store, &definitions, &HashMap::default(), first)
            .is_err()
    );
    assert!(
        equality
            .demand(&store, &definitions, &HashMap::default(), byte)?
            .eq
    );
    let mut drop = NeedsDrop::new([]);
    assert!(drop.demand(&store, &definitions, first).is_err());
    assert!(!drop.demand(&store, &definitions, byte)?);
    let mut live = LiveRegions::default();
    assert!(live.get(&store, first).is_err());
    assert!(live.get(&store, byte)?.is_empty());
    let environment = Arc::new(Environment::new(&store, &[])?);
    let mut shapes = Shapes::new(environment);
    assert!(shapes.demand(&store, first).is_err());
    assert!(shapes.demand(&store, byte)?.sized);
    let mut context = Context::default();
    let a = context.known(&store, first)?;
    let b = context.known(&store, second)?;
    context.equate(&store, a, a)?;
    assert_eq!(
        context.incompatible(&store, a, b),
        Err(Error::Normalization)
    );
    Ok(())
}

#[test]
fn projected_shapes_require_exact_trait_inputs_and_grant_only_declared_sizedness() -> TestResult {
    use parameters::{
        environment::Abilities,
        predicates::{Bound as Requirement, Predicate},
    };
    let mut store = Store::default();
    schema(&mut store, 11, true)?;
    schema(&mut store, 12, false)?;
    let subject = parameter(&mut store, 20, 0)?;
    let Some(Kind::Parameter(id)) = store.kind(subject) else {
        return Err("missing shape parameter".into());
    };
    let kind = application(&mut store, 11, subject, &[], &[])?;
    let Kind::Projection(input) = kind else {
        return Err("missing shape projection".into());
    };
    let projected = store.intern(kind)?;
    let relaxed = application(&mut store, 12, subject, &[], &[])?;
    let relaxed = store.intern(relaxed)?;
    let array = store.intern(Kind::Array {
        element: relaxed,
        length: 1,
    })?;
    let environment = Environment::new(&store, &[(id, Abilities::default())])?.with_predicates(
        &store,
        &[Predicate {
            ty: subject,
            bound: Requirement::Trait(input.trait_application),
        }],
    )?;
    let environment = Arc::new(environment);
    let mut shapes = Shapes::new(Arc::clone(&environment));
    let result = shapes.demand(&store, projected)?;
    assert!(result.sized && result.well_formed);
    let result = shapes.demand(&store, relaxed)?;
    assert!(!result.sized && result.well_formed);
    assert!(!shapes.demand(&store, array)?.well_formed);
    let wrong = application(&mut store, 11, TypeId::UNIT, &[], &[])?;
    let wrong = store.intern(wrong)?;
    assert!(shapes.demand(&store, wrong).is_err());
    let wrong = application(
        &mut store,
        11,
        subject,
        &[Argument::Type(TypeId::UNIT)],
        &[],
    )?;
    let wrong = store.intern(wrong)?;
    assert!(shapes.demand(&store, wrong).is_err());
    let definitions = Definitions::default();
    let mut copies = Copies::in_environment(Arc::clone(&environment));
    assert_eq!(
        copies.demand(&store, &definitions, &HashMap::default(), projected)?,
        rustgm::types::copy::Properties::default()
    );
    let mut drops = NeedsDrop::in_environment([], Arc::clone(&environment));
    assert!(drops.demand(&store, &definitions, projected)?);
    let mut regions =
        rustgm::types::destruction::regions::Regions::new([])?.scoped(Arc::clone(&environment));
    let parts = regions.symbolic(&store, &definitions, projected)?;
    assert_eq!(parts.parameters, &[id]);
    assert!(parts.regions.is_empty());
    assert!(
        Layouts::default()
            .demand(&store, &definitions, projected)
            .is_err()
    );
    let environment = environment.as_ref().clone().with_predicates(
        &store,
        &[Predicate {
            ty: relaxed,
            bound: Requirement::Sized,
        }],
    )?;
    assert!(
        Shapes::new(Arc::new(environment))
            .demand(&store, array)?
            .well_formed
    );
    Ok(())
}

#[test]
fn projection_formation_keeps_input_requirements_separate_from_output_components() -> TestResult {
    let mut store = Store::default();
    schema(&mut store, 11, true)?;
    let parameter = parameter(&mut store, 20, 0)?;
    let Some(Kind::Parameter(id)) = store.kind(parameter) else {
        return Err("missing input parameter".into());
    };
    let subject = borrow(&mut store, RegionId(9), parameter)?;
    let kind = application(&mut store, 11, subject, &[], &[])?;
    let projected = store.intern(kind)?;
    let outer = borrow(&mut store, RegionId(10), projected)?;
    let bounds = rustgm::types::outlives::Bounds::default();
    let callables = rustgm::types::callable::Templates::default();
    let mut queries = rustgm::types::outlives::Queries::new(&store, &bounds, &callables);
    let requirements = queries.symbolic_requirements(projected)?;
    assert!(requirements.scalars.regions.is_empty());
    assert_eq!(requirements.scalars.parameters, &[(id, RegionId(9))]);
    // The output can erase, rearrange or hide its input components. Its relation
    // to the outer borrow still requires an opaque-result or normalization proof.
    let requirements = queries.symbolic_requirements(outer)?;
    assert!(requirements.scalars.regions.is_empty());
    assert_eq!(requirements.scalars.parameters, &[(id, RegionId(9))]);
    assert_eq!(
        requirements.projections,
        &[(
            rustgm::types::outlives::Projection {
                ty: projected,
                depth: 0
            },
            RegionId(10)
        )]
    );
    assert!(LiveRegions::default().get(&store, projected).is_err());
    Ok(())
}

#[test]
fn one_projection_under_different_binders_has_separate_substitution_results() -> TestResult {
    let mut store = Store::default();
    schema(&mut store, 11, true)?;
    let byte = TypeId::primitive(Primitive::U8);
    let kind = application(
        &mut store,
        11,
        byte,
        &[Argument::Lifetime(RegionId::bound(1, 0)?)],
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let original = store.intern(kind)?;
    let local = borrow(&mut store, RegionId::bound(0, 0)?, byte)?;
    let parameters = store.intern_list(&[local])?;
    let function = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: original,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let mut fold = Bound::new(vec![RegionId(9)]);
    let at_root = fold.apply(&mut store, original)?;
    let in_function = fold.apply(&mut store, function)?;
    let expected_root = application(
        &mut store,
        11,
        byte,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
        &[Argument::Lifetime(RegionId(9))],
    )?;
    assert_eq!(store.kind(at_root), Some(expected_root));
    let expected_nested = application(
        &mut store,
        11,
        byte,
        &[Argument::Lifetime(RegionId(9))],
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let result = store.intern(expected_nested)?;
    assert_eq!(
        store.kind(in_function),
        Some(Kind::Function {
            bound_regions: 1,
            parameters,
            result,
            unsafe_: false,
            convention: rustgm::types::callable::Convention::RUST,
        })
    );
    assert_ne!(at_root, result);
    assert_eq!(store.escaping_binders(in_function), Some(0));
    Ok(())
}

#[test]
fn shared_projection_inputs_are_folded_once_with_bounded_iterative_work() -> TestResult {
    let mut store = Store::default();
    schema(&mut store, 11, true)?;
    let parameter = parameter(&mut store, 20, 0)?;
    let Kind::Parameter(id) = store.kind(parameter).ok_or("parameter")? else {
        return Err("kind".into());
    };
    let mut root = parameter;
    for _ in 0..10_000 {
        let kind = application(
            &mut store,
            11,
            root,
            &[],
            &[Argument::Type(root), Argument::Type(root)],
        )?;
        root = store.intern(kind)?;
    }
    let mut fold = Bound::parameters(
        &store,
        &[(id, Argument::Type(TypeId::primitive(Primitive::U8)))],
    )?;
    let result = fold.apply(&mut store, root)?;
    assert_eq!(store.contains_parameters(result), Some(false));
    assert_eq!(store.contains_projections(result), Some(true));
    assert!(fold.steps < 100_000, "{}", fold.steps);
    let steps = fold.steps;
    for _ in 0..100 {
        assert_eq!(fold.apply(&mut store, root)?, result);
    }
    assert_eq!(fold.steps, steps);
    Ok(())
}

#[test]
fn projection_inputs_are_invariant_even_in_covariant_field_positions() -> TestResult {
    let mut store = Store::default();
    schema(&mut store, 11, true)?;
    let t = parameter(&mut store, 20, 1)?;
    let u = parameter(&mut store, 20, 2)?;
    let v = parameter(&mut store, 20, 3)?;
    let region = RegionId::bound(0, 0)?;
    let kind = application(
        &mut store,
        11,
        t,
        &[Argument::Lifetime(region), Argument::Type(u)],
        &[Argument::Type(v)],
    )?;
    let projection = store.intern(kind)?;
    let fields = store.intern_list(&[projection])?;
    let mut definitions = Definitions::default();
    definitions.insert(
        ItemId(20),
        aggregate::Kind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let arguments = store.intern_arguments(&[
        Argument::Lifetime(region),
        Argument::Type(t),
        Argument::Type(u),
        Argument::Type(v),
    ])?;
    let variance = Variances::infer(&store, &definitions, &[(ItemId(20), arguments)])?;
    assert_eq!(
        variance.get(ItemId(20)),
        Some([Variance::Invariant; 4].as_slice())
    );
    Ok(())
}
