//! Stored values refine conservative owned type facts without changing permissions.
use super::{Answer, Constant, Reader};
use crate::{
    sema::{
        constants::Value,
        members::VariantId,
        model::{Declarations, Scalar},
    },
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId,
        aggregate::{Kind as Aggregate, Repr, SharedMutation, Variant},
    },
};

struct EmptySources;
impl crate::resolve::load::Sources for EmptySources {
    fn read(
        &mut self,
        _: &std::path::Path,
    ) -> Result<Option<crate::resolve::load::InputFile>, String> {
        Ok(None)
    }
}
fn facts() -> Result<Declarations, String> {
    let input = crate::resolve::load::InputFile {
        identity: "storage.rs".into(),
        path: "storage.rs".into(),
        source: crate::support::source::SourceFile::new(Vec::new())
            .map_err(|e| format!("{e:?}"))?,
    };
    let expanded = crate::resolve::expansion::analyze(
        input,
        &mut EmptySources,
        &crate::expand::config::Config::default(),
    )
    .map_err(|e| format!("{e:?}"))?;
    let report = crate::resolve::analyze_expanded(expanded).map_err(|e| format!("{e:?}"))?;
    Declarations::new(&report).map_err(|e| format!("{e:?}"))
}
fn atomic(facts: &mut Declarations) -> Result<(TypeId, Constant), String> {
    let fields = facts
        .types
        .intern_list(&[TypeId::primitive(Primitive::U32)])?;
    facts.aggregates.insert_with_mutation(
        ItemId(0),
        Aggregate::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
        SharedMutation::Interior,
    )?;
    let ty = facts
        .types
        .intern(Kind::Adt(ItemId(0), GenericArgs::EMPTY))?;
    let value = product(facts, &[Constant::scalar(Scalar::Integer(0))], None)?;
    Ok((ty, value))
}
fn product(
    facts: &mut Declarations,
    fields: &[Constant],
    variant: Option<VariantId>,
) -> Result<Constant, String> {
    facts
        .constant_products
        .sequence(fields.iter().map(|v| v.0), variant)
        .map(Constant)
        .map_err(String::from)
}

#[test]
fn actual_empty_arrays_refine_conservative_types() -> Result<(), String> {
    let mut facts = facts()?;
    let (atomic, value) = atomic(&mut facts)?;
    let empty = facts.types.intern(Kind::Array {
        element: atomic,
        length: 0,
    })?;
    let one = facts.types.intern(Kind::Array {
        element: atomic,
        length: 1,
    })?;
    let empty_value = product(&mut facts, &[], None)?;
    assert_eq!(
        Reader::new(&facts).shared_mutation(empty, empty_value)?,
        Answer::Ordinary
    );
    let repeated = Constant(facts.constant_products.repeat(value.0, 0)?);
    assert_eq!(
        Reader::new(&facts).shared_mutation(empty, repeated)?,
        Answer::Ordinary
    );
    let repeated = Constant(facts.constant_products.repeat(value.0, 1)?);
    assert_eq!(
        Reader::new(&facts).shared_mutation(one, repeated)?,
        Answer::Interior
    );
    assert!(
        Reader::new(&facts)
            .shared_mutation(empty, repeated)
            .is_err()
    );
    Ok(())
}

#[test]
fn only_active_variant_fields_contribute() -> Result<(), String> {
    use crate::{
        resolve::unit::{FileId, NodeKey},
        syntax::{ast::NodeId, item::Shape},
    };
    let mut facts = facts()?;
    let (atomic, value) = atomic(&mut facts)?;
    let item = ItemId(1);
    let empty = facts.types.intern_list(&[])?;
    let fields = facts.types.intern_list(&[atomic])?;
    facts.aggregates.insert(
        item,
        Aggregate::Enum,
        Repr::default(),
        &[
            Variant {
                fields: empty,
                discriminant: 0,
            },
            Variant {
                fields,
                discriminant: 1,
            },
        ],
    )?;
    let origin = NodeKey {
        file: FileId(0),
        node: NodeId(0),
    };
    let none = facts
        .members
        .insert_metadata(item, 0, origin, Shape::Unit, &[])?;
    let some = facts
        .members
        .insert_metadata(item, 1, origin, Shape::Tuple, &[])?;
    let ty = facts.types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    facts.aggregates.mark_discriminants_pending(item)?;
    let none_value = Constant(Value::Variant(none));
    let some_value = product(&mut facts, &[value], Some(some))?;
    assert_eq!(
        Reader::new(&facts).shared_mutation(ty, none_value)?,
        Answer::Ordinary
    );
    assert_eq!(
        Reader::new(&facts).shared_mutation(ty, some_value)?,
        Answer::Interior
    );
    let other = facts
        .members
        .insert_metadata(ItemId(2), 0, origin, Shape::Unit, &[])?;
    assert!(
        Reader::new(&facts)
            .shared_mutation(ty, Constant(Value::Variant(other)))
            .is_err()
    );
    let fields = facts.types.intern_list(&[ty, atomic])?;
    let tuple = facts.types.intern(Kind::Tuple(fields))?;
    let stored = product(&mut facts, &[none_value, value], None)?;
    assert_eq!(
        Reader::new(&facts).shared_mutation(tuple, stored)?,
        Answer::Interior
    );
    Ok(())
}

