//! Logical byte types never replace original allocation or pointer permissions.
use super::{BytePlace, Constant, Failure, Machine, RawAlignment, Reader, Resolved};
use crate::{
    const_eval::{
        Limits,
        memory::{RawAccess, StoragePath, Terminal},
    },
    sema::{
        self,
        model::Scalar,
        values::{Evaluator, Origin, WritePermission},
    },
    test_source,
    types::{Kind, Primitive, TypeId},
};
fn scalar(value: u128) -> Constant {
    Constant::scalar(Scalar::Integer(value))
}
fn with_machine(
    check: impl FnOnce(&mut Machine<'_, '_>) -> Result<(), Failure>,
) -> Result<(), String> {
    let report = test_source::resolve("", true)?;
    let mut facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    for element in [Primitive::U8, Primitive::U32] {
        for length in [8, 32, 128] {
            facts.types.intern(Kind::Array {
                element: TypeId::primitive(element),
                length,
            })?;
        }
    }
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
    check(&mut machine).map_err(|e| e.message().to_owned())
}
fn array(machine: &Machine<'_, '_>, element: TypeId, length: u64) -> Result<TypeId, Failure> {
    (0..machine.evaluator.facts().types.len()).filter_map(|n|u32::try_from(n).ok()).map(TypeId)
        .find(|&ty| matches!(machine.evaluator.facts().types.kind(ty),Some(Kind::Array{element:e,length:n}) if e==element && n==length)).ok_or("test array type".into())
}
fn raw(address: Constant, ty: TypeId, anchor: TypeId) -> Resolved {
    Resolved {
        address,
        ty,
        raw: Some(RawAccess {
            alignment: RawAlignment::Required,
            location: address.location(),
            ty: anchor,
        }),
        union: None,
    }
}
#[test]
fn byte_projection_layout_preserves_ordinary_storage_density() {
    assert_eq!(size_of::<StoragePath>(), 56);
    assert_eq!(size_of::<Option<Terminal>>(), 24);
    assert_eq!(size_of::<crate::sema::values::Constant>(), 32);
    assert_eq!(size_of::<crate::sema::values::Location>(), 64);
    assert_eq!(size_of::<Origin>(), 24);
}
#[test]
fn normalized_byte_views_keep_each_pointer_restriction_and_current_bytes() -> Result<(), String> {
    with_machine(|machine| {
        let word = TypeId::primitive(Primitive::U32);
        let half = TypeId::primitive(Primitive::U16);
        let address = machine.allocate(&mut Vec::new(), Some(scalar(0x0007_0001)), word)?;
        let base = raw(address, half, half);
        let selected = machine.byte_address(base, 2, half)?;
        let shared = machine.byte_address(raw(address.restrict_shared(), half, half), 2, half)?;
        assert_eq!(selected.write_permission(), Some(WritePermission::Writable));
        assert_eq!(
            shared.write_permission(),
            Some(WritePermission::SharedDerived)
        );
        let zero = machine.byte_address(raw(selected, half, half), 0, half)?;
        let location =
            Reader::new(machine.evaluator.facts()).location(zero.location().ok_or("view")?)?;
        assert!(
            matches!(location.origin,Origin::ByteOffset{base,offset:2} if Some(base)==address.location())
        );
        assert!(machine.read(raw(selected, half, half))?.ok_or("read")? == scalar(7));
        assert!(
            machine
                .write_place(raw(shared, half, half), scalar(9))
                .is_err()
        );
        machine.write_place(raw(selected, half, half), scalar(11))?;
        assert!(
            machine
                .read(raw(address, word, word))?
                .ok_or("whole read")?
                == scalar(0x000b_0001)
        );
        assert_eq!(address.write_permission(), Some(WritePermission::Writable));
        let past = machine.byte_address(base, 4, half)?;
        assert!(machine.read(raw(past, half, half)).is_err());
        assert!(machine.byte_address(base, 5, half).is_err());
        machine.memory.expire(&[address.location().ok_or("root")?]);
        assert!(machine.read(raw(selected, half, half)).is_err());
        Ok(())
    })
}
#[test]
fn byte_anchor_alignment_is_separate_from_leaf_reference_and_foreign_anchor() -> Result<(), String>
{
    with_machine(|machine| {
        let byte = TypeId::primitive(Primitive::U8);
        let word = TypeId::primitive(Primitive::U32);
        let array = array(machine, byte, 8)?;
        let value = machine.evaluator.repeat(scalar(0), 8)?;
        let address = machine.allocate(&mut Vec::new(), Some(value), array)?;
        let selected = machine.byte_address(raw(address, array, array), 1, word)?;
        let place = Resolved {
            address: selected,
            ty: word,
            raw: Some(RawAccess {
                alignment: RawAlignment::Required,
                location: address.location(),
                ty: array,
            }),
            union: None,
        };
        let leaf = BytePlace {
            root: address.location().ok_or("root")?,
            offset: 1,
            ty: word,
        };
        machine.check_byte_access(place, leaf, false)?;
        assert!(machine.check_byte_access(place, leaf, true).is_err());
        assert!(
            machine
                .check_byte_access(raw(selected, word, word), leaf, false)
                .is_err()
        );
        let foreign = machine.allocate(&mut Vec::new(), Some(scalar(0)), word)?;
        let wrong = Resolved {
            raw: Some(RawAccess {
                alignment: RawAlignment::Unaligned,
                location: foreign.location(),
                ty: word,
            }),
            ..place
        };
        assert!(machine.check_byte_access(wrong, leaf, false).is_err());
        assert!(machine.check_byte_access(wrong, leaf, true).is_err());
        Ok(())
    })
}
#[test]
fn repeated_selected_byte_reads_and_writes_have_width_independent_work() -> Result<(), String> {
    let mut deltas = None;
    for width in [8u64, 32, 128] {
        let mut rows = Vec::new();
        for count in [8usize, 32, 128] {
            with_machine(|machine| {
                let byte = TypeId::primitive(Primitive::U8);
                let word = TypeId::primitive(Primitive::U32);
                let array = array(machine, word, width)?;
                let value = machine.evaluator.repeat(scalar(7), width)?;
                let address = machine.allocate(&mut Vec::new(), Some(value), array)?;
                let selected =
                    machine.byte_address(raw(address, array, array), (width - 1) * 4, byte)?;
                let place = raw(selected, byte, byte);
                machine.read(place)?.ok_or("warm read")?;
                let retained = machine.memory.retained_cells();
                let before = machine.work.steps;
                for _ in 0..count {
                    machine.write_place(place, scalar(9))?;
                    assert!(machine.read(place)?.ok_or("read")? == scalar(9));
                }
                let steps = machine.work.steps - before;
                assert_eq!(machine.memory.retained_cells(), retained);
                assert!(steps <= count * 1024);
                println!(
                    "byte-access width={width} count={count} steps={steps} retained={retained}"
                );
                rows.push(steps);
                Ok(())
            })?;
        }
        let delta = (rows[1] - rows[0], rows[2] - rows[1]);
        assert_eq!(delta.1, delta.0 * 4);
        if let Some(prior) = deltas {
            assert_eq!(delta, prior);
        } else {
            deltas = Some(delta);
        }
    }
    Ok(())
}

#[test]
fn validated_normalized_reference_retains_the_complete_pointer_identity() -> Result<(), String> {
    with_machine(|machine| {
        let word = TypeId::primitive(Primitive::U32);
        let signed = TypeId::primitive(Primitive::I32);
        let address = machine.allocate(&mut Vec::new(), Some(scalar(7)), word)?;
        let selected = machine.byte_address(raw(address, signed, signed), 0, signed)?;
        let shared = selected.restrict_shared();
        assert!(
            machine
                .shared_raw_reference(raw(shared, signed, signed))?
                .ok_or("shared view")?
                == shared
        );
        assert!(
            machine
                .mutable_raw_reference(raw(selected, signed, signed))?
                .ok_or("mutable view")?
                == selected
        );
        assert!(
            machine
                .mutable_raw_reference(raw(shared, signed, signed))
                .is_err()
        );
        Ok(())
    })
}
