use rustgm::{
    abi::layout::Layouts,
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
        aggregate::Definitions,
        arguments::{Argument, ConstKind, Constant},
        callable::Templates,
        copy::Copies,
        destruction::{NeedsDrop, regions::Regions},
        equality::Equality,
        infer::{Constructor, Context, Domain},
        outlives::{Bounds, Queries},
        parameters::{Parameter, ParameterId, ParameterKind},
        substitute::Bound,
    },
};
use std::collections::HashMap;

fn type_parameter(
    store: &mut Store,
    owner: u32,
    index: u32,
    sized: bool,
) -> Result<TypeId, &'static str> {
    let id = store.intern_parameter(Parameter {
        owner: ItemId(owner),
        index,
        kind: ParameterKind::Type { sized },
    })?;
    store.intern(Kind::Parameter(id))
}

#[test]
fn parameters_are_owned_immutable_declarations() -> Result<(), &'static str> {
    let mut store = Store::default();
    let declaration = Parameter {
        owner: ItemId(2),
        index: 1,
        kind: ParameterKind::Type { sized: true },
    };
    let id = store.intern_parameter(declaration)?;
    assert_eq!(store.intern_parameter(declaration)?, id);
    let a = store.intern(Kind::Parameter(id))?;
    let b = type_parameter(&mut store, 3, 1, true)?;
    let c = type_parameter(&mut store, 2, 2, true)?;
    assert_ne!(a, b);
    assert_ne!(a, c);
    for kind in [
        ParameterKind::Type { sized: false },
        ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    ] {
        assert!(
            store
                .intern_parameter(Parameter {
                    kind,
                    ..declaration
                })
                .is_err()
        );
    }
    assert_eq!(store.parameter(id), Some(declaration));
    assert!(
        store
            .intern(Kind::Parameter(ParameterId(u32::MAX)))
            .is_err()
    );
    assert!(store.intern_const(ConstKind::Parameter(id)).is_err());
    assert!(
        store
            .intern_const(ConstKind::Parameter(ParameterId(u32::MAX)))
            .is_err()
    );
    Ok(())
}

#[test]
fn type_parameters_are_rigid_but_can_constrain_inference_variables()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let a = type_parameter(&mut store, 0, 0, true)?;
    let b = type_parameter(&mut store, 0, 1, true)?;
    let mut inference = Context::default();
    let left = inference.known(&store, a)?;
    let right = inference.known(&store, b)?;
    assert!(inference.equate(&store, left, right).is_err());
    let concrete = inference.known(&store, TypeId::primitive(Primitive::U8))?;
    assert!(inference.equate(&store, left, concrete).is_err());
    let variable = inference.variable(Domain::Any)?;
    inference.equate(&store, variable, left)?;
    assert_eq!(inference.resolved(&mut store, variable)?, a);
    let numeric = inference.variable(Domain::Integer)?;
    assert!(inference.equate(&store, numeric, right).is_err());
    let Some(Kind::Parameter(id)) = store.kind(a) else {
        return Err("missing type parameter".into());
    };
    let explicit = inference.construct(Constructor::Parameter(id), &[])?;
    assert_eq!(inference.resolved(&mut store, explicit)?, a);
    Ok(())
}

#[test]
fn const_parameters_never_masquerade_as_evaluated_values() -> Result<(), &'static str> {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let declaration = Parameter {
        owner: ItemId(4),
        index: 2,
        kind: ParameterKind::Const { ty: byte },
    };
    let parameter = store.intern_parameter(declaration)?;
    let term = store.intern_const(ConstKind::Parameter(parameter))?;
    assert_eq!(store.intern_const(ConstKind::Parameter(parameter))?, term);
    assert_eq!(
        store.const_kind(term),
        Some(ConstKind::Parameter(parameter))
    );
    assert_eq!(store.const_type(term), Some(byte));
    assert_eq!(store.constant(term), None);
    let zero = store.intern_constant(Constant { ty: byte, bits: 0 })?;
    assert_ne!(term, zero);
    let other = store.intern_parameter(Parameter {
        owner: ItemId(5),
        ..declaration
    })?;
    assert_ne!(store.intern_const(ConstKind::Parameter(other))?, term);
    assert!(store.intern(Kind::Parameter(parameter)).is_err());
    let arguments = store.intern_arguments(&[Argument::Const(term)])?;
    let nominal = store.intern(Kind::Adt(ItemId(8), arguments))?;
    assert_eq!(store.contains_parameters(nominal), Some(true));
    let concrete = store.intern_arguments(&[Argument::Const(zero)])?;
    let concrete = store.intern(Kind::Adt(ItemId(8), concrete))?;
    assert_eq!(store.contains_parameters(concrete), Some(false));
    for primitive in [Primitive::F32, Primitive::F64, Primitive::Str] {
        assert!(
            store
                .intern_parameter(Parameter {
                    kind: ParameterKind::Const {
                        ty: TypeId::primitive(primitive)
                    },
                    ..declaration
                })
                .is_err()
        );
    }
    Ok(())
}

