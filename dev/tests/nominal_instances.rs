use crate::qualify as rustc;
use rustgm::{
    abi::layout::Layouts,
    types::{
        ItemId, Kind, Primitive, RegionId, Store, TypeId,
        aggregate::{self, Definitions, Repr, Representation, Variant},
        arguments::{Argument, ConstKind, Constant},
        copy::{Copies, Properties},
        destruction::{NeedsDrop, regions::Regions},
        parameters::{Parameter, ParameterKind},
    },
};

fn parameter(types: &mut Store, owner: ItemId, index: u32) -> Result<TypeId, &'static str> {
    let id = types.intern_parameter(Parameter {
        owner,
        index,
        kind: ParameterKind::Type { sized: true },
    })?;
    types.intern(Kind::Parameter(id))
}

#[test]
fn conditional_derives_preserve_stronger_copy_bounds_for_packed_clone() -> Result<(), &'static str>
{
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let wrapper = ItemId(0);
    let inner = ItemId(1);
    let parameter = parameter(&mut types, wrapper, 0)?;
    let pointer = types.intern(Kind::Pointer {
        mutable: false,
        pointee: parameter,
    })?;
    let fields = types.intern_list(&[pointer])?;
    definitions.insert(
        wrapper,
        aggregate::Kind::Struct,
        Repr {
            packed: Some(1),
            ..Repr::default()
        },
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    define(&mut types, &mut definitions, inner, &[])?;
    let inner = nominal(&mut types, inner, &[])?;
    let byte = TypeId::primitive(Primitive::U8);
    let small = nominal(&mut types, wrapper, &[Argument::Type(byte)])?;
    let clone_only = nominal(&mut types, wrapper, &[Argument::Type(inner)])?;
    for ty in [small, clone_only] {
        definitions.prepare(&mut types, ty)?;
    }
    let selected = std::collections::HashMap::from([
        (
            wrapper,
            Properties {
                copy: true,
                clone_: true,
            },
        ),
        (
            ItemId(1),
            Properties {
                copy: false,
                clone_: true,
            },
        ),
    ]);
    let mut copies = Copies::default();
    assert_eq!(
        copies.demand(&types, &definitions, &selected, small)?,
        Properties {
            copy: true,
            clone_: true
        }
    );
    assert_eq!(
        copies.demand(&types, &definitions, &selected, clone_only)?,
        Properties {
            copy: false,
            clone_: false
        }
    );
    let mut drops = NeedsDrop::new([ItemId(1)]);
    assert!(!drops.demand(&types, &definitions, clone_only)?);
    assert!(drops.demand(&types, &definitions, inner)?);
    Ok(())
}

#[test]
fn custom_drop_retains_type_argument_lifetimes_independently_of_copy() -> Result<(), &'static str> {
    use rustgm::types::parameters::{
        environment::{Abilities, Environment},
        predicates::{Bound, Predicate},
    };
    use std::sync::Arc;
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let owner = ItemId(0);
    let field = parameter(&mut types, owner, 0)?;
    define(&mut types, &mut definitions, owner, &[field])?;
    let foreign = parameter(&mut types, ItemId(1), 0)?;
    let Some(Kind::Parameter(id)) = types.kind(foreign) else {
        return Err("parameter");
    };
    let symbolic = nominal(&mut types, owner, &[Argument::Type(foreign)])?;
    let referent = TypeId::primitive(Primitive::U8);
    let reference = types.intern(Kind::Reference {
        region: RegionId(7),
        mutable: false,
        referent,
    })?;
    let concrete = nominal(&mut types, owner, &[Argument::Type(reference)])?;
    let copy = Arc::new(Environment::new(
        &types,
        &[(
            id,
            Abilities {
                copy: true,
                clone_: true,
            },
        )],
    )?);
    for ty in [symbolic, concrete] {
        definitions.prepare(&mut types, ty)?;
    }
    let mut custom = Regions::new([owner])?.scoped(Arc::clone(&copy));
    assert_eq!(
        custom.symbolic(&types, &definitions, symbolic)?.parameters,
        &[id]
    );
    assert_eq!(custom.get(&types, &definitions, concrete)?, &[RegionId(7)]);
    let mut structural = Regions::new([])?.scoped(Arc::clone(&copy));
    assert!(
        structural
            .symbolic(&types, &definitions, symbolic)?
            .is_empty()
    );
    assert!(structural.get(&types, &definitions, concrete)?.is_empty());
    let mut drops = NeedsDrop::in_environment([owner], Arc::clone(&copy));
    assert!(drops.demand(&types, &definitions, symbolic)?);
    assert!(drops.demand(&types, &definitions, concrete)?);
    let selected = std::collections::HashMap::new();
    let environment = Environment::new(&types, &[(id, Abilities::default())])?.with_predicates(
        &types,
        &[Predicate {
            ty: symbolic,
            bound: Bound::Copy,
        }],
    )?;
    let mut copies = Copies::in_environment(Arc::new(environment));
    assert!(
        copies
            .demand(&types, &definitions, &selected, symbolic)?
            .copy
    );
    assert!(
        !copies
            .demand(&types, &definitions, &selected, foreign)?
            .copy
    );
    Ok(())
}

