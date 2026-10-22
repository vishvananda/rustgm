//! Canonical nullable layouts retain exact typed carriers and retryable bounded work.
use super::*;
use crate::types::{
    GenericArgs, ItemId, ListId, RegionId,
    aggregate::{self, Repr},
};
fn reference(types: &mut Store) -> Result<TypeId, &'static str> {
    types.intern(Kind::Reference {
        region: RegionId::STATIC,
        mutable: false,
        referent: TypeId::primitive(crate::types::Primitive::U32),
    })
}
fn wrapper(
    types: &mut Store,
    definitions: &mut Definitions,
    item: ItemId,
    child: TypeId,
) -> Result<TypeId, &'static str> {
    let fields = types.intern_list(&[TypeId::UNIT, child])?;
    definitions.insert(
        item,
        aggregate::Kind::Struct,
        Repr {
            representation: Representation::Transparent,
            ..Repr::default()
        },
        &[aggregate::Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    types.intern(Kind::Adt(item, GenericArgs::EMPTY))
}
fn nullable(
    types: &mut Store,
    definitions: &mut Definitions,
    item: ItemId,
    child: TypeId,
    register: bool,
) -> Result<TypeId, &'static str> {
    let fields = types.intern_list(&[child])?;
    definitions.insert(
        item,
        aggregate::Kind::Enum,
        Repr::default(),
        &[
            aggregate::Variant {
                fields,
                discriminant: 7,
            },
            aggregate::Variant {
                fields: ListId(0),
                discriminant: 19,
            },
        ],
    )?;
    if register {
        definitions.register_nullable_enum(
            types,
            item,
            NullableEnum {
                empty: 1,
                payload: 0,
                field: 0,
            },
        )?;
    }
    types.intern(Kind::Adt(item, GenericArgs::EMPTY))
}
#[test]
fn nullable_layout_is_canonical_and_keeps_ordinary_density() -> Result<(), String> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let word = reference(&mut types)?;
    let ty = nullable(&mut types, &mut definitions, ItemId(8), word, true)?;
    let ordinary = nullable(&mut types, &mut definitions, ItemId(9), word, false)?;
    let mut layouts = Layouts::default();
    layouts
        .demand(&types, &definitions, ordinary)
        .map_err(|e| e.to_string())?;
    assert!(layouts.niches.is_none());
    assert_eq!(layouts.get(ordinary).ok_or("ordinary")?.size, 16);
    layouts
        .demand(&types, &definitions, ty)
        .map_err(|e| e.to_string())?;
    let layout = layouts.get(ty).ok_or("nullable")?;
    assert_eq!((layout.size, layout.align), (8, 8));
    let encoding = layouts.niche(layout)?.ok_or("encoding")?;
    assert_eq!(
        (encoding.empty, encoding.payload, encoding.field),
        (1, 0, 0)
    );
    assert_eq!(
        (encoding.owner, encoding.word, encoding.ty),
        (ty, word, word)
    );
    assert_eq!(layouts.variants(layout)[0].discriminant, 7);
    assert_eq!(layouts.variants(layout)[1].discriminant, 19);
    assert_eq!(
        (
            size_of::<Layout>(),
            size_of::<Shape>(),
            size_of::<Variant>(),
            size_of::<Layouts>()
        ),
        (48, 32, 32, 152)
    );
    Ok(())
}
#[test]
fn nullable_layout_excludes_raw_wide_and_interior_wrappers() -> Result<(), String> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let word = reference(&mut types)?;
    let raw = types.intern(Kind::Pointer {
        mutable: false,
        pointee: TypeId::primitive(crate::types::Primitive::U32),
    })?;
    let slice = types.intern(Kind::Slice(TypeId::primitive(crate::types::Primitive::U8)))?;
    let wide = types.intern(Kind::Reference {
        region: RegionId::STATIC,
        mutable: false,
        referent: slice,
    })?;
    let fields = types.intern_list(&[word])?;
    definitions.insert_with_mutation(
        ItemId(10),
        aggregate::Kind::Struct,
        Repr {
            representation: Representation::Transparent,
            ..Repr::default()
        },
        &[aggregate::Variant {
            fields,
            discriminant: 0,
        }],
        SharedMutation::Interior,
    )?;
    let interior = types.intern(Kind::Adt(ItemId(10), GenericArgs::EMPTY))?;
    for (index, payload) in [raw, wide, interior].into_iter().enumerate() {
        let owner = ItemId(20 + u32::try_from(index).map_err(|e| e.to_string())?);
        let ty = nullable(&mut types, &mut definitions, owner, payload, true)?;
        let mut layouts = Layouts::default();
        layouts
            .demand(&types, &definitions, ty)
            .map_err(|e| e.to_string())?;
        assert!(layouts.niche(layouts.get(ty).ok_or("layout")?)?.is_none());
    }
    Ok(())
}
#[test]
fn nullable_layout_exhaustion_rolls_back_and_allows_retry() -> Result<(), String> {
    for limit in 0..9 {
        let mut types = Store::default();
        let mut definitions = Definitions::default();
        let word = reference(&mut types)?;
        let ty = nullable(&mut types, &mut definitions, ItemId(8), word, true)?;
        let mut layouts = Layouts::default();
        assert!(
            layouts
                .demand_bounded(&types, &definitions, ty, limit)
                .is_err()
        );
        assert!(layouts.get(ty).is_none());
        assert_eq!(layouts.niche_mark(), 0);
        layouts
            .demand_bounded(&types, &definitions, ty, 1000)
            .map_err(|e| e.to_string())?;
        assert_eq!(layouts.niche_mark(), 1);
        assert_eq!(layouts.variants(layouts.get(ty).ok_or("retry")?).len(), 2);
    }
    Ok(())
}
#[test]
fn nullable_carrier_work_depends_on_touched_depth_not_store_or_repetition_product()
-> Result<(), String> {
    for depth in [8u32, 32, 128] {
        let mut observed = Vec::new();
        for noise in [0u64, 1024] {
            let mut types = Store::default();
            let mut definitions = Definitions::default();
            let word = reference(&mut types)?;
            let mut child = word;
            for index in 0..depth {
                child = wrapper(&mut types, &mut definitions, ItemId(100 + index), child)?;
            }
            let ty = nullable(&mut types, &mut definitions, ItemId(8), child, true)?;
            for length in 0..noise {
                types.intern(Kind::Array {
                    element: TypeId::UNIT,
                    length,
                })?;
            }
            let mut layouts = Layouts::default();
            layouts
                .demand(&types, &definitions, ty)
                .map_err(|e| e.to_string())?;
            let cold = layouts.steps;
            let slots = layouts.niches.as_ref().ok_or("table")?.retained();
            let mut rows = Vec::new();
            for repeats in [8, 32, 128] {
                let start = layouts.steps;
                for _ in 0..repeats {
                    assert_eq!(
                        layouts.pointer_carrier(&types, &definitions, child)?,
                        Some(word)
                    );
                }
                let steps = layouts.steps - start;
                assert_eq!(steps, repeats);
                assert_eq!(layouts.niches.as_ref().ok_or("table")?.retained(), slots);
                rows.push(steps);
                println!(
                    "nullable depth={depth} noise={noise} repeats={repeats} cold={cold} warm={steps} bytes={slots}"
                );
            }
            observed.push((cold, slots, rows));
        }
        assert_eq!(observed[0], observed[1]);
    }
    Ok(())
}

