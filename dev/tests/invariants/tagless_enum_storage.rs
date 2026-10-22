//! Unique retained no-tag layouts use products without losing their sole variant.
use super::payload::{integer, payload_root, place, raw};
use super::*;
use crate::{
    const_eval::{
        memory::{Cell, Read},
        static_reads::ReadPurpose,
    },
    sema::{
        members::{FieldId, Selection},
        values::{Origin, StoredShape, WritePermission},
    },
    types::Primitive,
};
#[test]
fn tagless_enum_zero_size_encoding_rejects_forged_variants_and_thawed_cells() -> Result<(), String>
{
    with_enum_source("enum E{Only}enum F{Other}", 0, |machine, enums| {
        let (ty, ids) = &enums[0];
        let foreign = enums[1].1[0];
        assert!(
            matches!(Reader::new(machine.evaluator.facts()).storage_shape(*ty)?,StoredShape::Product{variant:Some(id),fields,..} if id==ids[0]&&fields.is_empty())
        );
        let mut backing = machine.empty_backing(*ty)?;
        for value in [
            machine.evaluator.fields(&[], Some(foreign))?,
            machine.evaluator.fields(&[], None)?,
            machine.evaluator.fields(&[integer(1)], Some(ids[0]))?,
        ] {
            assert!(machine.encode_bytes(&mut backing, 0, *ty, value).is_err());
        }
        let value = machine.evaluator.fields(&[], Some(ids[0]))?;
        let address = machine.allocate(&mut Vec::new(), Some(value), *ty)?;
        let root = address.location().ok_or("root")?;
        let cell = *machine.memory.roots.get(&root).ok_or("cell")?;
        machine.thaw(cell)?;
        let Cell::Fields { variant, .. } = &mut machine.memory.cells[cell] else {
            return Err("fields".into());
        };
        *variant = Some(foreign);
        assert!(machine.convert_byte_root(root).is_err());
        assert!(machine.byte_root(root).is_none());
        let Cell::Fields { variant, .. } = &mut machine.memory.cells[cell] else {
            return Err("fields".into());
        };
        *variant = Some(ids[0]);
        let id = machine.convert_byte_root(root)?;
        let Read::Value(decoded) = machine.decode_bytes(id, 0, *ty, ReadPurpose::Value)? else {
            return Err("pending".into());
        };
        assert!(decoded == value);
        assert!(machine.memory.bytes.enumerations.is_none());
        Ok(())
    })
}
#[test]
fn tagless_enum_field_projections_keep_ancestry_and_live_write_restrictions() -> Result<(), String>
{
    with_enum_source(
        "enum E{Only(u32)}enum F{Other(u32)}",
        0,
        |machine, enums| {
            let (ty, ids) = &enums[0];
            let foreign = enums[1].1[0];
            let word = TypeId::primitive(Primitive::U32);
            let value = machine.evaluator.fields(&[integer(37)], Some(ids[0]))?;
            let address = machine.allocate(&mut Vec::new(), Some(value), *ty)?;
            let root = address.location().ok_or("root")?;
            let field = Selection::Nominal(FieldId(
                machine.evaluator.facts().members.variants[ids[0].0 as usize].start,
            ));
            let wrong = Selection::Nominal(FieldId(
                machine.evaluator.facts().members.variants[foreign.0 as usize].start,
            ));
            assert!(
                machine
                    .field_projection(place(address, *ty), wrong, word)
                    .is_err()
            );
            assert!(
                machine
                    .field_projection(
                        place(address, *ty),
                        field,
                        TypeId::primitive(Primitive::Bool)
                    )
                    .is_err()
            );
            let selected = machine.field_projection(place(address, *ty), field, word)?;
            let shared =
                machine.field_projection(place(address.restrict_shared(), *ty), field, word)?;
            assert!(
                matches!(Reader::new(machine.evaluator.facts()).location(selected.location().ok_or("selected")?)?.origin,Origin::Projection{base,index:0} if base==root)
            );
            assert_eq!(
                shared.write_permission(),
                Some(WritePermission::SharedDerived)
            );
            assert!(machine.byte_root(root).is_none());
            let id = machine.convert_byte_root(root)?;
            let after = machine.field_projection(place(address, *ty), field, word)?;
            assert!(
                matches!(Reader::new(machine.evaluator.facts()).location(after.location().ok_or("after")?)?.origin, Origin::Projection { base, index: 0 } if base == root)
            );
            machine.check_raw_write(raw(selected, word))?;
            machine.write_place(raw(selected, word), integer(41))?;
            assert!(machine.read(raw(after, word))?.ok_or("field")? == integer(41));
            assert!(machine.check_raw_write(raw(shared, word)).is_err());
            assert!(
                machine
                    .read(raw(selected, word))?
                    .ok_or("unchanged writable alias")?
                    == integer(41)
            );
            assert!(machine.byte_root(root) == Some(id));
            machine.memory.expire(&[root]);
            assert!(machine.read(raw(selected, word)).is_err());
            Ok(())
        },
    )
}
#[test]
fn tagless_enum_warm_reads_recheck_initialization_and_reference_liveness() -> Result<(), String> {
    with_enum_source("enum E{Only(&'static u32)}", 0, |machine, enums| {
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
        assert!(matches!(
            machine.decode_bytes(id, 0, *ty, ReadPurpose::Value)?,
            Read::Value(_)
        ));
        machine
            .memory
            .expire(&[referent.location().ok_or("referent")?]);
        assert!(
            machine
                .decode_bytes(id, 0, *ty, ReadPurpose::Value)
                .is_err()
        );
        machine.memory.bytes.roots[id.0 as usize].initialized[0] = 0;
        assert!(
            machine
                .decode_bytes(id, 0, *ty, ReadPurpose::Value)
                .is_err()
        );
        assert!(machine.memory.bytes.enumerations.is_none());
        Ok(())
    })
}
#[test]
fn tagless_enum_repeated_field_work_is_independent_of_payload_extent_and_unrelated_types()
-> Result<(), String> {
    let mut expected = None;
    for width in [8u64, 32, 128, 4096] {
        let source = format!("enum E{{Only(u32,[u8;{width}])}}");
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
                let root = address.location().ok_or("root")?;
                let field = Selection::Nominal(FieldId(
                    machine.evaluator.facts().members.variants[ids[0].0 as usize].start,
                ));
                let selected = machine.field_projection(place(address, *ty), field, word)?;
                assert!(machine.byte_root(root).is_none());
                assert!(machine.memory.bytes.enumerations.is_none());
                let before = machine.work.steps;
                machine.convert_byte_root(root)?;
                let cold = machine.work.steps - before;
                let retained = machine.memory.retained_cells();
                let count = machine.memory.bytes.roots.len();
                let mut rows = Vec::new();
                for repeats in [8usize, 32, 128] {
                    let before = machine.work.steps;
                    for _ in 0..repeats {
                        let after = machine.field_projection(place(address, *ty), field, word)?;
                        assert!(
                            matches!(Reader::new(machine.evaluator.facts()).location(after.location().ok_or("after")?)?.origin, Origin::Projection { base, index: 0 } if base == root)
                        );
                        machine.check_raw_write(raw(after, word))?;
                        machine.write_place(raw(after, word), integer(41))?;
                        assert!(machine.read(raw(selected, word))?.ok_or("value")? == integer(41));
                    }
                    let steps = machine.work.steps - before;
                    rows.push(steps);
                    assert_eq!(machine.memory.bytes.roots.len(), count);
                    assert_eq!(machine.memory.retained_cells(), retained);
                    println!(
                        "tagless-enum width={width} noise={noise} repeats={repeats} cold={cold} warm={steps} retained={retained}"
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
