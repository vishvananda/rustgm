//! Candidate name completeness includes unknown heads and excludes unrelated owners.
use super::Pending;
use crate::{
    resolve::{model::DefId, names::NameId},
    types::{ItemId, subject_head::Head},
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
        reserve(&mut pending, DefId(id), Some(Head::Nominal(ItemId(id))));
        pending.insert(DefId(id), NameId(1), DefId(id + 10_000), false);
    }
    reserve(&mut pending, DefId(5000), None);
    pending.insert(DefId(5000), NameId(1), DefId(15_000), false);
    reserve(&mut pending, DefId(5001), Some(Head::Nominal(ItemId(1))));
    pending.insert(DefId(5001), NameId(2), DefId(15_001), false);
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
    reserve(&mut pending, DefId(3), Some(Head::Nominal(ItemId(7))));
    for (name, member) in [(1, 10), (1, 11), (2, 12)] {
        pending.insert(DefId(3), NameId(name), DefId(member), false);
    }
    reserve(&mut pending, DefId(4), Some(Head::Nominal(ItemId(7))));
    pending.insert(DefId(4), NameId(1), DefId(13), false);
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

fn reserve(pending: &mut Pending, owner: DefId, head: Option<Head>) {
    pending.owners.insert(
        owner,
        super::Members {
            head,
            all_marked: true,
            declarations: Vec::new(),
        },
    );
}

#[test]
fn structural_pending_names_preserve_mutability_and_empty_permission() {
    let mut pending = Pending::default();
    reserve(&mut pending, DefId(1), Some(Head::Pointer(false)));
    reserve(&mut pending, DefId(2), Some(Head::Pointer(true)));
    reserve(&mut pending, DefId(3), Some(Head::Array));
    assert!(!pending.permitted(DefId(1), false));
    assert!(pending.permitted(DefId(1), true));
    pending.insert(DefId(1), NameId(7), DefId(10), true);
    pending.insert(DefId(2), NameId(7), DefId(11), false);
    assert!(pending.permitted(DefId(1), false));
    assert!(!pending.permitted(DefId(2), true));
    assert_eq!(
        pending.head_dependencies(Head::Pointer(false), NameId(7)),
        vec![DefId(1)]
    );
    assert_eq!(
        pending.head_dependencies(Head::Pointer(true), NameId(7)),
        vec![DefId(2)]
    );
    assert!(!pending.contains_head(Head::Array, NameId(7)));
}