#[test]
fn nullable_owner_seals_and_none_inhabitedness_are_independent_of_payload() -> Result<(), String> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let word = reference(&mut types)?;
    definitions.insert(ItemId(40), aggregate::Kind::Enum, Repr::default(), &[])?;
    let empty = types.intern(Kind::Adt(ItemId(40), GenericArgs::EMPTY))?;
    let fields = types.intern_list(&[word, empty])?;
    definitions.insert(
        ItemId(41),
        aggregate::Kind::Struct,
        Repr {
            representation: Representation::Transparent,
            ..Repr::default()
        },
        &[aggregate::Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let payload = types.intern(Kind::Adt(ItemId(41), GenericArgs::EMPTY))?;
    let ty = nullable(&mut types, &mut definitions, ItemId(8), payload, true)?;
    let mut layouts = Layouts::default();
    layouts
        .demand(&types, &definitions, ty)
        .map_err(|e| e.to_string())?;
    assert!(!layouts.get(payload).ok_or("payload")?.inhabited);
    let layout = *layouts.get(ty).ok_or("nullable")?;
    assert!(layout.inhabited);
    let Shape::Niche { encoding, .. } = layout.shape else {
        return Err("niche".into());
    };
    assert_eq!(size_of::<Niche>(), 32);
    layouts.niches.as_mut().ok_or("table")?.records[encoding.0 as usize].owner = word;
    assert!(layouts.niche(&layout).is_err());
    layouts.niches.as_mut().ok_or("table")?.records[encoding.0 as usize].owner = ty;
    assert!(layouts.niche(&layout)?.is_some());
    let mut forged = layout;
    forged.size = 16;
    assert!(layouts.niche(&forged).is_err());
    Ok(())
}
