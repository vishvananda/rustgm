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

/// The split fixed messages: exhaustion stays a sticky budget stop by its
/// wording, and an invalid cause is an ordinary item failure.
const SPLIT: [&str; 12] = [
    "receiver inference graph",
    "method trait selection",
    "mapped caller membership",
    "nominal trait proof",
    "associated equality proof",
    "canonical trait selection",
    "associated equality selection",
    "inherited result defining-header proof",
    "source Metadata plan",
    "source Metadata Sized proof",
    "call result candidate",
    "proof source extension",
];

#[test]
fn split_fixed_messages_keep_exhaustion_apart_from_invalid_causes() {
    for subject in SPLIT {
        let invalid = format!("invalid {subject}");
        assert!(!exhausts_budget(&invalid), "{invalid}");
        let exhausted = if subject == "proof source extension" {
            format!("{subject} capacity exhausted")
        } else {
            format!("{subject} work exhausted")
        };
        assert!(exhausts_budget(&exhausted), "{exhausted}");
    }
    let budget = crate::types::infer::Error::Budget;
    assert_eq!(fold_cause(&budget, "exhausted", "invalid"), "exhausted");
    let overflow = crate::types::infer::Error::Overflow;
    assert_eq!(fold_cause(&overflow, "exhausted", "invalid"), "exhausted");
    let mismatch = crate::types::infer::Error::Mismatch;
    assert_eq!(fold_cause(&mismatch, "exhausted", "invalid"), "invalid");
    let selection = crate::sema::traits::selection::Failure::Invalid("incompatible types".into());
    assert_eq!(fold_cause(&selection, "exhausted", "invalid"), "invalid");
    let message = "projection work budget exceeded";
    assert_eq!(fold_cause(&message, "exhausted", "invalid"), "exhausted");
}
