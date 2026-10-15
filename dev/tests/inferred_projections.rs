//! Projection inputs survive inference transformations without becoming result equations.
use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument as Canonical, ConstKind, Constant},
    callable::Templates,
    infer::{Constructor, Context, Domain, Error, arguments::Argument},
    parameters::{Parameter, ParameterKind},
    projection::{Application, Schema},
    substitute::Bound,
    traits,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const ITEM: ItemId = ItemId(11);
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
const WORD: TypeId = TypeId::primitive(Primitive::U16);
const SIZE: TypeId = TypeId::primitive(Primitive::Usize);

fn store() -> Result<Store, &'static str> {
    let mut store = Store::default();
    store.declare_projection(
        ITEM,
        Schema {
            trait_item: ItemId(10),
            sized: true,
        },
    )?;
    Ok(store)
}
fn canonical(
    store: &mut Store,
    subject: TypeId,
    trait_arguments: &[Canonical],
    family: &[Canonical],
) -> Result<TypeId, &'static str> {
    let arguments = store.intern_arguments(trait_arguments)?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(10),
        arguments,
    })?;
    let arguments = store.intern_arguments(family)?;
    store.intern(Kind::Projection(Application {
        item: ITEM,
        subject,
        trait_application,
        arguments,
    }))
}

#[test]
fn self_trait_and_family_arguments_share_type_and_const_variables() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let t = context.variable(Domain::Any)?;
    let n = context.const_variable(&store, SIZE)?;
    let projection = context.projected(
        &store,
        ITEM,
        t,
        &[
            Argument::Lifetime(RegionId(7)),
            Argument::Type(t),
            Argument::Const(n),
        ],
        &[
            Argument::Const(n),
            Argument::Type(t),
            Argument::Lifetime(RegionId(8)),
        ],
    )?;
    assert_eq!(
        context.resolved(&mut store, projection),
        Err(Error::Unresolved)
    );
    let word = context.known(&store, WORD)?;
    context.equate(&store, t, word)?;
    assert_eq!(
        context.resolved(&mut store, projection),
        Err(Error::Unresolved)
    );
    let three = store.intern_constant(Constant { ty: SIZE, bits: 3 })?;
    let constant = context.known_const(&store, three)?;
    context.equate_consts(n, constant)?;
    let expected = canonical(
        &mut store,
        WORD,
        &[
            Canonical::Lifetime(RegionId(7)),
            Canonical::Type(WORD),
            Canonical::Const(three),
        ],
        &[
            Canonical::Const(three),
            Canonical::Type(WORD),
            Canonical::Lifetime(RegionId(8)),
        ],
    )?;
    assert_eq!(context.resolved(&mut store, projection)?, expected);
    assert_eq!(context.head(&store, projection), Err(Error::Normalization));
    Ok(())
}

#[test]
fn output_equations_cannot_infer_projection_inputs() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let t = context.variable(Domain::Any)?;
    let u = context.variable(Domain::Any)?;
    let first = context.projected(&store, ITEM, t, &[], &[])?;
    let second = context.projected(&store, ITEM, u, &[], &[])?;
    assert_eq!(
        context.equate(&store, first, second),
        Err(Error::Normalization)
    );
    let byte = context.known(&store, BYTE)?;
    assert_eq!(
        context.equate(&store, first, byte),
        Err(Error::Normalization)
    );
    assert_eq!(context.unresolved_domain(t)?, Some(Domain::Any));
    assert_eq!(context.unresolved_domain(u)?, Some(Domain::Any));
    assert_eq!(
        context.incompatible(&store, first, second),
        Err(Error::Normalization)
    );
    assert_eq!(context.resolved(&mut store, first), Err(Error::Unresolved));
    Ok(())
}

