//! Storage property proofs preserve pending facts, owned composition and scope boundaries.
use super::{Answer, Query};
use crate::types::{
    GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
    aggregate::{Definitions, Kind as Aggregate, Repr, ShapeReadiness, SharedMutation, Variant},
    arguments::Argument,
    parameters::{Parameter, ParameterKind},
};

fn nominal(
    types: &mut Store,
    definitions: &mut Definitions,
    item: ItemId,
    fields: &[TypeId],
    mutation: SharedMutation,
) -> Result<TypeId, String> {
    let ty = types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    let fields = types.intern_list(fields)?;
    definitions.insert_with_mutation(
        item,
        Aggregate::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
        mutation,
    )?;
    Ok(ty)
}
fn interior(types: &mut Store, definitions: &mut Definitions) -> Result<TypeId, String> {
    nominal(
        types,
        definitions,
        ItemId(0),
        &[TypeId::primitive(Primitive::U32)],
        SharedMutation::Interior,
    )
}

#[test]
fn owned_fields_compose_but_reference_slots_do_not() -> Result<(), String> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let atomic = interior(&mut types, &mut definitions)?;
    let zero = types.intern(Kind::Array {
        element: atomic,
        length: 0,
    })?;
    let reference = types.intern(Kind::Reference {
        referent: atomic,
        region: RegionId::STATIC,
        mutable: false,
    })?;
    let pointer = types.intern(Kind::Pointer {
        pointee: atomic,
        mutable: true,
    })?;
    let wrapper = nominal(
        &mut types,
        &mut definitions,
        ItemId(1),
        &[atomic],
        SharedMutation::Ordinary,
    )?;
    definitions.suppress_field_destruction(ItemId(1))?;
    let mut query = Query::new(&types, &definitions);
    assert_eq!(query.contains(atomic)?, Answer::Interior);
    assert_eq!(query.contains(wrapper)?, Answer::Interior);
    // A type-level may-contain result is deliberately conservative. The pinned
    // source controls require evaluated-value refinement for empty promotion.
    assert_eq!(query.contains(zero)?, Answer::Interior);
    assert_eq!(query.contains(reference)?, Answer::Ordinary);
    assert_eq!(query.contains(pointer)?, Answer::Ordinary);
    assert_eq!(
        definitions.shared_mutation(ItemId(1)),
        Some(SharedMutation::Ordinary)
    );
    Ok(())
}

#[test]
fn pending_fields_require_a_new_immutable_query_scope() -> Result<(), String> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let atomic = interior(&mut types, &mut definitions)?;
    let item = ItemId(2);
    let parameter = types.intern_parameter(Parameter {
        owner: item,
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let subject = types.intern(Kind::Parameter(parameter))?;
    let fields = types.intern_list(&[subject])?;
    let arguments = types.intern_arguments(&[Argument::Type(atomic)])?;
    let ty = types.intern(Kind::Adt(item, arguments))?;
    {
        let mut query = Query::new(&types, &definitions);
        assert_eq!(query.contains(ty)?, Answer::Unresolved);
        assert_eq!(query.contains(subject)?, Answer::Unresolved);
    }
    definitions.insert(
        item,
        Aggregate::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    assert!(matches!(
        definitions.try_shape(&types, ty)?,
        ShapeReadiness::Pending
    ));
    {
        let mut query = Query::new(&types, &definitions);
        assert_eq!(query.contains(ty)?, Answer::Unresolved);
    }
    definitions.instantiate(&mut types, ty)?;
    assert!(matches!(
        definitions.try_shape(&types, ty)?,
        ShapeReadiness::Ready(_)
    ));
    assert_eq!(
        Query::new(&types, &definitions).contains(ty)?,
        Answer::Interior
    );
    assert!(
        definitions
            .insert_with_mutation(
                item,
                Aggregate::Struct,
                Repr::default(),
                &[],
                SharedMutation::Interior
            )
            .is_err()
    );
    Ok(())
}

#[test]
fn structural_storage_does_not_wait_for_discriminants() -> Result<(), String> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let atomic = interior(&mut types, &mut definitions)?;
    let empty = types.intern_list(&[])?;
    let fields = types.intern_list(&[atomic])?;
    let item = ItemId(3);
    let ty = types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    definitions.insert(
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
                discriminant: 0,
            },
        ],
    )?;
    definitions.mark_discriminants_pending(item)?;
    assert!(matches!(
        definitions.try_shape(&types, ty)?,
        ShapeReadiness::Ready(_)
    ));
    assert_eq!(
        Query::new(&types, &definitions).contains(ty)?,
        Answer::Interior
    );
    assert!(!definitions.discriminants_ready(item));
    Ok(())
}

#[test]
fn deep_shared_graphs_are_iterative_and_cached() -> Result<(), String> {
    for depth in [8, 128, 4096] {
        let mut types = Store::default();
        let mut definitions = Definitions::default();
        let mut root = interior(&mut types, &mut definitions)?;
        for _ in 0..depth {
            let fields = types.intern_list(&[root, TypeId::primitive(Primitive::U32)])?;
            root = types.intern(Kind::Tuple(fields))?;
        }
        let mut query = Query::new(&types, &definitions);
        assert_eq!(query.contains(root)?, Answer::Interior);
        let first = query.work();
        for _ in 0..128 {
            assert_eq!(query.contains(root)?, Answer::Interior);
        }
        let final_ = query.work();
        assert_eq!(final_.nodes, first.nodes);
        assert_eq!(final_.edges, first.edges);
        assert_eq!(final_.cache_hits - first.cache_hits, 128);
        assert!(final_.nodes <= depth + 2);
        assert!(final_.steps <= 16 * (depth + 129));
    }
    Ok(())
}

#[test]
fn malformed_storage_and_owned_cycles_do_not_prove_ordinary() -> Result<(), String> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let item = ItemId(4);
    let ty = types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    let fields = types.intern_list(&[ty])?;
    definitions.insert(
        item,
        Aggregate::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    assert_eq!(
        Query::new(&types, &definitions).contains(ty).err(),
        Some("owned cycle in shared mutation query")
    );
    assert!(
        Query::new(&types, &definitions)
            .contains(TypeId(u32::MAX))
            .is_err()
    );
    assert!(definitions.try_shape(&types, TypeId::UNIT).is_err());
    let pointer = types.intern(Kind::Pointer {
        mutable: false,
        pointee: ty,
    })?;
    let fields = types.intern_list(&[pointer])?;
    let pointer_owner = ItemId(5);
    let pointer_ty = types.intern(Kind::Adt(pointer_owner, GenericArgs::EMPTY))?;
    definitions.insert(
        pointer_owner,
        Aggregate::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    assert_eq!(
        Query::new(&types, &definitions).contains(pointer_ty)?,
        Answer::Ordinary
    );
    Ok(())
}