#[test]
fn demanded_fields_stop_expanding_pointer_recursion_and_prepare_nested_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let item = ItemId(0);
    let parameter = parameter(&mut types, item, 0)?;
    let tuple = types.intern_list(&[parameter])?;
    let tuple = types.intern(Kind::Tuple(tuple))?;
    let next = nominal(&mut types, item, &[Argument::Type(tuple)])?;
    let pointer = types.intern(Kind::Pointer {
        mutable: false,
        pointee: next,
    })?;
    define(&mut types, &mut definitions, item, &[parameter, pointer])?;
    let byte = TypeId::primitive(Primitive::U8);
    let small = nominal(&mut types, item, &[Argument::Type(byte)])?;
    let nested = nominal(&mut types, item, &[Argument::Type(small)])?;
    definitions.prepare(&mut types, nested)?;
    assert!(definitions.instantiation_steps() < 100);
    let mut layouts = Layouts::default();
    for (ty, size) in [(small, 16), (nested, 24)] {
        let id = layouts.demand(&types, &definitions, ty)?;
        assert_eq!(layouts.at(id).ok_or("layout")?.size, size);
    }
    let steps = definitions.instantiation_steps();
    definitions.prepare(&mut types, nested)?;
    assert_eq!(definitions.instantiation_steps(), steps);
    Ok(())
}

#[test]
fn incomplete_field_graphs_never_publish_successful_descendants() -> Result<(), &'static str> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let missing = nominal(&mut types, ItemId(99), &[])?;
    let item = ItemId(0);
    let parameter = parameter(&mut types, item, 0)?;
    define(&mut types, &mut definitions, item, &[parameter, missing])?;
    let byte = TypeId::primitive(Primitive::U8);
    let root = nominal(&mut types, item, &[Argument::Type(byte)])?;
    assert!(definitions.prepare(&mut types, root).is_err());
    let steps = definitions.instantiation_steps();
    assert!(definitions.prepare(&mut types, root).is_err());
    assert_eq!(definitions.instantiation_steps(), steps);
    assert!(definitions.prepare(&mut types, missing).is_err());
    definitions.prepare(&mut types, byte)?;
    Ok(())
}

