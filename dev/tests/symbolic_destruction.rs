use crate::qualify as rustc;
use crate::semantic;
use rustgm::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    aggregate::Definitions,
    arguments::{Argument, ConstKind},
    destruction::NeedsDrop,
    parameters::{
        Parameter, ParameterId, ParameterKind,
        environment::{Abilities, Environment},
        predicates::{Bound, Predicate},
    },
};
use std::sync::Arc;

fn parameter(types: &mut Store, owner: u32) -> Result<(ParameterId, TypeId), &'static str> {
    let id = types.intern_parameter(Parameter {
        owner: ItemId(owner),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    Ok((id, types.intern(Kind::Parameter(id))?))
}

#[test]
fn symbolic_cleanup_depends_on_fixed_copy_assumptions() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let array = types.intern(Kind::Array {
        element: ty,
        length: 1_000_000_000,
    })?;
    let empty = types.intern(Kind::Array {
        element: ty,
        length: 0,
    })?;
    let pointer = types.intern(Kind::Pointer {
        mutable: true,
        pointee: ty,
    })?;
    let reference = types.intern(Kind::Reference {
        mutable: true,
        region: RegionId::STATIC,
        referent: ty,
    })?;
    let inputs = types.intern_list(&[ty])?;
    let function = types.intern(Kind::Function {
        bound_regions: 0,
        parameters: inputs,
        result: ty,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let definitions = Definitions::default();
    for abilities in [
        Abilities::default(),
        Abilities {
            copy: false,
            clone_: true,
        },
        Abilities {
            copy: true,
            clone_: true,
        },
    ] {
        let environment = Arc::new(Environment::new(&types, &[(id, abilities)])?);
        let mut query = NeedsDrop::in_environment([], environment);
        for root in [ty, array] {
            assert_eq!(query.demand(&types, &definitions, root)?, !abilities.copy);
        }
        for root in [empty, pointer, reference, function] {
            assert!(!query.demand(&types, &definitions, root)?);
        }
        assert!(query.steps < 40, "array length expanded: {}", query.steps);
    }
    assert!(NeedsDrop::new([]).demand(&types, &definitions, ty).is_err());
    Ok(())
}

#[test]
fn composite_copy_does_not_prove_element_cleanup_absent() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (a, first) = parameter(&mut types, 0)?;
    let (b, second) = parameter(&mut types, 1)?;
    let fields = types.intern_list(&[first, second])?;
    let tuple = types.intern(Kind::Tuple(fields))?;
    let parameters = [(a, Abilities::default()), (b, Abilities::default())];
    let environment = Environment::new(&types, &parameters)?.with_predicates(
        &types,
        &[Predicate {
            ty: tuple,
            bound: Bound::Copy,
        }],
    )?;
    let definitions = Definitions::default();
    let mut query = NeedsDrop::in_environment([], Arc::new(environment));
    assert!(!query.demand(&types, &definitions, tuple)?);
    assert!(query.demand(&types, &definitions, first)?);
    assert!(query.demand(&types, &definitions, second)?);
    let mut independent =
        NeedsDrop::in_environment([], Arc::new(Environment::new(&types, &parameters)?));
    assert!(independent.demand(&types, &definitions, tuple)?);
    Ok(())
}

#[test]
fn symbolic_array_lengths_keep_the_element_requirement() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let parameter = types.intern_parameter(Parameter {
        owner: ItemId(0),
        index: 1,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let length = types.intern_const(ConstKind::Parameter(parameter))?;
    let array = types.intern(Kind::ArrayTerm {
        element: ty,
        length,
    })?;
    let plain = types.intern(Kind::ArrayTerm {
        element: TypeId::UNIT,
        length,
    })?;
    let definitions = Definitions::default();
    let mut query = NeedsDrop::in_environment(
        [],
        Arc::new(Environment::new(&types, &[(id, Abilities::default())])?),
    );
    assert!(query.demand(&types, &definitions, array)?);
    assert!(!query.demand(&types, &definitions, plain)?);
    assert!(
        NeedsDrop::new([])
            .demand(&types, &definitions, plain)
            .is_err()
    );
    let arguments = types.intern_arguments(&[Argument::Type(ty)])?;
    let nominal = types.intern(Kind::Adt(ItemId(9), arguments))?;
    assert!(query.demand(&types, &definitions, nominal).is_err());
    // A closed nominal remains a separate completed-definition obligation too.
    let closed = types.intern(Kind::Adt(ItemId(9), GenericArgs::EMPTY))?;
    assert!(query.demand(&types, &definitions, closed).is_err());
    Ok(())
}

#[test]
fn scoped_queries_share_deep_graphs_and_recover_from_foreign_parameters() -> Result<(), &'static str>
{
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let (_, foreign) = parameter(&mut types, 1)?;
    let environment = Arc::new(Environment::new(&types, &[(id, Abilities::default())])?);
    let definitions = Definitions::default();
    let mut root = ty;
    for _ in 0..20_000 {
        let fields = types.intern_list(&[root, root])?;
        root = types.intern(Kind::Tuple(fields))?;
    }
    let mut query = NeedsDrop::in_environment([], environment);
    for _ in 0..2 {
        assert_eq!(
            query.demand(&types, &definitions, foreign),
            Err("destruction parameter is outside its environment")
        );
    }
    assert!(query.demand(&types, &definitions, root)?);
    let steps = query.steps;
    assert!(steps < types.len() * 10);
    for _ in 0..100 {
        assert!(query.demand(&types, &definitions, root)?);
    }
    assert_eq!(query.steps, steps);
    assert!(
        query
            .demand(&types, &definitions, TypeId(u32::MAX))
            .is_err()
    );
    assert!(!query.demand(&types, &definitions, TypeId::UNIT)?);
    Ok(())
}

#[test]
fn source_environments_preserve_possible_cleanup_and_match_concrete_rustc_controls()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("plain", "T", "T", "D", true),
        ("copy", "T:Copy", "T", "u8", false),
        ("clone", "T:Clone", "T", "D", true),
        ("static", "T:'static", "T", "D", true),
        ("empty", "T", "[T;0]", "[D;0]", false),
        ("array", "T", "[T;2]", "[D;2]", true),
        ("const-array", "T,const N:usize", "[T;N]", "[D;2]", true),
        (
            "copy-array",
            "T:Copy,const N:usize",
            "[T;N]",
            "[u8;2]",
            false,
        ),
        ("plain-array", "const N:usize", "[u8;N]", "[u8;2]", false),
        ("tuple", "T", "(u8,T)", "(u8,D)", true),
        ("shared", "T", "&T", "&'static D", false),
        ("mutable", "T", "&mut T", "&'static mut D", false),
        ("raw", "T", "*const T", "*const D", false),
        ("callable", "T", "fn(T)->T", "fn(D)->D", false),
    ];
    for (name, parameters, written, concrete, expected) in cases {
        let source = format!("fn probe<{parameters}>(value:{written}){{}}");
        rustc::qualify(&format!("symbolic-drop-{name}"), &source, true)?;
        let independent = format!(
            "#[derive(Clone)]struct D;impl Drop for D{{fn drop(&mut self){{}}}}const _: [();{}]=[();std::mem::needs_drop::<{concrete}>() as usize];",
            usize::from(expected)
        );
        rustc::qualify(&format!("concrete-drop-{name}"), &independent, true)?;
        let (_, facts) = semantic::analyze(&source)?;
        let function = facts.functions.first().ok_or("missing probe")?;
        let environment = facts
            .generics
            .environments
            .get(&function.definition)
            .ok_or("missing environment")?;
        let Some(Kind::Function { parameters, .. }) = facts.types.kind(function.signature) else {
            return Err("missing signature".into());
        };
        let ty = *facts
            .types
            .list(parameters)
            .and_then(|p| p.first())
            .ok_or("missing input")?;
        let mut query = NeedsDrop::in_environment([], Arc::clone(environment));
        assert_eq!(
            query.demand(&facts.types, &facts.aggregates, ty)?,
            expected,
            "{name}"
        );
    }
    Ok(())
}
