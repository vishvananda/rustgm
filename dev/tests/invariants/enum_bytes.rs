//! Physical family caches retain identities, never successful value validity.
use super::*;
use crate::{
    const_eval::{Limits, memory::bytes::continuations::Source},
    sema::values::Evaluator,
    test_source,
    types::{GenericArgs, Kind},
};
use std::fmt::Write;
#[path = "c_enum_storage.rs"]
mod c;
#[path = "payload_enum_storage.rs"]
mod payload;
#[path = "tagless_enum_storage.rs"]
mod tagless;
#[path = "wide_enum_storage.rs"]
mod wide;
fn with_enums(
    width: usize,
    noise: usize,
    check: impl FnOnce(&mut Machine<'_, '_>, &[(TypeId, Vec<VariantId>)]) -> Result<(), Failure>,
) -> Result<(), String> {
    let mut source = String::from("#[repr(u8)]enum E{");
    for ordinal in 0..width {
        write!(&mut source, "V{ordinal}={},", ordinal * 2).map_err(|e| e.to_string())?;
    }
    source.push_str("}#[repr(i8)]enum F{A=-1,B=7}");
    with_enum_source(&source, noise, check)
}
fn with_enum_source(
    source: &str,
    noise: usize,
    check: impl FnOnce(&mut Machine<'_, '_>, &[(TypeId, Vec<VariantId>)]) -> Result<(), Failure>,
) -> Result<(), String> {
    let report = test_source::resolve(source, true)?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let mut enums = Vec::new();
    let owners: Vec<_> = facts.aggregates.iter().map(|d| d.item).collect();
    for owner in owners {
        let ty = facts.types.intern(Kind::Adt(owner, GenericArgs::EMPTY))?;
        enums.push((ty, facts.members.variants(owner).to_vec()));
    }
    for length in 0..noise {
        facts.types.intern(Kind::Array {
            element: TypeId::primitive(crate::types::Primitive::U8),
            length: length as u64,
        })?;
    }
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
    for (ty, _) in &enums {
        machine.footprint(*ty).map_err(|e| e.message().to_owned())?;
    }
    check(&mut machine, &enums).map_err(|e| e.message().to_owned())
}
fn root(
    machine: &mut Machine<'_, '_>,
    ty: TypeId,
    variant: VariantId,
) -> Result<(crate::sema::values::LocationId, super::super::Id), Failure> {
    let value = machine.evaluator.fields(&[], Some(variant))?;
    let address = machine.allocate(&mut Vec::new(), Some(value), ty)?;
    let root = address.location().ok_or("enum root")?;
    let id = machine.convert_byte_root(root)?;
    Ok((root, id))
}
#[test]
fn enum_warm_index_rechecks_tag_initialization_and_physical_bounds() -> Result<(), String> {
    with_enums(4, 0, |machine, enums| {
        let (ty, ids) = &enums[0];
        let (root, id) = root(machine, *ty, ids[3])?;
        let value = machine.decode_byte_enum(Source::Backing(id), 0, *ty)?;
        assert!(
            matches!(Reader::new(machine.evaluator.facts()).view(value)?, View::Product {length:0,variant:Some(id)} if id==ids[3])
        );
        machine.fill_byte_root(root, 0, 1, 3)?;
        assert!(
            machine
                .decode_byte_enum(Source::Backing(id), 0, *ty)
                .is_err()
        );
        machine.fill_byte_root(root, 0, 1, 6)?;
        assert!(
            machine
                .decode_byte_enum(Source::Backing(id), 0, *ty)
                .is_ok()
        );
        machine.memory.bytes.roots[id.0 as usize].initialized[0] = 0;
        assert!(
            machine
                .decode_byte_enum(Source::Backing(id), 0, *ty)
                .is_err()
        );
        assert!(
            machine
                .decode_byte_enum(Source::Backing(id), 1, *ty)
                .is_err()
        );
        let value = machine.evaluator.fields(&[], Some(ids[0]))?;
        let mut backing = machine.empty_backing(*ty)?;
        assert!(
            machine
                .encode_byte_enum(&mut backing, 1, *ty, value)
                .is_err()
        );
        Ok(())
    })
}
#[test]
fn enum_encoding_retains_full_owner_and_zero_field_variant_identity() -> Result<(), String> {
    with_enums(4, 0, |machine, enums| {
        let (ty, ids) = &enums[0];
        let (_, foreign) = &enums[1];
        assert!(machine.prepare_byte_enum(*ty)?);
        let mut backing = machine.empty_backing(*ty)?;
        for value in [
            machine.evaluator.fields(&[], Some(foreign[0]))?,
            machine.evaluator.fields(&[], None)?,
            machine.evaluator.fields(
                &[Constant::scalar(crate::sema::model::Scalar::Integer(0))],
                Some(ids[0]),
            )?,
        ] {
            assert!(
                machine
                    .encode_byte_enum(&mut backing, 0, *ty, value)
                    .is_err()
            );
        }
        assert!(
            Reader::new(machine.evaluator.facts())
                .enum_variant_bits(*ty, VariantId(u32::MAX))
                .is_err()
        );
        Ok(())
    })
}
#[test]
fn enum_thawed_cells_keep_their_tag_and_ordinary_values_do_not_allocate_an_index()
-> Result<(), String> {
    with_enums(4, 0, |machine, enums| {
        assert!(machine.memory.bytes.enumerations.is_none());
        let (ty, ids) = &enums[0];
        let value = machine.evaluator.fields(&[], Some(ids[3]))?;
        let root = machine
            .allocate(&mut Vec::new(), Some(value), *ty)?
            .location()
            .ok_or("root")?;
        assert!(machine.memory.bytes.enumerations.is_none());
        let cell = *machine.memory.roots.get(&root).ok_or("cell")?;
        machine.thaw(cell)?;
        let id = machine.convert_byte_root(root)?;
        assert_eq!(machine.byte_backing(id)?.bytes, [6]);
        assert!(machine.decode_byte_enum(Source::Backing(id), 0, *ty)? == value);
        Ok(())
    })
}
#[test]
fn enum_duplicate_physical_tags_cannot_publish_an_index() -> Result<(), String> {
    let report = test_source::resolve("#[repr(u8)]enum E{A=1,B=7}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let item = facts.aggregates.iter().next().ok_or("enum")?.item;
    let ty = facts.types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    facts.aggregates.set_discriminant(item, 1, 1)?;
    facts.layouts = crate::abi::layout::Layouts::default();
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
    machine.footprint(ty).map_err(|e| e.message().to_owned())?;
    let result = machine.prepare_byte_enum(ty);
    assert!(
        matches!(result,Err(error) if error.message()=="stored enum contains duplicate physical tags")
    );
    assert!(machine.memory.bytes.enumerations.is_none());
    Ok(())
}
#[test]
fn enum_index_keys_retain_complete_closed_const_arguments() -> Result<(), String> {
    let report = test_source::resolve(
        "#[repr(u8)]enum E<const N:usize>{A=1,B=7}fn use_types(_:E<1>,_:E<2>){}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let types: Vec<_> = (0..facts.types.len())
        .filter_map(|n| u32::try_from(n).ok().map(TypeId))
        .filter(|&ty| {
            matches!(facts.types.kind(ty), Some(Kind::Adt(..)))
                && facts.types.contains_parameters(ty) == Some(false)
        })
        .collect();
    assert_eq!(types.len(), 2);
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
    for &ty in &types {
        machine.footprint(ty).map_err(|e| e.message().to_owned())?;
        assert!(
            machine
                .prepare_byte_enum(ty)
                .map_err(|e| e.message().to_owned())?
        );
    }
    let cache = machine.memory.bytes.enumerations.as_ref().ok_or("cache")?;
    assert_eq!(cache.index.len(), 2);
    assert!(types.iter().all(|ty| cache.index.contains_key(ty)));
    Ok(())
}
#[test]
fn enum_default_tags_remain_unsupported_while_explicit_payloads_are_supported() -> Result<(), String>
{
    let report =
        test_source::resolve("enum DefaultTag{A,B}#[repr(u8)]enum Payload{A(u8),B}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let owners: Vec<_> = facts.aggregates.iter().map(|d| d.item).collect();
    let mut types = Vec::new();
    for owner in owners {
        types.push(facts.types.intern(Kind::Adt(owner, GenericArgs::EMPTY))?);
    }
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
    let supported: Vec<_> = types
        .into_iter()
        .map(|ty| machine.byte_family(ty))
        .collect::<Result<_, _>>()
        .map_err(|e| e.message().to_owned())?;
    assert_eq!(supported.iter().filter(|&&yes| yes).count(), 1);
    assert_eq!(supported.iter().filter(|&&yes| !yes).count(), 1);
    assert!(machine.memory.bytes.enumerations.is_some());
    Ok(())
}
#[test]
fn enum_index_failures_do_not_publish_partial_family_or_poison_other_types() -> Result<(), String> {
    with_enums(128, 0, |machine, enums| {
        let ty = enums[0].0;
        machine.limits.slots = 1;
        assert!(machine.prepare_byte_enum(ty).is_err());
        assert!(machine.memory.bytes.enumerations.is_none());
        machine.limits.slots = 1 << 20;
        machine.limits.steps = machine.work.steps + 1;
        assert!(machine.prepare_byte_enum(ty).is_err());
        assert!(machine.memory.bytes.enumerations.is_none());
        machine.limits.steps = 1 << 20;
        assert!(machine.prepare_byte_enum(enums[1].0)?);
        assert!(machine.prepare_byte_enum(ty)?);
        assert_eq!(
            machine
                .memory
                .bytes
                .enumerations
                .as_ref()
                .ok_or("cache")?
                .index
                .len(),
            2
        );
        Ok(())
    })
}
#[test]
fn enum_index_warm_work_is_logarithmic_and_independent_of_unrelated_store_growth()
-> Result<(), String> {
    for width in [8usize, 32, 128] {
        let mut observations = Vec::new();
        for noise in [0, 1024] {
            with_enums(width, noise, |machine, enums| {
                let (ty, ids) = &enums[0];
                let before = machine.work.steps;
                let (_, id) = root(machine, *ty, ids[width - 1])?;
                let cold = machine.work.steps - before;
                let retained = machine.memory.retained_cells();
                let mut rows = Vec::new();
                for repeats in [8usize, 32, 128] {
                    let before = machine.work.steps;
                    for _ in 0..repeats {
                        machine.decode_byte_enum(Source::Backing(id), 0, *ty)?;
                    }
                    let work = machine.work.steps - before;
                    assert!(work <= repeats * (width.ilog2() as usize + 16));
                    assert_eq!(machine.memory.retained_cells(), retained);
                    rows.push(work);
                    println!(
                        "enum variants={width} noise={noise} repeats={repeats} cold={cold} warm={work} retained={retained}"
                    );
                }
                assert_eq!(rows[2] - rows[1], 4 * (rows[1] - rows[0]));
                machine.limits.slots = retained;
                let before = machine.work.steps;
                for _ in 0..128 {
                    assert!(machine.prepare_byte_enum(*ty)?);
                }
                assert_eq!(machine.work.steps - before, 128);
                assert_eq!(machine.memory.retained_cells(), retained);
                observations.push((cold, rows, retained));
                Ok(())
            })?;
        }
        assert_eq!(observations[0], observations[1]);
    }
    assert_eq!(size_of::<crate::sema::values::StoredShape<'_>>(), 32);
    println!(
        "enum StoredShape={} test-Storage={}",
        size_of::<crate::sema::values::StoredShape<'_>>(),
        size_of::<super::super::Storage>()
    );
    Ok(())
}