#[test]
fn constant_and_lifetime_arguments_separate_drop_queries_and_storage()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let owner = ItemId(0);
    let destructor = ItemId(1);
    let element = parameter(&mut types, owner, 0)?;
    let count = types.intern_parameter(Parameter {
        owner,
        index: 1,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let length = types.intern_const(ConstKind::Parameter(count))?;
    let array = types.intern(Kind::ArrayTerm { element, length })?;
    define(&mut types, &mut definitions, owner, &[array])?;
    let referent = TypeId::primitive(Primitive::U8);
    let reference = types.intern(Kind::Reference {
        region: RegionId::bound(0, 0)?,
        mutable: false,
        referent,
    })?;
    define(&mut types, &mut definitions, destructor, &[reference])?;
    let mut layouts = Layouts::default();
    let mut drops = NeedsDrop::new([destructor]);
    let mut regions = Regions::new([destructor])?;
    for region in [RegionId(5), RegionId(9)] {
        let element = nominal(&mut types, destructor, &[Argument::Lifetime(region)])?;
        for count in [0, 3] {
            let length = types.intern_const(ConstKind::Value(Constant {
                ty: TypeId::primitive(Primitive::Usize),
                bits: count,
            }))?;
            let root = nominal(
                &mut types,
                owner,
                &[Argument::Type(element), Argument::Const(length)],
            )?;
            definitions.prepare(&mut types, root)?;
            assert_eq!(drops.demand(&types, &definitions, root)?, count != 0);
            assert_eq!(
                regions.get(&types, &definitions, root)?,
                if count == 0 {
                    &[][..]
                } else {
                    std::slice::from_ref(&region)
                }
            );
            let id = layouts.demand(&types, &definitions, root)?;
            assert_eq!(u128::from(layouts.at(id).ok_or("layout")?.size), count * 8);
        }
    }
    let wrong = types.intern_const(ConstKind::Value(Constant {
        ty: TypeId::primitive(Primitive::Bool),
        bits: 1,
    }))?;
    let wrong = nominal(
        &mut types,
        owner,
        &[Argument::Type(referent), Argument::Const(wrong)],
    )?;
    assert!(definitions.instantiate(&mut types, wrong).is_err());
    let steps = definitions.instantiation_steps();
    assert!(definitions.instantiate(&mut types, wrong).is_err());
    assert!(definitions.shape(&types, wrong).is_err());
    assert_eq!(definitions.instantiation_steps(), steps);
    Ok(())
}

#[test]
fn generic_recipes_require_fields_even_when_the_lifetime_only_key_is_cached()
-> Result<(), &'static str> {
    use rustgm::{body::drops::glue::Catalog, sema::members::Members};
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let item = ItemId(0);
    define(&mut types, &mut definitions, item, &[])?;
    let ordinary = nominal(&mut types, item, &[])?;
    let generic = nominal(&mut types, item, &[Argument::Type(TypeId::UNIT)])?;
    let members = Members::default();
    let mut catalog = Catalog::new(&types, &definitions, &members, &[])?;
    assert_eq!(catalog.demand(ordinary)?, None);
    assert_eq!(
        catalog.demand(generic),
        Err("generic aggregate requires instantiated fields")
    );
    assert_eq!(catalog.demand(ordinary)?, None);
    Ok(())
}

#[test]
fn field_availability_does_not_authorize_recursive_by_value_layouts() -> Result<(), &'static str> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let item = ItemId(0);
    let parameter = parameter(&mut types, item, 0)?;
    let recursive = nominal(&mut types, item, &[Argument::Type(parameter)])?;
    define(&mut types, &mut definitions, item, &[parameter, recursive])?;
    let byte = TypeId::primitive(Primitive::U8);
    let root = nominal(&mut types, item, &[Argument::Type(byte)])?;
    definitions.prepare(&mut types, root)?;
    assert!(definitions.instantiation_steps() < 40);
    let mut layouts = Layouts::default();
    assert!(layouts.demand(&types, &definitions, root).is_err());
    let steps = layouts.steps;
    assert!(layouts.demand(&types, &definitions, root).is_err());
    assert_eq!(layouts.steps, steps);
    assert!(layouts.demand(&types, &definitions, byte).is_ok());
    assert_eq!(
        NeedsDrop::new([]).demand(&types, &definitions, root),
        Err("recursive by-value destruction requirement")
    );
    let selected = std::collections::HashMap::from([(
        item,
        Properties {
            copy: true,
            clone_: true,
        },
    )]);
    assert_eq!(
        Copies::default().demand(&types, &definitions, &selected, root),
        Err("recursive by-value Copy obligation")
    );
    Ok(())
}
fn nominal(
    types: &mut Store,
    owner: ItemId,
    arguments: &[Argument],
) -> Result<TypeId, &'static str> {
    let arguments = types.intern_arguments(arguments)?;
    types.intern(Kind::Adt(owner, arguments))
}
fn define(
    types: &mut Store,
    definitions: &mut Definitions,
    item: ItemId,
    fields: &[TypeId],
) -> Result<(), &'static str> {
    let fields = types.intern_list(fields)?;
    definitions.insert(
        item,
        aggregate::Kind::Struct,
        Repr {
            representation: Representation::C,
            ..Repr::default()
        },
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )
}

