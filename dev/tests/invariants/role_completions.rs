//! Real role identities stay distinct; storage is proportional to known roles only.
use super::{RoleSet, WORDS};
use crate::support::bindings::LangItem;

#[test]
fn every_actual_role_has_distinct_completion_and_union_identity() {
    let mut complete = RoleSet::EMPTY;
    for role in LangItem::ALL {
        assert!(!complete.contains(role), "duplicate role index: {role:?}");
        let mut selected = RoleSet::EMPTY;
        selected.insert(role);
        assert!(!complete.intersects(selected));
        assert!(!selected.is_empty());
        complete.extend(selected);
        assert!(complete.contains(role));
    }
    for role in LangItem::ALL {
        let mut missing = complete;
        missing.remove(role);
        assert_ne!(missing, complete);
        assert!(!missing.contains(role));
        missing.insert(role);
        assert_eq!(missing, complete);
    }
    let mut empty = complete;
    for role in LangItem::ALL {
        empty.remove(role);
    }
    assert_eq!(empty, RoleSet::EMPTY);
    assert_eq!(size_of::<RoleSet>(), WORDS * size_of::<u64>());
    assert_eq!(WORDS, LangItem::ALL.len().div_ceil(64));
    println!(
        "completion-set roles={} words={WORDS} bytes={} ledger={}",
        LangItem::ALL.len(),
        size_of::<RoleSet>(),
        size_of::<super::super::Validated>()
    );
}
