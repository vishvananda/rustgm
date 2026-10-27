//! Inferred C tags and selected Rust tags use one retained tag lookup algorithm.
use super::payload::{integer, place};
use super::*;
use crate::const_eval::memory::{Read, UnionField};
use crate::sema::members::{FieldId, Selection};
use crate::types::Primitive;
#[test]
fn active_union_tags_keep_exact_fields_without_reading_payloads() -> Result<(), String> {
    for declaration in [
        "#[repr(C)]enum E{A(u32),Empty}",
        "enum E{A(u32),Empty}",
        "enum E{Only(u32)}",
    ] {
        let source = format!("#[derive(Copy,Clone)]{declaration}union U{{e:E,b:[u8;16]}}");
        with_enum_source(&source, 0, |machine, enums| {
            let (ty, ids) = &enums[0];
            let (union, groups) = &enums[1];
            let field =
                FieldId(machine.evaluator.facts().members.variants[groups[0].0 as usize].start);
            let value = machine.evaluator.fields(&[integer(37)], Some(ids[0]))?;
            let image = machine.make_union(*union, Selection::Nominal(field), value)?;
            let address = machine.allocate(&mut Vec::new(), Some(image), *union)?;
            let selected = crate::const_eval::memory::Resolved {
                union: Some(UnionField {
                    owner: *union,
                    field: Selection::Nominal(field),
                }),
                ..place(address, *ty)
            };
            assert!(
                matches!(machine.read_active_variant(selected)?, Read::Value(id) if id == ids[0])
            );
            assert!(
                machine
                    .read_active_variant(crate::const_eval::memory::Resolved {
                        ty: TypeId::primitive(Primitive::U32),
                        ..selected
                    })
                    .is_err()
            );
            let root = address.location().ok_or("union root")?;
            let backing = machine.convert_byte_root(root)?;
            let (_, offsets) =
                Reader::new(machine.evaluator.facts()).enum_variant_fields(*ty, ids[0])?;
            let start = usize::try_from(offsets[0]).map_err(|_| "payload offset")?;
            for index in start..start + 4 {
                machine.memory.bytes.roots[backing.0 as usize].initialized[index / 64] &=
                    !(1 << (index % 64));
            }
            assert!(
                matches!(machine.read_active_variant(selected)?, Read::Value(id) if id == ids[0])
            );
            assert!(machine.read(selected).is_err());
            machine.memory.expire(&[root]);
            assert!(machine.read_active_variant(selected).is_err());
            Ok(())
        })?;
    }
    Ok(())
}
#[test]
fn c_enum_tag_families_follow_retained_signed_and_unsigned_abi_facts() -> Result<(), String> {
    for (variants, tag, bits) in [
        ("A=3,B=7", Primitive::U32, 3u128),
        ("A=-1,B=7", Primitive::I32, u128::from(u32::MAX)),
        ("A=4294967296,B=4294967297", Primitive::U64, 1u128 << 32),
        (
            "A=-2147483649,B=7",
            Primitive::I64,
            u128::from(u64::MAX) - (1u128 << 31),
        ),
    ] {
        with_enum_source(
            &format!("#[repr(C)]enum E{{{variants}}}"),
            0,
            |machine, enums| {
                let (ty, ids) = &enums[0];
                let reader = Reader::new(machine.evaluator.facts());
                assert_eq!(reader.enum_variants(*ty)?.tag.ty, tag);
                assert_eq!(reader.enum_variant_bits(*ty, ids[0])?, bits);
                let (_, id) = root(machine, *ty, ids[0])?;
                let value = machine.decode_byte_enum(Source::Backing(id), 0, *ty)?;
                assert!(
                    matches!(Reader::new(machine.evaluator.facts()).view(value)?,View::Product{length:0,variant:Some(id)} if id==ids[0])
                );
                Ok(())
            },
        )?;
    }
    Ok(())
}
#[test]
fn c_enum_warm_tag_work_is_bounded_by_width_and_logarithmic_lookup() -> Result<(), String> {
    for (base, width) in [(0u128, 4usize), (1u128 << 32, 8usize)] {
        warm_tag_work("#[repr(C)]", base, width)?;
    }
    Ok(())
}
pub(super) fn warm_tag_work(repr: &str, base: u128, width: usize) -> Result<(), String> {
    for count in [8usize, 32, 128] {
        let mut text = format!("{repr}enum E{{");
        for n in 0..count {
            write!(&mut text, "V{n}={},", base + 2 * n as u128).map_err(|e| e.to_string())?;
        }
        text.push('}');
        let mut observations = Vec::new();
        for noise in [0, 1024] {
            with_enum_source(&text, noise, |machine, enums| {
                let (ty, ids) = &enums[0];
                let before = machine.work.steps;
                let (_, id) = root(machine, *ty, ids[count - 1])?;
                let cold = machine.work.steps - before;
                let retained = machine.memory.retained_cells();
                let mut rows = Vec::new();
                for repeats in [8usize, 32, 128] {
                    let before = machine.work.steps;
                    for _ in 0..repeats {
                        machine.decode_byte_enum(Source::Backing(id), 0, *ty)?;
                    }
                    let steps = machine.work.steps - before;
                    assert_eq!(steps, repeats * (count.ilog2() as usize + width + 3));
                    assert_eq!(machine.memory.retained_cells(), retained);
                    rows.push(steps);
                    println!(
                        "c-enum width={width} variants={count} noise={noise} repeats={repeats} cold={cold} warm={steps} retained={retained}"
                    );
                }
                observations.push((cold, retained, rows));
                Ok(())
            })?;
        }
        assert_eq!(observations[0], observations[1]);
    }
    Ok(())
}