#[test]
fn shared_value_graphs_and_repeats_do_not_expand_exponentially() -> Result<(), String> {
    let mut facts = facts()?;
    let (atomic, _) = atomic(&mut facts)?;
    let mut ty = facts.types.intern(Kind::Array {
        element: atomic,
        length: 0,
    })?;
    let mut value = product(&mut facts, &[], None)?;
    for _ in 0..256 {
        let fields = facts.types.intern_list(&[ty, ty])?;
        ty = facts.types.intern(Kind::Tuple(fields))?;
        value = product(&mut facts, &[value, value], None)?;
    }
    let (answer, steps) = Reader::new(&facts).shared_mutation_work(ty, value)?;
    assert_eq!(answer, Answer::Ordinary);
    assert!(steps < 10_000);
    let array = facts.types.intern(Kind::Array {
        element: ty,
        length: 1_000_000,
    })?;
    let repeated = Constant(facts.constant_products.repeat(value.0, 1_000_000)?);
    let (answer, work) = Reader::new(&facts).shared_mutation_work(array, repeated)?;
    assert_eq!(answer, Answer::Ordinary);
    assert!(work < steps + 32);
    let pending = facts
        .types
        .intern(Kind::Adt(ItemId(3), GenericArgs::EMPTY))?;
    assert_eq!(
        Reader::new(&facts).shared_mutation(pending, value)?,
        Answer::Unresolved
    );
    Ok(())
}

#[test]
fn references_require_actual_static_backing_and_raw_slots_stay_terminal() -> Result<(), String> {
    use crate::{
        resolve::model::DefId,
        sema::constants::references::{Access, Place},
        types::RegionId,
    };
    let mut facts = facts()?;
    let (atomic, value) = atomic(&mut facts)?;
    let reference = facts.types.intern(Kind::Reference {
        referent: atomic,
        region: RegionId::STATIC,
        mutable: false,
    })?;
    let raw = facts.types.intern(Kind::Pointer {
        pointee: atomic,
        mutable: false,
    })?;
    let owned = Constant(facts.constant_references.insert(Place {
        value: Some(value.0),
        ty: atomic,
        access: Access::Shared,
        packed: None,
        projection: None,
    })?);
    assert_eq!(
        Reader::new(&facts).shared_mutation(reference, owned)?,
        Answer::Ordinary
    );
    assert_eq!(
        Reader::new(&facts).shared_reference_storage(reference, owned)?,
        Answer::Interior
    );
    assert_eq!(
        Reader::new(&facts).shared_reference_storage(raw, owned)?,
        Answer::Ordinary
    );
    let static_address = Constant(facts.constant_references.static_place(DefId(17), atomic)?);
    assert_eq!(
        Reader::new(&facts).shared_reference_storage(reference, static_address)?,
        Answer::Ordinary
    );
    let slot = facts.types.intern_list(&[reference])?;
    let wrapped = facts.types.intern(Kind::Tuple(slot))?;
    let stored = product(&mut facts, &[owned], None)?;
    assert_eq!(
        Reader::new(&facts).shared_reference_storage(wrapped, stored)?,
        Answer::Interior
    );
    assert_eq!(
        Reader::new(&facts).shared_reference_storage(atomic, value)?,
        Answer::Ordinary
    );
    Ok(())
}

#[test]
fn references_to_empty_storage_refine_values_and_pending_backing_cannot_escape()
-> Result<(), String> {
    use crate::{
        sema::constants::references::{Access, Place},
        types::RegionId,
    };
    let mut facts = facts()?;
    let (atomic, _) = atomic(&mut facts)?;
    let array = facts.types.intern(Kind::Array {
        element: atomic,
        length: 0,
    })?;
    let reference = facts.types.intern(Kind::Reference {
        referent: array,
        region: RegionId::STATIC,
        mutable: false,
    })?;
    let value = product(&mut facts, &[], None)?;
    let address = Constant(facts.constant_references.insert(Place {
        value: Some(value.0),
        ty: array,
        access: Access::Shared,
        packed: None,
        projection: None,
    })?);
    assert_eq!(
        Reader::new(&facts).shared_reference_storage(reference, address)?,
        Answer::Ordinary
    );
    let pending = Constant(facts.constant_references.insert(Place {
        value: None,
        ty: array,
        access: Access::Shared,
        packed: None,
        projection: None,
    })?);
    assert!(
        Reader::new(&facts)
            .shared_reference_storage(reference, pending)
            .is_err()
    );
    Ok(())
}

#[test]
fn ordinary_subobject_reference_does_not_inherit_its_allocation_siblings() -> Result<(), String> {
    use crate::{
        sema::constants::references::{Access, Place},
        types::RegionId,
    };
    let mut facts = facts()?;
    let (atomic, atomic_value) = atomic(&mut facts)?;
    let integer = TypeId::primitive(Primitive::U32);
    let fields = facts.types.intern_list(&[atomic, integer])?;
    let pair = facts.types.intern(Kind::Tuple(fields))?;
    let scalar = Constant::scalar(Scalar::Integer(7));
    let value = product(&mut facts, &[atomic_value, scalar], None)?;
    let root = facts.constant_references.insert(Place {
        value: Some(value.0),
        ty: pair,
        access: Access::Shared,
        packed: None,
        projection: None,
    })?;
    let crate::sema::constants::Value::Reference(base) = root else {
        return Err("missing pointer".into());
    };
    let selected = Constant(facts.constant_references.project(
        base,
        1,
        Place {
            value: Some(scalar.0),
            ty: integer,
            access: Access::Shared,
            packed: None,
            projection: None,
        },
    )?);
    let reference = facts.types.intern(Kind::Reference {
        referent: integer,
        region: RegionId::STATIC,
        mutable: false,
    })?;
    assert_eq!(
        Reader::new(&facts).shared_mutation(pair, value)?,
        Answer::Interior
    );
    assert_eq!(
        Reader::new(&facts).shared_reference_storage(reference, selected)?,
        Answer::Ordinary
    );
    Ok(())
}
