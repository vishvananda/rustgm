//! Str references use byte validity and retain current masks, provenance and authority.
use super::*;
use crate::const_eval::memory::{RawAccess, RawAlignment, Read};
fn raw(address: Constant, ty: TypeId) -> Resolved {
    Resolved {
        address,
        ty,
        union: None,
        raw: Some(RawAccess {
            location: address.location(),
            ty,
            alignment: RawAlignment::Required,
        }),
    }
}
#[test]
fn str_reference_rechecks_current_masks_without_unicode_policy() -> Result<(), String> {
    with_array(2, 1 << 20, |machine, array| {
        let value = machine
            .evaluator
            .fields(&[scalar(255), scalar(195)], None)?;
        let address = machine.allocate(&mut Vec::new(), Some(value), array)?;
        let root = address.location().ok_or("array root")?;
        let ty = TypeId::primitive(Primitive::Str);
        let selected = machine.evaluator.subsequence(address, 0, 2, ty)?;
        assert!(matches!(
            machine.validate_sequence_word_reference(selected, ty, false)?,
            Read::Value(_)
        ));
        machine.convert_byte_root(root)?;
        let snapshot = Snapshot {
            bytes: vec![255, 195],
            initialized: vec![1],
            fragments: None,
        };
        machine.commit_byte_snapshot(root, 0, &snapshot)?;
        assert!(
            machine
                .validate_sequence_word_reference(selected, ty, false)
                .is_err()
        );
        let snapshot = Snapshot {
            initialized: vec![3],
            ..snapshot
        };
        machine.commit_byte_snapshot(root, 0, &snapshot)?;
        assert!(matches!(
            machine.validate_sequence_word_reference(selected, ty, false)?,
            Read::Value(_)
        ));
        assert!(
            machine
                .validate_sequence_word_reference(selected.unsize(3)?, ty, false)
                .is_err()
        );
        assert_eq!(machine.slots, 0);
        Ok(())
    })
}
#[test]
fn str_empty_bounds_and_mutability_do_not_recover_authority_or_liveness() -> Result<(), String> {
    with_array(0, 1 << 20, |machine, array| {
        let value = machine.evaluator.fields(&[], None)?;
        let address = machine.allocate(&mut Vec::new(), Some(value), array)?;
        let ty = TypeId::primitive(Primitive::Str);
        let pointer = address.unsize(0)?;
        assert!(machine.shared_raw_reference(raw(pointer, ty))?.is_some());
        assert!(
            machine
                .mutable_raw_reference(raw(pointer.restrict_shared(), ty))
                .is_err()
        );
        assert!(machine.mutable_raw_reference(raw(pointer, ty))?.is_some());
        assert!(
            machine
                .shared_raw_reference(raw(address.unsize(1)?, ty))
                .is_err()
        );
        machine
            .memory
            .expire(&[address.location().ok_or("array root")?]);
        assert!(machine.shared_raw_reference(raw(pointer, ty)).is_err());
        Ok(())
    })
}
#[test]
fn str_one_past_empty_reference_uses_original_range_and_pointer_restriction() -> Result<(), String>
{
    with_array(2, 1 << 20, |machine, array| {
        let value = machine.evaluator.fields(&[scalar(65), scalar(66)], None)?;
        let address = machine.allocate(&mut Vec::new(), Some(value), array)?;
        let ty = TypeId::primitive(Primitive::Str);
        let (end, work) = machine.evaluator.byte_projection(
            address.restrict_shared(),
            2,
            TypeId::primitive(Primitive::U8),
            10000,
        )?;
        machine.charge(work)?;
        let reference = machine
            .shared_raw_reference(raw(end.unsize(0)?, ty))?
            .ok_or("empty reference")?;
        let selected = machine.path(reference)?;
        let range = selected.range()?.ok_or("range")?;
        assert_eq!((range.from, range.length), (2, 0));
        assert_eq!(
            reference.write_permission(),
            address.restrict_shared().write_permission()
        );
        assert!(
            machine
                .shared_raw_reference(raw(end.unsize(1)?, ty))
                .is_err()
        );
        assert!(
            machine
                .mutable_raw_reference(raw(end.unsize(0)?, ty))
                .is_err()
        );
        Ok(())
    })
}
#[test]
fn str_storage_requires_exact_byte_elements_and_stable_ordinary_layouts() -> Result<(), String> {
    assert_eq!(size_of::<Constant>(), 32);
    assert_eq!(size_of::<crate::sema::values::Location>(), 64);
    with_storage(
        1 << 20,
        |types| {
            Ok(types.intern(Kind::Array {
                element: TypeId::primitive(Primitive::U16),
                length: 2,
            })?)
        },
        |machine, array| {
            let value = machine.evaluator.fields(&[scalar(1), scalar(2)], None)?;
            let address = machine.allocate(&mut Vec::new(), Some(value), array)?;
            let ty = TypeId::primitive(Primitive::Str);
            assert!(machine.evaluator.subsequence(address, 0, 2, ty).is_err());
            assert!(
                machine
                    .shared_raw_reference(raw(address.unsize(2)?, ty))
                    .is_err()
            );
            assert!(
                Reader::new(machine.evaluator.facts())
                    .sequence_element(array)
                    .is_err()
            );
            assert_eq!(
                Reader::new(machine.evaluator.facts()).sequence_element(ty)?,
                TypeId::primitive(Primitive::U8)
            );
            Ok(())
        },
    )
}
#[test]
fn str_range_thin_access_keeps_current_masks_offset_and_write_authority() -> Result<(), String> {
    with_array(3, 1 << 20, |machine, array| {
        let value = machine
            .evaluator
            .fields(&[scalar(1), scalar(2), scalar(3)], None)?;
        let address = machine.allocate(&mut Vec::new(), Some(value), array)?;
        let root = address.location().ok_or("root")?;
        let ty = TypeId::primitive(Primitive::Str);
        let selected = machine.evaluator.subsequence(address, 1, 1, ty)?;
        let (crate::sema::values::WordTarget::Value(thin), _) =
            machine.evaluator.pointer_words(selected)?
        else {
            return Err("data pointer".into());
        };
        let byte = TypeId::primitive(Primitive::U8);
        let place = raw(thin, byte);
        assert!(machine.read(place)?.ok_or("read")? == scalar(2));
        machine.write_place(place, scalar(9))?;
        assert!(machine.read(place)?.ok_or("read")? == scalar(9));
        assert!(
            machine
                .write_place(raw(thin.restrict_shared(), byte), scalar(7))
                .is_err()
        );
        machine.commit_byte_snapshot(
            root,
            0,
            &Snapshot {
                bytes: vec![1, 9, 3],
                initialized: vec![5],
                fragments: None,
            },
        )?;
        assert!(machine.read(place).is_err());
        Ok(())
    })
}
#[test]
fn str_range_layout_uses_exact_array_owner_element_and_bounds() -> Result<(), String> {
    with_array(4, 1 << 20, |machine, array| {
        machine.footprint(array)?;
        let reader = Reader::new(machine.evaluator.facts());
        let layout = *machine
            .evaluator
            .facts()
            .layouts
            .get(array)
            .ok_or("layout")?;
        let text = TypeId::primitive(Primitive::Str);
        assert_eq!(
            reader.sequence_layout_displacement(array, layout, 1, 2, text)?,
            1
        );
        assert!(
            reader
                .sequence_layout_displacement(text, layout, 1, 2, text)
                .is_err()
        );
        assert!(
            reader
                .sequence_layout_displacement(array, layout, 3, 2, text)
                .is_err()
        );
        let mut wrong = layout;
        wrong.shape = crate::abi::layout::Shape::Array {
            element: TypeId::primitive(Primitive::U16),
            stride: 2,
            length: 4,
        };
        assert!(
            reader
                .sequence_layout_displacement(array, wrong, 1, 2, text)
                .is_err()
        );
        wrong.shape = crate::abi::layout::Shape::Array {
            element: TypeId::primitive(Primitive::U8),
            stride: 1,
            length: 5,
        };
        assert!(
            reader
                .sequence_layout_displacement(array, wrong, 1, 2, text)
                .is_err()
        );
        Ok(())
    })
}
#[test]
fn str_selected_byte_work_is_independent_of_backing_width() -> Result<(), String> {
    let mut per_read = None;
    for width in [8, 32, 128] {
        for repeats in [8, 32, 128] {
            with_array(width, 1 << 20, |machine, array| {
                let value = machine.evaluator.repeat(scalar(65), width)?;
                let address = machine.allocate(&mut Vec::new(), Some(value), array)?;
                let text = machine.evaluator.subsequence(
                    address,
                    width - 1,
                    1,
                    TypeId::primitive(Primitive::Str),
                )?;
                let (crate::sema::values::WordTarget::Value(thin), _) =
                    machine.evaluator.pointer_words(text)?
                else {
                    return Err("data pointer".into());
                };
                let place = raw(thin, TypeId::primitive(Primitive::U8));
                assert!(machine.read(place)?.ok_or("warm")? == scalar(65));
                let before = machine.work.steps;
                let retained = machine.memory.retained_cells();
                for _ in 0..repeats {
                    assert!(machine.read(place)?.ok_or("read")? == scalar(65));
                }
                let steps = machine.work.steps - before;
                let unit = *per_read.get_or_insert(steps / repeats);
                assert_eq!(steps, unit * repeats);
                assert_eq!(machine.memory.retained_cells(), retained);
                eprintln!(
                    "str-byte width={width} repeats={repeats} steps={steps} retained={retained}"
                );
                Ok(())
            })?;
        }
    }
    Ok(())
}
