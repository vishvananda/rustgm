//! Census records keep discovery order, root causes and sticky budget stops.
use super::*;
use crate::support::source::FileId;

#[test]
fn blocked_records_name_the_root_failure_and_repeats_are_recognized() {
    let mut census = Census::default();
    let first = Error::at(FileId(0), 7, "const parameter cannot be used as a type");
    let cause = census.failed(Phase::Declarations, "struct `Bad`".into(), &first);
    census.blocked(
        Phase::Declarations,
        "fn `uses`".into(),
        Location::Unlocated,
        cause,
    );
    census.blocked(
        Phase::Declarations,
        "fn `later`".into(),
        Location::Unlocated,
        1,
    );
    let same = Error::at(FileId(0), 7, "const parameter cannot be used as a type");
    let other = Error::at(FileId(0), 8, "const parameter cannot be used as a type");
    assert_eq!(census.repeated(&same), Some(cause));
    assert_eq!(census.repeated(&other), None);
    let causes: Vec<_> = census
        .records()
        .iter()
        .map(|record| record.blocked_by)
        .collect();
    assert_eq!(causes, [None, Some(cause), Some(cause)]);
    assert_eq!(
        census.counts(Phase::Declarations),
        Counts {
            checked: 0,
            failed: 1,
            blocked: 2
        }
    );
    assert!(census.has_failures(Phase::Declarations) && !census.has_failures(Phase::Bodies));
    assert!(census.terminal().is_none());
    census.stop_after_failures(Phase::Declarations);
    let terminal = census
        .terminal()
        .map(|(index, record)| (index, record.category));
    assert_eq!(terminal, Some((3, Category::Stopped)));
}

#[test]
fn budget_vocabulary_is_sticky_and_language_errors_are_not() {
    for message in [
        "constant execution frame limit exceeded",
        "source macro contracts exceed their work budget",
        "alias proof scope budget exhausted",
        "too many definitions",
        "body identity space exhausted",
        "source header contract query work exceeds its input-graph bound",
        "source contract work or retention exceeds its input-graph bound",
        "canonical projection premise work exhausted",
    ] {
        assert!(exhausts_budget(message), "{message}");
        let mut census = Census::default();
        census.stop(Phase::Bodies, &Error::unlocated(message));
        assert_eq!(
            census.terminal().map(|(_, record)| record.category),
            Some(Category::Budget)
        );
    }
    for message in [
        "incompatible types",
        "recursive type alias",
        "const parameter cannot be used as a type",
        "unresolved value name",
        "invalid canonical projection premise",
        "missing associated result contract",
    ] {
        assert!(!exhausts_budget(message), "{message}");
        let mut census = Census::default();
        census.stop(Phase::Frontend, &Error::unlocated(message));
        assert_eq!(
            census.terminal().map(|(_, record)| record.category),
            Some(Category::Crate)
        );
    }
}
