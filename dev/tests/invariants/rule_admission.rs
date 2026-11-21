//! An oversized optional result-rule entry leaves other entries reusable.
use super::{Arc, Assumptions, Cache, ItemId, TypeId};

#[test]
fn declining_large_rule_entries_preserves_existing_and_later_hits() {
    let mut cache = Cache::default();
    cache.enable();
    let rules = Arc::new(Assumptions::default());
    let ty = TypeId::UNIT;
    cache.insert(ty, ItemId(0), &rules, 1);
    cache.insert(ty, ItemId(1), &rules, usize::MAX);
    cache.insert(ty, ItemId(2), &rules, 1);

    assert!(cache.get(ty, ItemId(1)).is_none());
    for item in [ItemId(0), ItemId(2)] {
        assert!(
            cache
                .get(ty, item)
                .is_some_and(|cached| Arc::ptr_eq(&cached, &rules))
        );
    }
}
