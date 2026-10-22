//! Forwarding a checked tail's representation metadata supplies no new access authority.
use super::{Failure, Machine, Reader, Resolved};
use crate::{
    const_eval::Limits,
    sema::{
        self,
        members::{FieldId, Selection},
        model::Scalar,
        values::{Constant, Evaluator, Metadata, WritePermission},
    },
    test_source,
    types::{ItemId, Kind, Primitive, TypeId},
};
fn with_machine(
    check: impl FnOnce(&mut Machine<'_, '_>, ItemId, ItemId, TypeId, TypeId) -> Result<(), Failure>,
) -> Result<(), String> {
    let text = "trait Tag{}impl Tag for u8{}struct A<T:?Sized>{head:u8,tail:T}struct B<T:?Sized>{head:u8,tail:T}const fn f(a:&A<[u8]>,b:&B<[u8]>,c:&A<dyn Tag>,d:&A<A<[u8]>>){let _=A{head:37u8,tail:[7u8;3]};let _=A{head:0u8,tail:7u8};let _=A{head:0u8,tail:[0u32;3]};let _=A{head:0u8,tail:A{head:1u8,tail:[7u8;3]}};}";
    let report = test_source::resolve(text, true)?;
    let owner = |name: &str| -> Result<ItemId, String> {
        let index = report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == name)
            .ok_or("owner")?;
        Ok(ItemId(u32::try_from(index).map_err(|_| "owner index")?))
    };
    let a = owner("A")?;
    let b = owner("B")?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let mut facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let byte = TypeId::primitive(Primitive::U8);
    let slice = facts.types.intern(Kind::Slice(byte))?;
    let fields = facts.types.intern_list(&[byte, slice])?;
    let tuple = facts.types.intern(Kind::Tuple(fields))?;
    let fields = facts.types.intern_list(&[slice, byte])?;
    let invalid = facts.types.intern(Kind::Tuple(fields))?;
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
    check(&mut machine, a, b, tuple, invalid).map_err(|e| e.message().to_owned())
}
fn nominal(
    machine: &Machine<'_, '_>,
    item: ItemId,
    child: impl Fn(Kind) -> bool,
) -> Result<TypeId, Failure> {
    let types = &machine.evaluator.facts().types;
    for n in 0..types.len() {
        let ty = TypeId(u32::try_from(n).map_err(|_| "type index")?);
        if let Some(Kind::Adt(owner, args)) = types.kind(ty)
            && owner == item
            && types.contains_parameters(ty) == Some(false)
            && let Some(crate::types::arguments::Argument::Type(argument)) =
                types.arguments(args).and_then(|mut args| args.next())
            && types.kind(argument).is_some_and(&child)
        {
            return Ok(ty);
        }
    }
    Err("missing concrete nominal test type".into())
}
fn field(machine: &Machine<'_, '_>, item: ItemId, index: u32) -> Result<Selection, Failure> {
    let members = &machine.evaluator.facts().members;
    let [variant] = members.variants(item) else {
        return Err("variant".into());
    };
    let group = members.variants.get(variant.0 as usize).ok_or("group")?;
    let id = group
        .start
        .checked_add(index)
        .filter(|n| *n < group.end)
        .ok_or("field")?;
    Ok(Selection::Nominal(FieldId(id)))
}
fn place(address: Constant, ty: TypeId) -> Resolved {
    Resolved {
        address,
        ty,
        raw: None,
        union: None,
    }
}
#[test]
fn final_tail_keeps_length_capability_and_original_pointer_after_prefix_read() -> Result<(), String>
{
    with_machine(|machine, a, _, _, _| {
        let logical = nominal(machine, a, |k| matches!(k, Kind::Slice(_)))?;
        let actual = nominal(machine, a, |k| matches!(k, Kind::Array { .. }))?;
        let tail = field(machine, a, 1)?;
        let head = field(machine, a, 0)?;
        let byte = TypeId::primitive(Primitive::U8);
        let slice = Reader::new(machine.evaluator.facts()).stored_child(logical, None, 1)?;
        let repeated = machine
            .evaluator
            .repeat(Constant::scalar(Scalar::Integer(7)), 3)?;
        let value = machine
            .evaluator
            .fields(&[Constant::scalar(Scalar::Integer(37)), repeated], None)?;
        let address = machine
            .allocate(&mut Vec::new(), Some(value), actual)?
            .unsize(3)?;
        let root = place(address, logical);
        let prefix = machine.field_projection(root, head, byte)?;
        assert_eq!(prefix.metadata(), None);
        assert_eq!(
            machine
                .read(place(prefix, byte))?
                .ok_or("prefix read")?
                .as_scalar()?,
            Scalar::Integer(37)
        );
        assert_eq!(root.address.length(), Some(3));
        let selected = machine.field_projection(root, tail, slice)?;
        assert_eq!(selected.length(), Some(3));
        assert_eq!(
            Reader::new(machine.evaluator.facts())
                .location(selected.location().ok_or("selected location")?)?
                .ty,
            Reader::new(machine.evaluator.facts()).stored_child(actual, None, 1)?
        );
        assert_eq!(selected.write_permission(), Some(WritePermission::Writable));
        let shared =
            machine.field_projection(place(address.restrict_shared(), logical), tail, slice)?;
        assert_eq!(shared.length(), Some(3));
        assert_eq!(
            shared.write_permission(),
            Some(WritePermission::SharedDerived)
        );
        assert_eq!(address.write_permission(), Some(WritePermission::Writable));
        assert_eq!(machine.sequence_length(place(shared, slice))?, 3);
        Ok(())
    })
}
#[test]
fn tail_selection_rejects_foreign_fields_wrong_types_and_missing_metadata() -> Result<(), String> {
    with_machine(|machine, a, b, _, _| {
        let owner = nominal(machine, a, |k| matches!(k, Kind::Slice(_)))?;
        let tail = field(machine, a, 1)?;
        let foreign = field(machine, b, 1)?;
        let slice = Reader::new(machine.evaluator.facts()).stored_child(owner, None, 1)?;
        let reader = Reader::new(machine.evaluator.facts());
        assert_eq!(
            reader
                .projected_metadata(owner, foreign, slice, Some(Metadata::Length(3)))
                .err(),
            Some("projected tail field has a different owner or ordinal")
        );
        assert_eq!(
            reader
                .projected_metadata(
                    owner,
                    tail,
                    TypeId::primitive(Primitive::U32),
                    Some(Metadata::Length(3))
                )
                .err(),
            Some("projected tail field differs from its checked type")
        );
        assert_eq!(
            reader.projected_metadata(owner, tail, slice, None).err(),
            Some("constant unsized tail lacks metadata")
        );
        assert_eq!(
            reader
                .projected_metadata(TypeId(u32::MAX), tail, slice, None)
                .err(),
            Some("missing projected owner sizedness")
        );
        Ok(())
    })
}
#[test]
fn tuple_tail_representation_does_not_authorize_source_tuple_unsizing() -> Result<(), String> {
    with_machine(|machine, _, _, owner, invalid| {
        let byte = TypeId::primitive(Primitive::U8);
        let slice = Reader::new(machine.evaluator.facts()).stored_child(owner, None, 1)?;
        let reader = Reader::new(machine.evaluator.facts());
        assert_eq!(
            reader.projected_metadata(
                owner,
                Selection::Tuple(0),
                byte,
                Some(Metadata::Length(3))
            )?,
            None
        );
        assert_eq!(
            reader.projected_metadata(
                owner,
                Selection::Tuple(1),
                slice,
                Some(Metadata::Length(3))
            )?,
            Some(Metadata::Length(3))
        );
        assert_eq!(
            reader
                .projected_metadata(
                    invalid,
                    Selection::Tuple(0),
                    slice,
                    Some(Metadata::Length(3))
                )
                .err(),
            Some("unsized projected field is not its owner's final field")
        );
        Ok(())
    })
}
#[test]
fn final_object_tail_preserves_complete_selected_metadata() -> Result<(), String> {
    with_machine(|machine, a, _, _, _| {
        let logical = nominal(machine, a, |k| matches!(k, Kind::Dynamic { .. }))?;
        let actual = nominal(machine, a, |k| matches!(k, Kind::Primitive(Primitive::U8)))?;
        let tail = field(machine, a, 1)?;
        let target = Reader::new(machine.evaluator.facts()).stored_child(logical, None, 1)?;
        let value = machine.evaluator.fields(
            &[
                Constant::scalar(Scalar::Integer(0)),
                Constant::scalar(Scalar::Integer(7)),
            ],
            None,
        )?;
        let metadata = Metadata::Object {
            concrete: TypeId::primitive(Primitive::U8),
            target,
        };
        let root = machine
            .allocate(&mut Vec::new(), Some(value), actual)?
            .with_metadata(metadata)?
            .restrict_shared();
        let selected = machine.field_projection(place(root, logical), tail, target)?;
        assert_eq!(selected.metadata(), Some(metadata));
        assert_eq!(selected.write_permission(), root.write_permission());
        Ok(())
    })
}

