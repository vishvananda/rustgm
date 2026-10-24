//! Candidate name completeness includes unknown heads and excludes unrelated owners.
use super::Pending;
use crate::{
    resolve::{model::DefId, names::NameId},
    types::ItemId,
};

#[test]
fn lookup_and_retirement_are_partitioned_by_nominal_and_name() {
    eprintln!(
        "inherent dependency layouts Requirement={} Dependency={} DeclarationWork={} PendingHeaders={}",
        size_of::<crate::sema::bodies::Requirement>(),
        size_of::<crate::sema::bodies::Dependency>(),
        size_of::<super::super::Work>(),
        size_of::<Pending>()
    );

    let mut pending = Pending::default();
    for id in 0..4096 {
        pending.insert(DefId(id), Some(ItemId(id)), NameId(1), DefId(id + 10_000));
    }
    pending.insert(DefId(5000), None, NameId(1), DefId(15_000));
    pending.insert(DefId(5001), Some(ItemId(1)), NameId(2), DefId(15_001));
    for id in 0..4096 {
        assert_eq!(
            pending.dependencies(ItemId(id), NameId(1)),
            vec![DefId(id), DefId(5000)]
        );
        assert!(pending.contains(ItemId(id), NameId(1)));
        assert!(pending.remove(DefId(id)).is_some());
        assert_eq!(
            pending.dependencies(ItemId(id), NameId(1)),
            vec![DefId(5000)]
        );
    }
    assert_eq!(
        pending.dependencies(ItemId(1), NameId(2)),
        vec![DefId(5001)]
    );
    assert!(pending.remove(DefId(5000)).is_some());
    assert!(!pending.contains(ItemId(1), NameId(1)));
    assert!(pending.remove(DefId(5001)).is_some());
    assert!(pending.owners.is_empty());
    assert!(pending.buckets.is_empty());
}

#[test]
fn an_owner_publishes_all_names_and_duplicate_declarations_once() {
    let mut pending = Pending::default();
    for (name, member) in [(1, 10), (1, 11), (2, 12)] {
        pending.insert(DefId(3), Some(ItemId(7)), NameId(name), DefId(member));
    }
    pending.insert(DefId(4), Some(ItemId(7)), NameId(1), DefId(13));
    assert_eq!(
        pending.dependencies(ItemId(7), NameId(1)),
        vec![DefId(3), DefId(4)]
    );
    let members = pending.remove(DefId(3));
    assert_eq!(
        members.map(|m| m.declarations),
        Some(vec![
            (NameId(1), DefId(10)),
            (NameId(1), DefId(11)),
            (NameId(2), DefId(12))
        ])
    );
    assert_eq!(pending.dependencies(ItemId(7), NameId(1)), vec![DefId(4)]);
    assert!(!pending.contains(ItemId(7), NameId(2)));
    assert!(pending.remove(DefId(3)).is_none());
}