#[test]
fn generic_presence_survives_every_structural_constructor() -> Result<(), &'static str> {
    let mut store = Store::default();
    let parameter = type_parameter(&mut store, 0, 0, true)?;
    let list = store.intern_list(&[parameter])?;
    let args = store.intern_arguments(&[Argument::Type(parameter)])?;
    let function = store.intern(Kind::Function {
        bound_regions: 0,
        parameters: list,
        result: TypeId::UNIT,
        unsafe_: false,
    })?;
    let empty = store.intern_list(&[])?;
    let output = store.intern(Kind::Function {
        bound_regions: 0,
        parameters: empty,
        result: parameter,
        unsafe_: false,
    })?;
    let closed = store.intern(Kind::Function {
        bound_regions: 0,
        parameters: empty,
        result: TypeId::UNIT,
        unsafe_: false,
    })?;
    for kind in [
        Kind::Tuple(list),
        Kind::Adt(ItemId(3), args),
        Kind::Array {
            element: parameter,
            length: 0,
        },
        Kind::Slice(parameter),
        Kind::Reference {
            mutable: false,
            region: RegionId::STATIC,
            referent: parameter,
        },
        Kind::Pointer {
            mutable: false,
            pointee: parameter,
        },
        Kind::FunctionItem {
            item: ItemId(3),
            signature: function,
            arguments: GenericArgs::EMPTY,
        },
        Kind::FunctionItem {
            item: ItemId(3),
            signature: closed,
            arguments: args,
        },
    ] {
        let ty = store.intern(kind)?;
        assert_eq!(store.contains_parameters(ty), Some(true));
    }
    assert_eq!(store.contains_parameters(function), Some(true));
    assert_eq!(store.contains_parameters(output), Some(true));
    assert_eq!(store.contains_parameters(closed), Some(false));
    assert_eq!(store.contains_parameters(TypeId::UNIT), Some(false));
    assert_eq!(store.contains_parameters(TypeId(u32::MAX)), None);
    Ok(())
}

#[test]
fn lifetime_substitution_preserves_rigid_parameters_and_their_requirements()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let parameter = type_parameter(&mut store, 0, 0, false)?;
    assert_eq!(store.is_sized(parameter), Some(false));
    let borrowed = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: parameter,
    })?;
    let expected = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId(10),
        referent: parameter,
    })?;
    let mut substitution = Bound::new(vec![RegionId(10)]);
    assert_eq!(substitution.apply(&mut store, borrowed)?, expected);
    assert_eq!(substitution.apply(&mut store, parameter)?, parameter);
    assert_eq!(store.contains_parameters(expected), Some(true));
    assert_eq!(store.is_sized(expected), Some(true));
    Ok(())
}

#[test]
fn closed_proofs_require_generic_environments_instead_of_fabricated_facts()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let parameter = type_parameter(&mut store, 0, 0, true)?;
    let definitions = Definitions::default();
    assert!(
        Layouts::default()
            .demand(&store, &definitions, parameter)
            .is_err()
    );
    assert!(
        Copies::default()
            .demand(&store, &definitions, &HashMap::new(), parameter)
            .is_err()
    );
    assert!(
        Equality::default()
            .demand(&store, &definitions, &HashMap::new(), parameter)
            .is_err()
    );
    assert!(
        NeedsDrop::new([])
            .demand(&store, &definitions, parameter)
            .is_err()
    );
    assert!(
        Regions::new([])?
            .get(&store, &definitions, parameter)
            .is_err()
    );
    let bounds = Bounds::infer(&store, &definitions, &[])?;
    let callables = Templates::default();
    assert!(
        Queries::new(&store, &bounds, &callables)
            .requirements(parameter)
            .is_err()
    );
    let shared = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::STATIC,
        referent: parameter,
    })?;
    assert!(
        Copies::default()
            .demand(&store, &definitions, &HashMap::new(), shared)?
            .copy
    );
    assert!(!NeedsDrop::new([]).demand(&store, &definitions, shared)?);
    Ok(())
}
