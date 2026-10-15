//! Isolated trait equations preserve shared Self/type/const variables and region obligations.
use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, ConstKind, Constant},
    infer::{self, Context, Domain, Instantiation},
    parameters::{
        Parameter, ParameterKind,
        predicates::{Bound, Predicate},
    },
    traits::Application,
};

fn goal(
    store: &mut Store,
    subject: TypeId,
    item: u32,
    arguments: &[Argument],
) -> Result<Predicate, &'static str> {
    let arguments = store.intern_arguments(arguments)?;
    let application = store.intern_trait_application(Application {
        item: ItemId(item),
        arguments,
    })?;
    Ok(Predicate {
        ty: subject,
        bound: Bound::Trait(application),
    })
}

#[test]
fn probes_preserve_aliases_across_self_type_and_const_roots()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let word = TypeId::primitive(Primitive::U16);
    let usize_ = TypeId::primitive(Primitive::Usize);
    let t = store.intern_parameter(Parameter {
        owner: ItemId(10),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let n = store.intern_parameter(Parameter {
        owner: ItemId(10),
        index: 1,
        kind: ParameterKind::Const { ty: usize_ },
    })?;
    let ty = store.intern(Kind::Parameter(t))?;
    let length = store.intern_const(ConstKind::Parameter(n))?;
    let array = store.intern(Kind::ArrayTerm {
        element: ty,
        length,
    })?;
    let template = goal(
        &mut store,
        ty,
        200,
        &[
            Argument::Type(ty),
            Argument::Type(array),
            Argument::Const(length),
        ],
    )?;
    let three = store.intern_constant(Constant {
        ty: usize_,
        bits: 3,
    })?;
    let four = store.intern_constant(Constant {
        ty: usize_,
        bits: 4,
    })?;
    let bytes = store.intern(Kind::Array {
        element: byte,
        length: 3,
    })?;
    let mut source = Context::default();
    let unknown = source.variable(Domain::Any)?;
    let count = source.const_variable(&store, usize_)?;
    let mut substitution = Instantiation::new(
        &mut source,
        &store,
        &[
            (t, infer::arguments::Argument::Type(unknown)),
            (n, infer::arguments::Argument::Const(count)),
        ],
    )?;
    let predicate = source.instantiate_predicate(&mut store, template, &mut substitution)?;
    for (item, argument, constant) in [(200, word, three), (200, byte, four), (201, byte, three)] {
        let candidate = goal(
            &mut store,
            byte,
            item,
            &[
                Argument::Type(argument),
                Argument::Type(bytes),
                Argument::Const(constant),
            ],
        )?;
        let mut probe = Context::default();
        let copied = probe.import_predicate(&store, &mut source, &predicate)?;
        assert_eq!(
            probe.equate_trait_predicate(&store, &copied, candidate),
            Err(infer::Error::Mismatch)
        );
        assert!(matches!(
            source.resolved_predicate(&mut store, &predicate),
            Err(infer::Error::Unresolved)
        ));
        assert!(source.region_equalities.is_empty());
    }
    let selected = goal(
        &mut store,
        byte,
        200,
        &[
            Argument::Type(byte),
            Argument::Type(bytes),
            Argument::Const(three),
        ],
    )?;
    let mut probe = Context::default();
    let copied = probe.import_predicate(&store, &mut source, &predicate)?;
    probe.equate_trait_predicate(&store, &copied, selected)?;
    assert_eq!(probe.resolved_predicate(&mut store, &copied)?, selected);
    assert!(matches!(
        source.resolved_predicate(&mut store, &predicate),
        Err(infer::Error::Unresolved)
    ));
    source.equate_trait_predicate(&store, &predicate, selected)?;
    let (_, solved) = source.finish_with_predicates(&mut store, std::iter::once(&predicate))?;
    assert_eq!(solved, [selected]);
    Ok(())
}

#[test]
fn lifetime_equations_remain_requirements_until_the_shared_region_proof()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let reference = store.intern(Kind::Reference {
        referent: byte,
        mutable: false,
        region: a,
    })?;
    let other = store.intern(Kind::Reference {
        referent: byte,
        mutable: false,
        region: b,
    })?;
    let template = goal(
        &mut store,
        byte,
        200,
        &[Argument::Lifetime(a), Argument::Type(reference)],
    )?;
    let candidate = goal(
        &mut store,
        byte,
        200,
        &[Argument::Lifetime(b), Argument::Type(other)],
    )?;
    let mut source = Context::default();
    let mut substitution = Instantiation::new(&mut source, &store, &[])?;
    let predicate = source.instantiate_predicate(&mut store, template, &mut substitution)?;
    let mut probe = Context::default();
    let copied = probe.import_predicate(&store, &mut source, &predicate)?;
    probe.equate_trait_predicate(&store, &copied, candidate)?;
    assert!(source.region_equalities.is_empty());
    assert!(!probe.region_equalities.is_empty());
    assert!(
        probe
            .region_equalities
            .iter()
            .all(|pair| *pair == (a, b) || *pair == (b, a))
    );
    let (solved, _) = probe.finish_with_predicates(&mut store, std::iter::once(&copied))?;
    assert!(!solved.region_equalities.is_empty());
    Ok(())
}

#[test]
fn repeated_argument_roots_share_the_import_mapping() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let t = store.intern_parameter(Parameter {
        owner: ItemId(10),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = store.intern(Kind::Parameter(t))?;
    let mut source = Context::default();
    let term = source.variable(Domain::Any)?;
    let mut substitution = Instantiation::new(
        &mut source,
        &store,
        &[(t, infer::arguments::Argument::Type(term))],
    )?;
    let template = goal(&mut store, ty, 200, &vec![Argument::Type(ty); 4096])?;
    let predicate = source.instantiate_predicate(&mut store, template, &mut substitution)?;
    let mut probe = Context::default();
    let copied = probe.import_predicate(&store, &mut source, &predicate)?;
    let (_, arguments) = copied
        .trait_arguments()
        .ok_or("missing imported trait arguments")?;
    assert_eq!(arguments.len(), 4096);
    assert!(
        arguments
            .iter()
            .all(|a| *a == infer::arguments::Argument::Type(copied.subject()))
    );
    assert!(probe.steps < 10_000, "{}", probe.steps);
    Ok(())
}
