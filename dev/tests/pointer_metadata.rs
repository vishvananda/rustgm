//! PA22 storage-tail classification does not imply a source-level coercion proof.
use rustgm::{
    abi::{
        layout::{Layouts, Shape},
        metadata::{Metadata, Queries},
    },
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
        aggregate::{self, Definitions, Repr, Variant},
        arguments::Argument,
        parameters::{Parameter, ParameterKind},
    },
};

const U8: TypeId = TypeId::primitive(Primitive::U8);
const STR: TypeId = TypeId::primitive(Primitive::Str);

fn tuple(types: &mut Store, fields: &[TypeId]) -> Result<TypeId, &'static str> {
    let fields = types.intern_list(fields)?;
    types.intern(Kind::Tuple(fields))
}

fn record(
    types: &mut Store,
    definitions: &mut Definitions,
    item: ItemId,
    fields: &[TypeId],
) -> Result<TypeId, &'static str> {
    let fields = types.intern_list(fields)?;
    definitions.insert(
        item,
        aggregate::Kind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    types.intern(Kind::Adt(item, GenericArgs::EMPTY))
}

#[test]
fn tails_inherit_metadata_without_inventing_a_value_layout()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let mut query = Queries::default();
    let slice = types.intern(Kind::Slice(U8))?;
    for (index, tail) in [slice, STR, U8, TypeId::UNIT].into_iter().enumerate() {
        let expected = if index < 2 {
            Metadata::Length
        } else {
            Metadata::Thin
        };
        let inner = tuple(&mut types, &[U8, tail])?;
        let outer = record(
            &mut types,
            &mut definitions,
            ItemId(u32::try_from(index)?),
            &[U8, inner],
        )?;
        assert_eq!(query.demand(&types, &definitions, outer)?, expected);
        assert_eq!(query.demand(&types, &definitions, inner)?, expected);
        for pointer in [
            Kind::Reference {
                referent: outer,
                mutable: false,
                region: RegionId::STATIC,
            },
            Kind::Pointer {
                pointee: outer,
                mutable: true,
            },
        ] {
            let pointer = types.intern(pointer)?;
            assert_eq!(query.demand(&types, &definitions, pointer)?, Metadata::Thin);
        }
    }
    let unsized_tuple = tuple(&mut types, &[U8, slice])?;
    let reference = types.intern(Kind::Reference {
        referent: unsized_tuple,
        mutable: false,
        region: RegionId::STATIC,
    })?;
    let mut layouts = Layouts::default();
    let layout = layouts.demand(&types, &definitions, reference)?;
    let layout = layouts.at(layout).ok_or("missing tuple pointer layout")?;
    assert_eq!((layout.size, layout.align), (16, 8));
    assert!(matches!(layout.shape, Shape::WidePointer));
    assert!(layouts.demand(&types, &definitions, unsized_tuple).is_err());
    Ok(())
}

#[test]
fn unknown_tails_require_substitution_and_do_not_grant_sized()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let mut query = Queries::default();
    let owner = ItemId(0);
    let parameter = types.intern_parameter(Parameter {
        owner,
        index: 0,
        kind: ParameterKind::Type { sized: false },
    })?;
    let parameter = types.intern(Kind::Parameter(parameter))?;
    let _ = record(&mut types, &mut definitions, owner, &[U8, parameter])?;
    let arguments = types.intern_arguments(&[Argument::Type(parameter)])?;
    let template = types.intern(Kind::Adt(owner, arguments))?;
    definitions.instantiate(&mut types, template)?;
    assert_eq!(
        query.demand(&types, &definitions, template)?,
        Metadata::Unknown(parameter)
    );
    for (argument, expected) in [(STR, Metadata::Length), (U8, Metadata::Thin)] {
        let arguments = types.intern_arguments(&[Argument::Type(argument)])?;
        let instance = types.intern(Kind::Adt(owner, arguments))?;
        definitions.instantiate(&mut types, instance)?;
        assert_eq!(query.demand(&types, &definitions, instance)?, expected);
    }
    assert_eq!(types.is_sized(parameter), Some(false));
    assert_eq!(
        query.demand(&types, &definitions, template)?,
        Metadata::Unknown(parameter)
    );
    Ok(())
}

#[test]
fn deep_shared_tails_have_linear_work_and_constant_time_reuse()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let mut query = Queries::default();
    let mut chain = vec![STR];
    for index in 0..8192 {
        let tail = *chain.last().ok_or("missing tail")?;
        chain.push(record(
            &mut types,
            &mut definitions,
            ItemId(index),
            &[U8, tail],
        )?);
    }
    for &ty in chain.iter().rev() {
        assert_eq!(query.demand(&types, &definitions, ty)?, Metadata::Length);
    }
    assert_eq!(query.steps, chain.len());
    for index in 8192..16384 {
        let root = record(
            &mut types,
            &mut definitions,
            ItemId(index),
            &[*chain.last().ok_or("missing shared tail")?],
        )?;
        assert_eq!(query.demand(&types, &definitions, root)?, Metadata::Length);
    }
    assert_eq!(query.steps, chain.len() + 2 * 8192);
    Ok(())
}

#[test]
fn failed_cycles_are_cached_without_poisoning_independent_queries()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let root = types.intern(Kind::Adt(ItemId(0), GenericArgs::EMPTY))?;
    let _ = record(&mut types, &mut definitions, ItemId(0), &[root])?;
    let mut query = Queries::default();
    assert_eq!(
        query.demand(&types, &definitions, root),
        Err("recursive unsized storage tail")
    );
    let steps = query.steps;
    assert!(query.demand(&types, &definitions, root).is_err());
    assert_eq!(query.steps, steps);
    assert_eq!(query.demand(&types, &definitions, STR)?, Metadata::Length);
    assert_eq!(query.demand(&types, &definitions, U8)?, Metadata::Thin);
    Ok(())
}

#[test]
fn exhausted_metadata_work_preserves_completed_results() -> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let definitions = Definitions::default();
    let inner = tuple(&mut types, &[U8, STR])?;
    let outer = tuple(&mut types, &[U8, inner])?;
    let mut query = Queries::default();
    assert_eq!(query.demand(&types, &definitions, U8)?, Metadata::Thin);
    query.steps = 999_999;
    assert_eq!(
        query.demand(&types, &definitions, outer),
        Err("pointer metadata work budget exceeded")
    );
    let steps = query.steps;
    assert!(query.demand(&types, &definitions, outer).is_err());
    assert_eq!(query.demand(&types, &definitions, U8)?, Metadata::Thin);
    assert_eq!(query.steps, steps);
    Ok(())
}