#[test]
fn concrete_field_instances_change_layout_and_reuse_completed_substitution()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let item = ItemId(0);
    let parameter = parameter(&mut types, item, 0)?;
    let byte = TypeId::primitive(Primitive::U8);
    let word = TypeId::primitive(Primitive::U64);
    define(&mut types, &mut definitions, item, &[byte, parameter])?;
    let small = nominal(&mut types, item, &[Argument::Type(byte)])?;
    let large = nominal(&mut types, item, &[Argument::Type(word)])?;
    assert!(definitions.shape(&types, small).is_err());
    assert!(
        Layouts::default()
            .demand(&types, &definitions, small)
            .is_err()
    );
    let mut layouts = Layouts::default();
    for (ty, size, align) in [(small, 2, 1), (large, 16, 8)] {
        definitions.instantiate(&mut types, ty)?;
        let id = layouts.demand(&types, &definitions, ty)?;
        let layout = layouts.at(id).ok_or("layout")?;
        assert_eq!((layout.size, layout.align), (size, align));
        let fields = definitions
            .shape(&types, ty)?
            .variant(0)
            .ok_or("variant")?
            .fields;
        assert_eq!(
            types.list(fields),
            Some([byte, if ty == small { byte } else { word }].as_slice())
        );
    }
    let steps = definitions.instantiation_steps();
    let count = types.len();
    for _ in 0..100 {
        definitions.instantiate(&mut types, small)?;
        definitions.instantiate(&mut types, large)?;
    }
    assert_eq!(definitions.instantiation_steps(), steps);
    assert_eq!(types.len(), count);
    Ok(())
}

#[test]
fn joint_field_arguments_keep_const_lengths_lifetimes_and_final_discriminants()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let owner = ItemId(0);
    let element = parameter(&mut types, owner, 1)?;
    let count = types.intern_parameter(Parameter {
        owner,
        index: 2,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let count = types.intern_const(ConstKind::Parameter(count))?;
    let array = types.intern(Kind::ArrayTerm {
        element,
        length: count,
    })?;
    let reference = types.intern(Kind::Reference {
        region: RegionId::bound(0, 0)?,
        mutable: false,
        referent: element,
    })?;
    let fields = types.intern_list(&[array, reference])?;
    definitions.insert(
        owner,
        aggregate::Kind::Enum,
        Repr {
            representation: Representation::C,
            tag: Some(Primitive::U8),
            ..Repr::default()
        },
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let byte = TypeId::primitive(Primitive::U8);
    let count = types.intern_const(ConstKind::Value(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 3,
    }))?;
    let ty = nominal(
        &mut types,
        owner,
        &[
            Argument::Lifetime(RegionId(9)),
            Argument::Type(byte),
            Argument::Const(count),
        ],
    )?;
    definitions.instantiate(&mut types, ty)?;
    definitions.set_discriminant(owner, 0, 17)?;
    let shape = definitions.shape(&types, ty)?;
    let variant = shape.variant(0).ok_or("variant")?;
    assert_eq!(variant.discriminant, 17);
    let fields = types.list(variant.fields).ok_or("fields")?;
    assert_eq!(
        types.kind(fields[0]),
        Some(Kind::Array {
            element: byte,
            length: 3
        })
    );
    assert_eq!(
        types.kind(fields[1]),
        Some(Kind::Reference {
            region: RegionId(9),
            mutable: false,
            referent: byte
        })
    );
    let mut layouts = Layouts::default();
    let id = layouts.demand(&types, &definitions, ty)?;
    let layout = layouts.at(id).ok_or("layout")?;
    assert_eq!((layout.size, layout.align), (24, 8));
    assert_eq!(layouts.variants(layout)[0].discriminant, 17);
    Ok(())
}

#[test]
fn reference_equality_preparation_is_distinct_from_owned_layout_preparation()
-> Result<(), &'static str> {
    use rustgm::types::equality::{Equality, Properties as EqualityProperties};
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let owner = ItemId(0);
    let parameter = parameter(&mut types, owner, 0)?;
    let pointer = types.intern(Kind::Pointer {
        mutable: false,
        pointee: parameter,
    })?;
    define(&mut types, &mut definitions, owner, &[pointer])?;
    let float = TypeId::primitive(Primitive::F32);
    let wrapper = nominal(&mut types, owner, &[Argument::Type(float)])?;
    let reference = types.intern(Kind::Reference {
        region: RegionId::STATIC,
        mutable: false,
        referent: wrapper,
    })?;
    definitions.prepare(&mut types, reference)?;
    assert!(definitions.shape(&types, wrapper).is_err());
    definitions.prepare_equality(&mut types, reference)?;
    let selected = std::collections::HashMap::from([(
        owner,
        EqualityProperties {
            partial_eq: true,
            eq: true,
        },
    )]);
    let mut query = Equality::default();
    assert_eq!(
        query.demand(&types, &definitions, &selected, reference)?,
        EqualityProperties {
            partial_eq: true,
            eq: false
        }
    );
    let cost = query.work;
    query.demand(&types, &definitions, &selected, reference)?;
    assert_eq!(query.work, cost);
    Ok(())
}

