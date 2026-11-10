//! Bare array-to-slice results retain bounds, validity and the original provenance.
use super::*;
use crate::const_eval::memory::Read;

fn with_slice(
    length: u64,
    check: impl FnOnce(&mut Machine<'_, '_>, TypeId, TypeId) -> Result<(), Failure>,
) -> Result<(), String> {
    let slice = std::cell::Cell::new(TypeId::primitive(Primitive::U8));
    with_storage(
        1 << 20,
        |types| {
            slice.set(types.intern(Kind::Slice(TypeId::primitive(Primitive::U8)))?);
            Ok(types.intern(Kind::Array {
                element: TypeId::primitive(Primitive::U8),
                length,
            })?)
        },
        |machine, array| check(machine, array, slice.get()),
    )
}

#[test]
fn named_slice_results_match_independent_source_outcomes() -> Result<(), String> {
    for name in ["byte", "array", "empty", "str", "ordinary"] {
        let path = format!("pa31/tests/spec/whole-slice-validity/{name}.rs");
        let source = test_source::fixture(&path).map_err(|error| error.to_string())?;
        let report = test_source::source_provider(&source, "whole_slices")?;
        crate::pipeline::analyze_source(&report).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[test]
fn whole_array_views_preserve_metadata_and_backing_identity() -> Result<(), String> {
    for length in [0, 2, 8] {
        with_slice(length, |machine, array, slice| {
            let elements = vec![scalar(49); usize::try_from(length).map_err(|_| "length")?];
            let value = machine.evaluator.fields(&elements, None)?;
            let address = machine.allocate(&mut Vec::new(), Some(value), array)?;
            let pointer = address.unsize(length)?;
            for byte_backing in [false, true] {
                if byte_backing {
                    machine.convert_byte_root(address.location().ok_or("array root")?)?;
                }
                let Read::Value(checked) =
                    machine.validate_sequence_word_reference(pointer, slice, false)?
                else {
                    return Err("valid array view remained pending".into());
                };
                assert_eq!(checked.location(), pointer.location());
                assert_eq!(checked.length(), Some(length));
                assert!(
                    machine
                        .validate_sequence_word_reference(pointer.unsize(length + 1)?, slice, false)
                        .is_err()
                );
                assert!(
                    machine
                        .validate_sequence_word_reference(checked, slice, true)
                        .is_err()
                );
            }
            machine
                .memory
                .expire(&[address.location().ok_or("array root")?]);
            assert!(
                machine
                    .validate_sequence_word_reference(pointer, slice, false)
                    .is_err()
            );
            assert_eq!(machine.slots, 0);
            Ok(())
        })?;
    }
    Ok(())
}

#[test]
fn whole_array_validation_rechecks_uninitialized_bytes_and_element_type() -> Result<(), String> {
    with_slice(2, |machine, array, slice| {
        let empty = machine.allocate(&mut Vec::new(), None, array)?;
        assert!(
            machine
                .validate_sequence_word_reference(empty.unsize(2)?, slice, false)
                .is_err()
        );
        let value = machine.evaluator.fields(&[scalar(48), scalar(49)], None)?;
        let address = machine.allocate(&mut Vec::new(), Some(value), array)?;
        let pointer = address.unsize(2)?;
        let root = address.location().ok_or("array root")?;
        machine.convert_byte_root(root)?;
        assert!(matches!(
            machine.validate_sequence_word_reference(pointer, slice, false)?,
            Read::Value(_)
        ));
        machine.commit_byte_snapshot(
            root,
            0,
            &Snapshot {
                bytes: vec![48, 49],
                initialized: vec![1],
                fragments: None,
            },
        )?;
        assert!(
            machine
                .validate_sequence_word_reference(pointer, slice, false)
                .is_err()
        );
        Ok(())
    })?;
    with_storage(
        1 << 20,
        |types| {
            Ok(types.intern(Kind::Array {
                element: TypeId::primitive(Primitive::U16),
                length: 2,
            })?)
        },
        |machine, array| {
            let value = machine.evaluator.fields(&[scalar(48), scalar(49)], None)?;
            let address = machine.allocate(&mut Vec::new(), Some(value), array)?;
            assert!(
                machine
                    .validate_sequence_word_reference(
                        address.unsize(2)?,
                        TypeId::primitive(Primitive::Str),
                        false
                    )
                    .is_err()
            );
            Ok(())
        },
    )
}