#[test]
fn physical_tail_selection_rejects_wrong_owner_element_length_and_object() -> Result<(), String> {
    with_machine(|machine, a, b, _, _| {
        let logical = nominal(machine, a, |k| matches!(k, Kind::Slice(_)))?;
        let actual = nominal(
            machine,
            a,
            |k| matches!(k, Kind::Array { element, .. } if element == TypeId::primitive(Primitive::U8)),
        )?;
        let wrong = nominal(
            machine,
            a,
            |k| matches!(k, Kind::Array { element, .. } if element == TypeId::primitive(Primitive::U32)),
        )?;
        let foreign = nominal(machine, b, |k| matches!(k, Kind::Slice(_)))?;
        let object = nominal(machine, a, |k| matches!(k, Kind::Dynamic { .. }))?;
        let reader = Reader::new(machine.evaluator.facts());
        let target = reader.stored_child(logical, None, 1)?;
        assert!(
            reader
                .tail_storage(logical, foreign, 1, target, Metadata::Length(3))
                .is_err()
        );
        assert!(
            reader
                .tail_storage(logical, wrong, 1, target, Metadata::Length(3))
                .is_err()
        );
        assert!(
            reader
                .tail_storage(logical, actual, 1, target, Metadata::Length(4))
                .is_err()
        );
        let concrete = nominal(machine, a, |k| matches!(k, Kind::Primitive(Primitive::U8)))?;
        let target = reader.stored_child(object, None, 1)?;
        assert!(
            reader
                .tail_storage(
                    object,
                    concrete,
                    1,
                    target,
                    Metadata::Object {
                        concrete: TypeId::primitive(Primitive::U32),
                        target
                    }
                )
                .is_err()
        );
        Ok(())
    })
}
#[test]
fn nested_tail_locations_keep_each_immediate_physical_parent() -> Result<(), String> {
    with_machine(|machine, a, _, _, _| {
        let nested = |slice: bool| {
            nominal(machine, a, |kind| {
                let Kind::Adt(_, args) = kind else {
                    return false;
                };
                let types = &machine.evaluator.facts().types;
                let Some(crate::types::arguments::Argument::Type(child)) =
                    types.arguments(args).and_then(|mut args| args.next())
                else {
                    return false;
                };
                if slice {
                    matches!(types.kind(child), Some(Kind::Slice(_)))
                } else {
                    matches!(types.kind(child), Some(Kind::Array { .. }))
                }
            })
        };
        let logical = nested(true)?;
        let actual = nested(false)?;
        let repeated = machine
            .evaluator
            .repeat(Constant::scalar(Scalar::Integer(7)), 3)?;
        let inner = machine
            .evaluator
            .fields(&[Constant::scalar(Scalar::Integer(1)), repeated], None)?;
        let value = machine
            .evaluator
            .fields(&[Constant::scalar(Scalar::Integer(0)), inner], None)?;
        let address = machine
            .allocate(&mut Vec::new(), Some(value), actual)?
            .unsize(3)?;
        let tail = field(machine, a, 1)?;
        let reader = Reader::new(machine.evaluator.facts());
        let logical_inner = reader.stored_child(logical, None, 1)?;
        let actual_inner = reader.stored_child(actual, None, 1)?;
        let inner = machine.field_projection(place(address, logical), tail, logical_inner)?;
        assert_eq!(
            Reader::new(machine.evaluator.facts())
                .location(inner.location().ok_or("inner location")?)?
                .ty,
            actual_inner
        );
        let reader = Reader::new(machine.evaluator.facts());
        let logical_leaf = reader.stored_child(logical_inner, None, 1)?;
        let actual_leaf = reader.stored_child(actual_inner, None, 1)?;
        let leaf = machine.field_projection(place(inner, logical_inner), tail, logical_leaf)?;
        assert_eq!(
            Reader::new(machine.evaluator.facts())
                .location(leaf.location().ok_or("leaf location")?)?
                .ty,
            actual_leaf
        );
        assert_eq!(leaf.length(), Some(3));
        assert_eq!(machine.sequence_length(place(leaf, logical_leaf))?, 3);
        Ok(())
    })
}
