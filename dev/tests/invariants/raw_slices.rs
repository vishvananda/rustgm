//! Slice metadata, address arithmetic and actual access retain different obligations.
use super::{Failure, Machine, RawAccess, Resolved};
use crate::{
    const_eval::{Limits, memory::RawAlignment},
    sema::{
        self,
        model::Scalar,
        values::{Constant, Evaluator, WritePermission},
    },
    test_source,
    types::{Kind, Primitive, TypeId},
};
fn with_array(
    element: TypeId,
    check: impl FnOnce(&mut Machine<'_, '_>, TypeId, TypeId) -> Result<(), Failure>,
) -> Result<(), String> {
    let report = test_source::resolve("", true)?;
    let mut facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let array = facts.types.intern(Kind::Array { element, length: 3 })?;
    let slice = facts.types.intern(Kind::Slice(element))?;
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
    check(&mut machine, array, slice).map_err(|e| e.message().to_owned())
}
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
fn same_path(machine: &mut Machine<'_, '_>, a: Constant, b: Constant) -> Result<(), Failure> {
    let a = machine.path(a)?;
    let b = machine.path(b)?;
    assert_eq!(a.root, b.root);
    assert_eq!(a.fields, b.fields);
    assert_eq!(
        a.range.map(|r| (r.from, r.length)),
        b.range.map(|r| (r.from, r.length))
    );
    Ok(())
}
#[test]
fn raw_slice_selection_keeps_logical_physical_bounds_and_pointer_permissions() -> Result<(), String>
{
    let element = TypeId::primitive(Primitive::U32);
    with_array(element, |machine, array, slice| {
        let value = machine
            .evaluator
            .repeat(Constant::scalar(Scalar::Integer(7)), 3)?;
        let address = machine.allocate(&mut Vec::new(), Some(value), array)?;
        let place = raw(address.unsize(5)?, slice);
        let selected = machine.raw_slice_element(place, 1, element)?;
        assert_eq!(
            machine.read(selected)?.ok_or("read")?.as_scalar()?,
            Scalar::Integer(7)
        );
        assert_eq!(
            machine
                .raw_slice_element(place, 3, element)
                .err()
                .map(Failure::message),
            Some("constant raw slice element is outside backing bounds")
        );
        assert_eq!(
            machine
                .raw_slice_element(place, 5, element)
                .err()
                .map(Failure::message),
            Some("constant index is out of bounds")
        );
        let shared =
            machine.raw_slice_element(raw(place.address.restrict_shared(), slice), 1, element)?;
        same_path(machine, shared.address, selected.address)?;
        assert_eq!(
            shared.address.write_permission(),
            Some(WritePermission::SharedDerived)
        );
        assert_eq!(
            machine.check_raw_write(shared).err().map(Failure::message),
            Some("constant raw pointer write is derived from a shared reference")
        );
        assert_eq!(
            selected.address.write_permission(),
            Some(WritePermission::Writable)
        );
        machine.check_raw_write(selected)?;
        let first = machine.evaluator.projection(address, 0, element)?;
        let offset = machine.raw_slice_element(raw(first.unsize(3)?, slice), 2, element)?;
        let expected = machine.evaluator.projection(address, 2, element)?;
        same_path(machine, offset.address, expected)?;
        Ok(())
    })
}
#[test]
fn raw_slice_metadata_and_zero_offsets_do_not_read_or_revive_expired_storage() -> Result<(), String>
{
    let element = TypeId::primitive(Primitive::U32);
    with_array(element, |machine, array, slice| {
        let mut roots = Vec::new();
        let address = machine.allocate(&mut roots, None, array)?;
        let place = raw(address.unsize(3)?, slice);
        assert_eq!(machine.raw_slice_metadata(place)?.1, 3);
        let selected = machine.raw_slice_element(place, 0, element)?;
        assert!(machine.read(selected).is_err());
        machine.memory.expire(&roots);
        assert_eq!(machine.raw_slice_metadata(place)?.1, 3);
        let selected = machine.raw_slice_element(place, 0, element)?;
        for error in [
            machine.read(selected).err(),
            machine.check_raw_write(selected).err(),
            machine.shared_raw_reference(selected).err(),
        ] {
            assert_eq!(
                error.map(Failure::message),
                Some("constant pointer refers to expired storage")
            );
        }
        assert_eq!(
            machine
                .raw_slice_element(place, 1, element)
                .err()
                .map(Failure::message),
            Some("constant pointer refers to expired storage")
        );
        Ok(())
    })?;
    with_array(TypeId::UNIT, |machine, array, slice| {
        let mut roots = Vec::new();
        let address = machine.allocate(&mut roots, None, array)?;
        machine.memory.expire(&roots);
        let selected =
            machine.raw_slice_element(raw(address.unsize(3)?, slice), 1, TypeId::UNIT)?;
        assert_eq!(
            machine.read(selected).err().map(Failure::message),
            Some("constant pointer refers to expired storage")
        );
        Ok(())
    })
}
#[test]
fn raw_slice_anchor_type_and_budget_failures_cannot_publish_a_projection() -> Result<(), String> {
    let element = TypeId::primitive(Primitive::U32);
    with_array(element, |machine, array, slice| {
        let address = machine.allocate(&mut Vec::new(), None, array)?;
        let place = raw(address.unsize(3)?, slice);
        let other = machine.allocate(&mut Vec::new(), None, array)?;
        let wrong = Resolved {
            raw: Some(RawAccess {
                location: other.location(),
                ..place.raw.ok_or("anchor")?
            }),
            ..place
        };
        assert_eq!(
            machine
                .raw_slice_metadata(wrong)
                .err()
                .map(Failure::message),
            Some("constant raw slice differs from its checked access anchor")
        );
        assert_eq!(
            machine
                .raw_slice_element(place, 0, TypeId::primitive(Primitive::U8))
                .err()
                .map(Failure::message),
            Some("constant raw slice element differs from its checked type")
        );
        let slots = machine.slots;
        let limit = machine.limits.slots;
        machine.limits.slots = machine.memory.retained_cells() + slots;
        assert_eq!(
            machine
                .raw_slice_element(place, 0, element)
                .err()
                .map(Failure::message),
            Some("constant execution storage limit exceeded")
        );
        assert_eq!(machine.slots, slots);
        machine.limits.slots = limit;
        machine.raw_slice_element(place, 0, element)?;
        assert_eq!(machine.slots, slots);
        Ok(())
    })
}
