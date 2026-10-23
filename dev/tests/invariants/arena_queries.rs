//! Exact exposure caches retain arena identity across moves, never replacement.
use super::*;
use crate::types::{Kind, Primitive};
const LIMIT: usize = 1024 * 1024;

#[test]
fn exact_exposure_preflights_before_allocating_lazy_arena_tokens() {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    for limit in [0, 1, 64] {
        assert!(NeedsDrop::for_exposure(&mut types, &mut definitions, limit).is_err());
        assert!(types.identity.is_none());
    }
    // Comparing with a freshly minted independent token does not allocate in either arena.
    let foreign = Identity::new();
    assert!(!types.has_identity(&foreign));
    assert!(!definitions.has_identity(&foreign));
}

#[test]
fn exact_exposure_warm_false_rejects_equal_id_replacement_in_either_arena()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let mut query = NeedsDrop::for_exposure(&mut types, &mut definitions, LIMIT)?;
    assert!(!query.demand_catalogue(&types, &definitions, TypeId::UNIT, LIMIT)?);
    let before = query.steps;
    let mut foreign_types = Store::default();
    let mut foreign_definitions = Definitions::default();
    // Both foreign inventories contain the identical primitive IDs. Even initializing
    // their tokens cannot make those completed false facts interchangeable.
    foreign_types.arena_identity();
    foreign_definitions.arena_identity();
    for (types, definitions) in [
        (&foreign_types, &definitions),
        (&types, &foreign_definitions),
        (&foreign_types, &foreign_definitions),
    ] {
        assert_eq!(
            query.demand_catalogue(types, definitions, TypeId::UNIT, LIMIT),
            Err("exact exposure query belongs to another arena")
        );
        assert_eq!(query.steps, before);
    }
    let original_types = std::mem::take(&mut types);
    let original_definitions = std::mem::take(&mut definitions);
    assert!(
        query
            .demand_catalogue(&types, &definitions, TypeId::UNIT, LIMIT)
            .is_err()
    );
    assert!(!query.demand_catalogue(
        &original_types,
        &original_definitions,
        TypeId::UNIT,
        LIMIT
    )?);
    Ok(())
}

#[test]
fn exact_exposure_identity_survives_moves_and_append_only_type_growth() -> Result<(), &'static str>
{
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let mut query = NeedsDrop::for_exposure(&mut types, &mut definitions, LIMIT)?;
    let first = types.arena_identity();
    let second = definitions.arena_identity();
    let (mut moved_types, mut moved_definitions) = (types, definitions);
    assert!(moved_types.has_identity(&first));
    assert!(moved_definitions.has_identity(&second));
    for length in 0..128 {
        moved_types.intern(Kind::Array {
            element: TypeId::primitive(Primitive::U8),
            length,
        })?;
    }
    let fields = moved_types.intern_list(&[TypeId::UNIT])?;
    moved_definitions.insert(
        crate::types::ItemId(99),
        crate::types::aggregate::Kind::Struct,
        crate::types::aggregate::Repr::default(),
        &[crate::types::aggregate::Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    assert!(moved_types.has_identity(&first));
    assert!(moved_definitions.has_identity(&second));
    assert!(!query.demand_catalogue(&moved_types, &moved_definitions, TypeId::UNIT, LIMIT)?);
    let bytes = query.retained_bytes();
    let before = query.steps;
    for _ in 0..128 {
        assert!(!query.demand_catalogue(&moved_types, &moved_definitions, TypeId::UNIT, LIMIT)?);
    }
    assert_eq!(query.steps, before + 128);
    assert_eq!(query.retained_bytes(), bytes);
    println!(
        "Store={} Definitions={} Scope={} Binding={} query-bytes={bytes}",
        size_of::<Store>(),
        size_of::<Definitions>(),
        size_of::<Scope>(),
        size_of::<Binding>()
    );
    Ok(())
}

#[test]
fn dense_and_unbound_sparse_modes_do_not_claim_exact_exposure_authority() -> Result<(), &'static str>
{
    let types = Store::default();
    let definitions = Definitions::default();
    let mut dense = NeedsDrop::new([]);
    assert_eq!(dense.catalogue_bytes()?, None);
    assert!(!dense.demand_catalogue(&types, &definitions, TypeId::UNIT, LIMIT)?);
    assert!(types.identity.is_none());
    let unbound = NeedsDrop::concrete_sparse([]);
    assert!(unbound.catalogue_bytes().is_err());
    Ok(())
}
