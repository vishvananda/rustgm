//! Completed empty analysis is distinct from promoted and deferred roots.
use super::{Evaluated, Facts, ValueId};
use crate::sema::{model::Scalar, values::Constant};

#[test]
fn absent_facts_are_not_a_completed_promotion_analysis() {
    let absent: Option<Facts> = None;
    assert!(absent.is_none());
    assert!(absent.as_ref().is_none_or(Facts::is_empty));
}

#[test]
fn completed_empty_analysis_ignores_work_telemetry() {
    let empty = Facts {
        steps: 23,
        ..Facts::default()
    };
    assert!(empty.is_empty());
    assert!(Some(empty).as_ref().is_some_and(Facts::is_empty));
}

#[test]
fn raw_entries_and_index_each_prevent_empty_classification() {
    let mut facts = Facts::default();
    facts.entries.push((
        ValueId(0),
        Evaluated::Constant(Constant::scalar(Scalar::Integer(7))),
    ));
    assert!(!facts.is_empty());
    facts.index.insert(ValueId(0), 0);
    assert!(!facts.is_empty());
    facts.entries.clear();
    assert!(!facts.is_empty());
}

#[test]
fn deferred_roots_are_not_empty_filtered_constant_results() {
    let mut facts = Facts::default();
    facts.entries.push((ValueId(0), Evaluated::Deferred));
    facts.index.insert(ValueId(0), 0);
    assert!(facts.entries().next().is_none());
    assert_eq!(facts.deferred().next(), Some(ValueId(0)));
    assert!(!facts.is_empty());
}