#[test]
fn shared_generic_field_graphs_are_iterative_and_memoized() -> Result<(), Box<dyn std::error::Error>>
{
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let owner = ItemId(0);
    let parameter = parameter(&mut types, owner, 0)?;
    define(&mut types, &mut definitions, owner, &[parameter, parameter])?;
    let mut root = TypeId::UNIT;
    for _ in 0..10_000 {
        root = nominal(&mut types, owner, &[Argument::Type(root)])?;
    }
    definitions.prepare(&mut types, root)?;
    assert!(definitions.instantiation_steps() < 250_000);
    let count = types.len();
    let steps = definitions.instantiation_steps();
    definitions.prepare(&mut types, root)?;
    assert_eq!(definitions.instantiation_steps(), steps);
    assert_eq!(types.len(), count);
    let mut layouts = Layouts::default();
    let id = layouts.demand(&types, &definitions, root)?;
    assert_eq!(layouts.at(id).ok_or("layout")?.size, 0);
    let selected = std::collections::HashMap::from([(
        owner,
        Properties {
            copy: true,
            clone_: true,
        },
    )]);
    assert!(
        Copies::default()
            .demand(&types, &definitions, &selected, root)?
            .copy
    );
    assert!(!NeedsDrop::new([]).demand(&types, &definitions, root)?);
    Ok(())
}

#[test]
fn packed_generic_equality_requires_copy_predicates_from_the_same_environment()
-> Result<(), &'static str> {
    use rustgm::types::equality::{Equality, Properties as EqualityProperties};
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let owner = ItemId(0);
    let inner = ItemId(1);
    let parameter = parameter(&mut types, owner, 0)?;
    let pointer = types.intern(Kind::Pointer {
        mutable: false,
        pointee: parameter,
    })?;
    let fields = types.intern_list(&[pointer])?;
    definitions.insert(
        owner,
        aggregate::Kind::Struct,
        Repr {
            packed: Some(1),
            ..Repr::default()
        },
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    define(&mut types, &mut definitions, inner, &[])?;
    let noncopy = nominal(&mut types, inner, &[])?;
    let both = EqualityProperties {
        partial_eq: true,
        eq: true,
    };
    let selected = std::collections::HashMap::from([(owner, both), (inner, both)]);
    let copy_derives = std::collections::HashMap::new();
    let mut copies = Copies::default();
    let mut query = Equality::default();
    for (argument, expected) in [
        (TypeId::primitive(Primitive::U8), both),
        (
            TypeId::primitive(Primitive::F32),
            EqualityProperties {
                partial_eq: true,
                eq: false,
            },
        ),
        (noncopy, EqualityProperties::default()),
    ] {
        let root = nominal(&mut types, owner, &[Argument::Type(argument)])?;
        definitions.prepare_equality(&mut types, root)?;
        assert_eq!(
            Equality::default().demand(&types, &definitions, &selected, root),
            Err("generic packed equality requires Copy obligations")
        );
        assert_eq!(
            query.demand_with_copy(&types, &definitions, &selected, root, |ty| {
                copies
                    .demand(&types, &definitions, &copy_derives, ty)
                    .map(|p| p.copy)
            })?,
            expected
        );
    }
    Ok(())
}

