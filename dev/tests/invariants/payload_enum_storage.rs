//! Payload field validity follows the selected canonical variant, never inactive bytes.
use super::*;
use crate::{
    const_eval::{
        memory::{RawAccess, RawAlignment, Read, Resolved, bytes::Id},
        static_reads::ReadPurpose,
    },
    sema::{
        members::{FieldId, Selection},
        model::Scalar,
        values::WritePermission,
    },
    types::Primitive,
};
fn integer(value: u128) -> Constant {
    Constant::scalar(Scalar::Integer(value))
}
fn place(address: Constant, ty: TypeId) -> Resolved {
    Resolved {
        address,
        ty,
        raw: None,
        union: None,
    }
}
fn raw(address: Constant, ty: TypeId) -> Resolved {
    Resolved {
        raw: Some(RawAccess {
            alignment: RawAlignment::Required,
            location: address.location(),
            ty,
        }),
        ..place(address, ty)
    }
}
fn payload_root(
    machine: &mut Machine<'_, '_>,
    ty: TypeId,
    id: VariantId,
    fields: &[Constant],
) -> Result<(Constant, Id), Failure> {
    let value = machine.evaluator.fields(fields, Some(id))?;
    let address = machine.allocate(&mut Vec::new(), Some(value), ty)?;
    let root = address.location().ok_or("root")?;
    let cell = *machine.memory.roots.get(&root).ok_or("root cell")?;
    machine.thaw(cell)?;
    let id = machine.convert_byte_root(root)?;
    Ok((address, id))
}
#[test]
fn payload_enum_padding_and_inactive_bytes_do_not_acquire_validity() -> Result<(), String> {
    with_enum_source(
        "#[repr(u8)]enum E{A(u32),B(bool),Empty}",
        0,
        |machine, enums| {
            let (ty, ids) = &enums[0];
            let (address, id) = payload_root(machine, *ty, ids[0], &[integer(37)])?;
            let root = address.location().ok_or("root")?;
            let backing = machine.byte_backing(id)?;
            assert_eq!(backing.bytes, [0, 0, 0, 0, 37, 0, 0, 0]);
            assert_eq!(backing.initialized[0], 0xf1);
            machine.fill_byte_root(root, 1, 1, 2)?;
            assert!(matches!(
                machine.decode_bytes(id, 0, *ty, ReadPurpose::Value)?,
                Read::Value(_)
            ));
            machine.fill_byte_root(root, 0, 1, 1)?;
            assert!(
                machine
                    .decode_bytes(id, 0, *ty, ReadPurpose::Value)
                    .is_err()
            );
            machine.fill_byte_root(root, 1, 1, 1)?;
            assert!(matches!(
                machine.decode_bytes(id, 0, *ty, ReadPurpose::Value)?,
                Read::Value(_)
            ));
            machine.fill_byte_root(root, 0, 1, 2)?;
            machine.memory.bytes.roots[id.0 as usize].initialized[0] = 1;
            let Read::Value(value) = machine.decode_bytes(id, 0, *ty, ReadPurpose::Value)? else {
                return Err("empty pending".into());
            };
            assert!(
                matches!(Reader::new(machine.evaluator.facts()).view(value)?,View::Product{length:0,variant:Some(id)}if id==ids[2])
            );
            machine.fill_byte_root(root, 0, 1, 0)?;
            assert!(
                machine
                    .decode_bytes(id, 0, *ty, ReadPurpose::Value)
                    .is_err()
            );
            Ok(())
        },
    )
}
#[test]
fn payload_enum_field_addresses_keep_physical_offsets_and_pointer_restrictions()
-> Result<(), String> {
    with_enum_source(
        "#[repr(u8)]enum E{A(u32),B(bool)}#[repr(u8)]enum F{A(u32),B(bool)}",
        0,
        |machine, enums| {
            let (ty, ids) = &enums[0];
            let foreign = enums[1].1[0];
            let word = TypeId::primitive(Primitive::U32);
            let boolean = TypeId::primitive(Primitive::Bool);
            let value = machine.evaluator.fields(&[integer(37)], Some(ids[0]))?;
            let address = machine.allocate(&mut Vec::new(), Some(value), *ty)?;
            let root = address.location().ok_or("root")?;
            let field =
                FieldId(machine.evaluator.facts().members.variants[ids[0].0 as usize].start);
            let foreign_field =
                FieldId(machine.evaluator.facts().members.variants[foreign.0 as usize].start);
            assert!(
                machine
                    .field_projection(place(address, *ty), Selection::Nominal(foreign_field), word)
                    .is_err()
            );
            assert!(
                machine
                    .field_projection(place(address, *ty), Selection::Nominal(field), boolean)
                    .is_err()
            );
            let selected =
                machine.field_projection(place(address, *ty), Selection::Nominal(field), word)?;
            let shared = machine.field_projection(
                place(address.restrict_shared(), *ty),
                Selection::Nominal(field),
                word,
            )?;
            assert!(machine.byte_root(root).is_none());
            assert_eq!(
                shared.write_permission(),
                Some(WritePermission::SharedDerived)
            );
            assert!(machine.read(raw(selected, word))?.ok_or("field")? == integer(37));
            machine.fill_byte_root(root, 0, 1, 1)?;
            machine.fill_byte_root(root, 1, 1, 1)?;
            machine.write_place(raw(selected, word), integer(41))?;
            assert!(machine.read(raw(selected, word))?.ok_or("old field")? == integer(41));
            assert!(machine.write_place(raw(shared, word), integer(9)).is_err());
            let id = machine.byte_root(root).ok_or("backing")?;
            let Read::Value(value) = machine.decode_bytes(id, 0, *ty, ReadPurpose::Value)? else {
                return Err("pending".into());
            };
            assert!(
                matches!(Reader::new(machine.evaluator.facts()).view(value)?,View::Product{length:1,variant:Some(id)}if id==ids[1])
            );
            machine.memory.expire(&[root]);
            assert!(machine.read(raw(selected, word)).is_err());
            Ok(())
        },
    )
}
#[test]
fn payload_enum_repeated_field_access_reuses_one_backing_and_has_width_independent_work()
-> Result<(), String> {
    let mut expected = None;
    for width in [8u64, 32, 128, 4096] {
        let source = format!("#[repr(u8)]enum E{{A(u32,[u8;{width}]),Empty}}");
        let mut observations = Vec::new();
        for noise in [0, 1024] {
            with_enum_source(&source, noise, |machine, enums| {
                let (ty, ids) = &enums[0];
                let word = TypeId::primitive(Primitive::U32);
                let array = machine.evaluator.repeat(integer(7), width)?;
                let value = machine
                    .evaluator
                    .fields(&[integer(37), array], Some(ids[0]))?;
                let address = machine.allocate(&mut Vec::new(), Some(value), *ty)?;
                let owner = place(address, *ty);
                let field = Selection::Nominal(FieldId(
                    machine.evaluator.facts().members.variants[ids[0].0 as usize].start,
                ));
                assert!(machine.memory.bytes.enumerations.is_none());
                let before = machine.work.steps;
                let selected = machine.field_projection(owner, field, word)?;
                machine.read(raw(selected, word))?.ok_or("warm field")?;
                let cold = machine.work.steps - before;
                let retained = machine.memory.retained_cells();
                let count = machine.memory.bytes.roots.len();
                let mut rows = Vec::new();
                for repeats in [8usize, 32, 128] {
                    let before = machine.work.steps;
                    for _ in 0..repeats {
                        let selected = machine.field_projection(owner, field, word)?;
                        machine.write_place(raw(selected, word), integer(41))?;
                        assert!(machine.read(raw(selected, word))?.ok_or("field")? == integer(41));
                    }
                    let steps = machine.work.steps - before;
                    rows.push(steps);
                    assert_eq!(machine.memory.bytes.roots.len(), count);
                    assert_eq!(machine.memory.retained_cells(), retained);
                    println!(
                        "payload-enum width={width} noise={noise} repeats={repeats} cold={cold} warm={steps} retained={retained}"
                    );
                }
                assert_eq!(rows[2] - rows[1], 4 * (rows[1] - rows[0]));
                if let Some(prior) = &expected {
                    assert_eq!(&rows, prior);
                } else {
                    expected = Some(rows.clone());
                }
                observations.push((cold, retained, rows));
                Ok(())
            })?;
        }
        assert_eq!(observations[0], observations[1]);
    }
    Ok(())
}
#[test]
fn payload_enum_reference_fields_repeat_actual_referent_validity_on_warm_cache()
-> Result<(), String> {
    with_enum_source(
        "#[repr(u8)]enum E{A(&'static u32),Empty}",
        0,
        |machine, enums| {
            let (ty, ids) = &enums[0];
            let referent = machine.allocate(
                &mut Vec::new(),
                Some(integer(37)),
                TypeId::primitive(Primitive::U32),
            )?;
            let (_, id) = payload_root(machine, *ty, ids[0], &[referent.restrict_shared()])?;
            machine.read_operation = Some(crate::const_eval::ReadOperation {
                frame: 1,
                position: 0,
            });
            let before = machine.memory.bytes.checked_duties;
            assert!(matches!(
                machine.decode_bytes(id, 0, *ty, ReadPurpose::Value)?,
                Read::Value(_)
            ));
            assert_eq!(machine.memory.bytes.checked_duties, before + 1);
            machine
                .memory
                .expire(&[referent.location().ok_or("referent")?]);
            assert!(
                machine
                    .decode_bytes(id, 0, *ty, ReadPurpose::Value)
                    .is_err()
            );
            assert_eq!(machine.memory.bytes.checked_duties, before + 2);
            Ok(())
        },
    )
}
