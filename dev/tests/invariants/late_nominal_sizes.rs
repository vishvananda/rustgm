//! A late leaf keeps old snapshots immutable and cannot import another size contract.
use super::{ItemId, Kind, Rule, Rules, Store, TypeId};
use crate::types::{
    GenericArgs, Primitive,
    aggregate::{Definitions, Kind as AggregateKind, Repr, Variant},
};
use std::sync::Arc;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fields(store: &mut Store, definitions: &mut Definitions, item: ItemId, ty: TypeId) -> Result {
    let fields = store.intern_list(&[ty])?;
    definitions.insert(
        item,
        AggregateKind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    Ok(())
}
fn fixture() -> Result<(Store, Definitions, ItemId)> {
    let mut store = Store::default();
    let mut definitions = Definitions::default();
    fields(
        &mut store,
        &mut definitions,
        ItemId(1),
        TypeId::primitive(Primitive::U8),
    )?;
    let mut rules = Rules::default();
    assert_eq!(
        rules.prepare(&mut store, &definitions, ItemId(1), |_, _, _| Ok(false))?,
        None
    );
    store.seal_nominal_sizes(Arc::new(rules))?;
    let old = store.intern(Kind::Adt(ItemId(1), GenericArgs::EMPTY))?;
    fields(
        &mut store,
        &mut definitions,
        ItemId(2),
        TypeId::primitive(Primitive::U32),
    )?;
    fields(&mut store, &mut definitions, ItemId(3), old)?;
    Ok((store, definitions, ItemId(2)))
}
#[test]
fn late_sized_leaf_keeps_unique_and_shared_snapshots_and_old_entries() -> Result {
    for shared in [false, true] {
        let (mut store, definitions, item) = fixture()?;
        let mut prior = if shared {
            let (mut prior, _, _) = fixture()?;
            prior.nominal_sizes = store.nominal_sizes.as_ref().map(Arc::clone);
            Some(prior)
        } else {
            None
        };
        let old = store.nominal_size_rule(ItemId(1));
        let work = store.append_sized_leaf(&definitions, item)?;
        assert!(work > 0);
        assert_eq!(store.nominal_size_rule(item), Some(Rule::Sized));
        assert_eq!(store.nominal_size_rule(ItemId(1)), old);
        let ty = store.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
        if let Some(prior) = &mut prior {
            assert_eq!(prior.nominal_size_rule(item), None);
            assert_eq!(prior.nominal_size_rule(ItemId(1)), old);
            assert!(prior.kind(ty).is_none());
            assert!(prior.intern(Kind::Adt(item, GenericArgs::EMPTY)).is_err());
        }
        assert!(store.append_sized_leaf(&definitions, item).is_err());
    }
    Ok(())
}
#[test]
fn late_sized_leaf_rejects_dependencies_foreign_ids_and_exhaustion_before_publication() -> Result {
    let (mut store, definitions, item) = fixture()?;
    for wrong in [ItemId(3), ItemId(99)] {
        assert!(store.append_sized_leaf(&definitions, wrong).is_err());
        assert_eq!(store.nominal_size_rule(wrong), None);
        assert_eq!(store.nominal_size_rule(ItemId(1)), Some(Rule::Sized));
    }
    Arc::get_mut(store.nominal_sizes.as_mut().ok_or("rules")?)
        .ok_or("unique rules")?
        .steps = 1_000_000;
    let snapshot = Arc::clone(store.nominal_sizes.as_ref().ok_or("rules")?);
    assert!(store.append_sized_leaf(&definitions, item).is_err());
    assert_eq!(store.nominal_size_rule(item), None);
    assert_eq!(snapshot.get(item), None);
    assert_eq!(
        store.nominal_sizes.as_ref().ok_or("rules")?.steps,
        1_000_000
    );
    Ok(())
}