#[test]
fn nominal_query_controls_match_pinned_rustc() -> Result<(), Box<dyn std::error::Error>> {
    // Outcomes are declared before running either compiler. These controls
    // qualify typed query rules; source nominal elaboration is still pending.
    for (name, source, accepted) in [
        (
            "nominal-layout",
            "#[repr(C)]struct S<T>{a:u8,b:T}const _: [();2]=[();size_of::<S<u8>>()];const _: [();16]=[();size_of::<S<u64>>()];const _: [();8]=[();align_of::<S<u64>>()];",
            true,
        ),
        (
            "nominal-enum",
            "#[repr(C,u8)]enum E<'a,T,const N:usize>{A([T;N],&'a T)=17}const _: [();24]=[();size_of::<E<'static,u8,3>>()];const _: [();8]=[();align_of::<E<'static,u8,3>>()];",
            true,
        ),
        (
            "nominal-expanding-pointer",
            "#[repr(C)]struct N<T>{value:T,next:*const N<(T,)>}const _: [();16]=[();size_of::<N<u8>>()];const _: [();24]=[();size_of::<N<N<u8>>>()];",
            true,
        ),
        (
            "nominal-pointer-copy",
            "#[derive(Copy,Clone)]struct P<T>(*const T);fn f<T>(x:P<T>)->(P<T>,P<T>){(x,x)}",
            false,
        ),
        (
            "nominal-pointer-copy-bound",
            "#[derive(Copy,Clone)]struct P<T>(*const T);fn f<T:Copy>(x:P<T>)->(P<T>,P<T>){(x,x)}",
            true,
        ),
        (
            "nominal-function-copy",
            "#[derive(Copy,Clone)]struct P<T>(fn()->T);fn f<T>(x:P<T>)->(P<T>,P<T>){(x,x)}",
            false,
        ),
        (
            "nominal-packed-clone",
            "#[repr(packed)]#[derive(Copy,Clone)]struct P<T>(*const T);#[derive(Clone)]struct C;fn f(x:P<C>)->P<C>{x.clone()}",
            false,
        ),
        (
            "nominal-unpacked-clone",
            "#[derive(Copy,Clone)]struct P<T>(*const T);#[derive(Clone)]struct C;fn f(x:P<C>)->P<C>{x.clone()}",
            true,
        ),
        (
            "nominal-packed-copy-clone",
            "#[repr(packed)]#[derive(Copy,Clone)]struct P<T>(*const T);fn f(x:P<u8>)->P<u8>{x.clone()}",
            true,
        ),
        (
            "nominal-packed-no-copy",
            "#[repr(packed)]#[derive(Copy,Clone)]struct P<T>(*const T);#[derive(Clone)]struct C;fn f(x:P<C>)->(P<C>,P<C>){(x,x)}",
            false,
        ),
        (
            "nominal-exact-copy",
            "#[derive(Copy,Clone)]struct P<T>(*const T);fn f<T>(x:P<T>)->(P<T>,P<T>)where P<T>:Copy{(x,x)}",
            true,
        ),
        (
            "nominal-partial-equality",
            "#[derive(PartialEq,Eq)]struct P<T>(*const T);fn f(x:&P<f32>,y:&P<f32>)->bool{x==y}",
            true,
        ),
        (
            "nominal-packed-equality-copy",
            "#[repr(packed)]#[derive(PartialEq,Eq)]struct P<T>(*const T);fn f(x:P<u8>,y:P<u8>)->bool{x==y}",
            true,
        ),
        (
            "nominal-packed-partial-equality-copy",
            "#[repr(packed)]#[derive(PartialEq,Eq)]struct P<T>(*const T);fn f(x:P<f32>,y:P<f32>)->bool{x==y}",
            true,
        ),
        (
            "nominal-packed-equality-noncopy",
            "#[repr(packed)]#[derive(PartialEq,Eq)]struct P<T>(*const T);#[derive(PartialEq,Eq)]struct C;fn f(x:P<C>,y:P<C>)->bool{x==y}",
            false,
        ),
        (
            "nominal-no-equality",
            "#[derive(PartialEq,Eq)]struct P<T>(*const T);fn need<T:Eq>(){}fn f(){need::<P<f32>>();}",
            false,
        ),
        (
            "nominal-drop-copy-live",
            "struct D<T:Copy>(T);impl<T:Copy> Drop for D<T>{fn drop(&mut self){}}fn f(){let mut n=0u8;let d=D(&n);n=1;}",
            false,
        ),
        (
            "nominal-drop-copy-ended",
            "struct D<T:Copy>(T);impl<T:Copy> Drop for D<T>{fn drop(&mut self){}}fn f(){let mut n=0u8;{let d=D(&n);}n=1;}",
            true,
        ),
    ] {
        rustc::qualify(name, source, accepted)?;
    }
    Ok(())
}