#[test]
fn parameter_instantiation_preserves_projection_argument_kinds_and_binders() -> TestResult {
    let mut store = store()?;
    let t = store.intern_parameter(Parameter {
        owner: ItemId(20),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let n = store.intern_parameter(Parameter {
        owner: ItemId(20),
        index: 1,
        kind: ParameterKind::Const { ty: SIZE },
    })?;
    let ty = store.intern(Kind::Parameter(t))?;
    let constant = store.intern_const(ConstKind::Parameter(n))?;
    let projected = canonical(
        &mut store,
        ty,
        &[
            Canonical::Lifetime(RegionId::bound(0, 0)?),
            Canonical::Const(constant),
        ],
        &[
            Canonical::Type(ty),
            Canonical::Lifetime(RegionId::bound(1, 0)?),
        ],
    )?;
    let parameters = store.intern_list(&[projected])?;
    let template = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
    })?;
    let mut context = Context::default();
    let value = context.variable(Domain::Any)?;
    let size = context.const_variable(&store, SIZE)?;
    let inferred = context.instantiate(
        &mut store,
        template,
        &[(t, Argument::Type(value)), (n, Argument::Const(size))],
    )?;
    let reference = store.intern(Kind::Reference {
        region: RegionId(7),
        mutable: false,
        referent: WORD,
    })?;
    let known = context.known(&store, reference)?;
    context.equate(&store, value, known)?;
    let five = store.intern_constant(Constant { ty: SIZE, bits: 5 })?;
    let known = context.known_const(&store, five)?;
    context.equate_consts(size, known)?;
    let expected = Bound::parameters(
        &store,
        &[(t, Canonical::Type(reference)), (n, Canonical::Const(five))],
    )?
    .apply(&mut store, template)?;
    assert_eq!(context.resolved(&mut store, inferred)?, expected);
    let replaced = context.substitute_regions(&mut store, inferred, &[RegionId(8)])?;
    let expected = Bound::new(vec![RegionId(8)]).apply(&mut store, expected)?;
    assert_eq!(context.resolved(&mut store, replaced)?, expected);
    Ok(())
}

#[test]
fn projection_inputs_survive_import_without_sharing_probe_variables() -> TestResult {
    let mut store = store()?;
    let mut source = Context::default();
    let t = source.variable(Domain::Any)?;
    let n = source.const_variable(&store, SIZE)?;
    let projected = source.projected(
        &store,
        ITEM,
        t,
        &[Argument::Type(t), Argument::Const(n)],
        &[Argument::Type(t), Argument::Const(n)],
    )?;
    let array = source.construct(Constructor::ArrayInfer(n), &[t])?;
    let tuple = source.construct(Constructor::Tuple, &[projected, t, array])?;
    let mut probe = Context::default();
    let copied = probe.import(&store, &mut source, tuple)?;
    let copied_t = probe.child(&store, copied, 1)?;
    let copied_array = probe.child(&store, copied, 2)?;
    let word = probe.known(&store, WORD)?;
    probe.equate(&store, copied_t, word)?;
    let three_array = probe.construct(Constructor::Array(3), &[word])?;
    probe.equate(&store, copied_array, three_array)?;
    let copied_projection = probe.child(&store, copied, 0)?;
    let three = store.intern_constant(Constant { ty: SIZE, bits: 3 })?;
    let expected = canonical(
        &mut store,
        WORD,
        &[Canonical::Type(WORD), Canonical::Const(three)],
        &[Canonical::Type(WORD), Canonical::Const(three)],
    )?;
    assert_eq!(probe.resolved(&mut store, copied_projection)?, expected);
    assert_eq!(
        source.resolved(&mut store, projected),
        Err(Error::Unresolved)
    );
    Ok(())
}

#[test]
fn region_substitution_changes_inputs_without_freshening_an_unknown_result() -> TestResult {
    let mut store = store()?;
    let mut context = Context::default();
    let t = context.variable(Domain::Any)?;
    let projected = context.projected(
        &store,
        ITEM,
        t,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
        &[Argument::Lifetime(RegionId::bound(0, 1)?)],
    )?;
    let mapped = context.substitute_regions(&mut store, projected, &[RegionId(7), RegionId(8)])?;
    let byte = context.known(&store, BYTE)?;
    context.equate(&store, t, byte)?;
    let expected = canonical(
        &mut store,
        BYTE,
        &[Canonical::Lifetime(RegionId(7))],
        &[Canonical::Lifetime(RegionId(8))],
    )?;
    assert_eq!(context.resolved(&mut store, mapped)?, expected);
    assert!(matches!(
        context.fresh_regions(&mut store, &Templates::default(), mapped, 20),
        Err(Error::Normalization)
    ));
    assert!(
        context
            .projected(&store, ItemId(99), byte, &[], &[])
            .is_err()
    );
    Ok(())
}
