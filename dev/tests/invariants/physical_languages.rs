//! Actual ABI producers feed independent scalar, padding, tag and reference controls.
use super::*;
use crate::abi::transmutability::{Assumptions, Condition, Limits, Obligation, Proof, decide};
use crate::types::{
    GenericArgs, ItemId, ListId, RegionId,
    aggregate::{self, Repr, Representation},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn boolean(proof: &Proof) -> Result<bool> {
    match proof.condition(proof.root) {
        Some(Condition::Yes) => Ok(true),
        Some(Condition::No) => Ok(false),
        _ => Err("unexpected conditional byte relation".into()),
    }
}
fn nominal(
    types: &mut Store,
    definitions: &mut Definitions,
    item: u32,
    kind: aggregate::Kind,
    repr: Repr,
    fields: &[&[TypeId]],
) -> Result<TypeId> {
    let variants = fields
        .iter()
        .enumerate()
        .map(|(i, fields)| {
            Ok(aggregate::Variant {
                fields: types.intern_list(fields)?,
                discriminant: i as u128,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    definitions.insert(ItemId(item), kind, repr, &variants)?;
    Ok(types.intern(Kind::Adt(ItemId(item), GenericArgs::EMPTY))?)
}
fn relation(
    types: &Store,
    definitions: &Definitions,
    layouts: &mut Layouts,
    roots: [TypeId; 2],
    assumptions: Assumptions,
) -> Result<bool> {
    for ty in roots {
        layouts.demand(types, definitions, ty)?;
    }
    let mut pay = |_, _| Ok(());
    let mut budget = Budget::new(Limits::default(), &mut pay);
    let (mut language, [source, destination]) =
        describe(types, definitions, layouts, roots, &mut budget)?;
    boolean(&decide(
        &mut language,
        source,
        destination,
        assumptions,
        &mut budget,
    )?)
}
#[test]
fn actual_primitive_and_billion_element_layouts_retain_validity_without_expansion() -> Result {
    let mut types = Store::default();
    let definitions = Definitions::default();
    let mut layouts = Layouts::default();
    let byte = TypeId::primitive(Primitive::U8);
    let truth = TypeId::primitive(Primitive::Bool);
    let character = TypeId::primitive(Primitive::Char);
    let integer = TypeId::primitive(Primitive::U32);
    for (roots, expected) in [
        ([truth, byte], true),
        ([byte, truth], false),
        ([character, integer], true),
        ([integer, character], false),
        ([byte, TypeId::UNIT], true),
        ([TypeId::UNIT, byte], false),
    ] {
        assert_eq!(
            relation(
                &types,
                &definitions,
                &mut layouts,
                roots,
                Assumptions::default()
            )?,
            expected
        );
    }
    let characters = types.intern(Kind::Array {
        element: character,
        length: 1_000_000_000,
    })?;
    let bytes = types.intern(Kind::Array {
        element: byte,
        length: 4_000_000_000,
    })?;
    for ty in [characters, bytes] {
        layouts.demand(&types, &definitions, ty)?;
    }
    let mut pay = |_, _| Ok(());
    let mut budget = Budget::new(Limits::default(), &mut pay);
    let (mut language, [source, destination]) = describe(
        &types,
        &definitions,
        &layouts,
        [characters, bytes],
        &mut budget,
    )?;
    assert!(boolean(&decide(
        &mut language,
        source,
        destination,
        Assumptions::default(),
        &mut budget
    )?)?);
    assert!(!boolean(&decide(
        &mut language,
        destination,
        source,
        Assumptions::default(),
        &mut budget
    )?)?);
    assert!(language.len() < 256);
    assert!(budget.consumed().0 < 30_000);
    Ok(())
}
#[test]
fn actual_padding_and_union_alternatives_use_physical_offsets() -> Result {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let mut layouts = Layouts::default();
    let byte = TypeId::primitive(Primitive::U8);
    let truth = TypeId::primitive(Primitive::Bool);
    let integer = TypeId::primitive(Primitive::U32);
    let wide = TypeId::primitive(Primitive::U64);
    let structure = nominal(
        &mut types,
        &mut definitions,
        10,
        aggregate::Kind::Struct,
        Repr {
            representation: Representation::C,
            ..Repr::default()
        },
        &[&[byte, integer]],
    )?;
    let union = nominal(
        &mut types,
        &mut definitions,
        11,
        aggregate::Kind::Union,
        Repr::default(),
        &[&[truth, byte]],
    )?;
    let assumed = Assumptions {
        safety: true,
        ..Assumptions::default()
    };
    assert!(!relation(
        &types,
        &definitions,
        &mut layouts,
        [structure, wide],
        assumed
    )?);
    assert!(relation(
        &types,
        &definitions,
        &mut layouts,
        [wide, structure],
        assumed
    )?);
    assert!(relation(
        &types,
        &definitions,
        &mut layouts,
        [union, byte],
        Assumptions::default()
    )?);
    assert!(!relation(
        &types,
        &definitions,
        &mut layouts,
        [byte, union],
        Assumptions::default()
    )?);
    assert!(relation(
        &types,
        &definitions,
        &mut layouts,
        [byte, union],
        assumed
    )?);
    Ok(())
}
#[test]
fn actual_tagged_enum_preserves_independent_word_sets() -> Result {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let mut layouts = Layouts::default();
    let byte = TypeId::primitive(Primitive::U8);
    let truth = TypeId::primitive(Primitive::Bool);
    let enumeration = nominal(
        &mut types,
        &mut definitions,
        12,
        aggregate::Kind::Enum,
        Repr {
            tag: Some(Primitive::U8),
            ..Repr::default()
        },
        &[&[truth], &[byte]],
    )?;
    let assumed = Assumptions {
        safety: true,
        ..Assumptions::default()
    };
    layouts.demand(&types, &definitions, enumeration)?;
    let mut pay = |_, _| Ok(());
    let mut budget = Budget::new(Limits::default(), &mut pay);
    let (mut language, [destination, _]) = describe(
        &types,
        &definitions,
        &layouts,
        [enumeration; 2],
        &mut budget,
    )?;
    assert_eq!(layouts.get(enumeration).ok_or("enum layout")?.size, 2);
    for tag in [0_u16, 1, 2, 255] {
        for value in [0_u16, 1, 2, 255] {
            let first = language.byte(ByteRange::new(tag, tag + 1)?, &mut budget)?;
            let last = language.byte(ByteRange::new(value, value + 1)?, &mut budget)?;
            let source = language.sequence(first, last, &mut budget)?;
            assert_eq!(
                boolean(&decide(
                    &mut language,
                    source,
                    destination,
                    assumed,
                    &mut budget
                )?)?,
                tag == 0 && value <= 1 || tag == 1,
                "tag={tag}, payload={value}"
            );
        }
    }
    Ok(())
}
#[test]
fn completed_reference_layouts_keep_real_referents_regions_and_unsupported_opponents() -> Result {
    let mut types = Store::default();
    let definitions = Definitions::default();
    let mut layouts = Layouts::default();
    let integer = TypeId::primitive(Primitive::U32);
    let source = types.intern(Kind::Reference {
        mutable: true,
        region: RegionId(7),
        referent: integer,
    })?;
    let destination = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId(8),
        referent: integer,
    })?;
    for ty in [integer, source, destination] {
        layouts.demand(&types, &definitions, ty)?;
    }
    let mut pay = |_, _| Ok(());
    let mut budget = Budget::new(Limits::default(), &mut pay);
    let (mut language, [source, destination]) = describe(
        &types,
        &definitions,
        &layouts,
        [source, destination],
        &mut budget,
    )?;
    let proof = decide(
        &mut language,
        source,
        destination,
        Assumptions::default(),
        &mut budget,
    )?;
    let mut leaves = Vec::new();
    let mut stack = vec![proof.root];
    while let Some(id) = stack.pop() {
        match proof.condition(id).ok_or("condition")? {
            Condition::All(first, last) | Condition::Any(first, last) => {
                stack.push(first);
                stack.push(last);
            }
            Condition::Obligation(value) => leaves.push(value),
            _ => {}
        }
    }
    assert!(leaves.contains(&Obligation::Transmutable {
        source: integer,
        destination: integer
    }));
    assert!(leaves.contains(&Obligation::Outlives {
        long: RegionId(7),
        short: RegionId(8)
    }));
    assert!(leaves.contains(&Obligation::Freeze(integer)));
    let pointer = types.intern(Kind::Pointer {
        mutable: false,
        pointee: integer,
    })?;
    layouts.demand(&types, &definitions, pointer)?;
    assert!(describe(&types, &definitions, &layouts, [pointer; 2], &mut budget).is_err());
    let function = types.intern(Kind::Function {
        bound_regions: 0,
        parameters: ListId(0),
        result: TypeId::UNIT,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    layouts.demand(&types, &definitions, function)?;
    assert!(describe(&types, &definitions, &layouts, [function; 2], &mut budget).is_err());
    Ok(())
}
