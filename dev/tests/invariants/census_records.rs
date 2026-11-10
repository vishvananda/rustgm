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

/// Ambiguity and the unmapped-lifetime invariant keep their causes' own texts
/// in whatever shape the cause arrives, and those texts are their single
/// producers' own, so a reworded producer fails here instead of folding to
/// "invalid".
#[test]
fn folds_keep_ambiguity_and_the_unmapped_lifetime_invariant_apart() {
    use crate::sema::traits::selection::{Dependency, Failure};
    use crate::types::{
        RegionId,
        infer::{self, UNRESOLVED},
        substitute::{Bound, UNMAPPED_LIFETIME},
    };
    assert_eq!(infer::Error::Unresolved.to_string(), UNRESOLVED);
    let mut restore = Bound::restore_regions(crate::support::hash::HashMap::default());
    assert_eq!(restore.region(RegionId(7), 0), Err(UNMAPPED_LIFETIME));
    for (text, kind) in [
        (UNRESOLVED, CauseKind::Ambiguous),
        (UNMAPPED_LIFETIME, CauseKind::UnmappedLifetime),
    ] {
        assert_eq!(text.kind(), kind);
        assert_eq!(fold_cause(&text, "exhausted", "invalid"), text);
        assert_eq!(fold_cause(&text.to_string(), "exhausted", "invalid"), text);
        let selection = Failure::Invalid(text.into());
        assert_eq!(fold_cause(&selection, "exhausted", "invalid"), text);
        // A message that only mentions the text is an ordinary failure.
        let wrapped = format!("{text} in a receiver");
        assert_eq!(fold_cause(&wrapped, "exhausted", "invalid"), "invalid");
    }
    assert_eq!(infer::Error::Unresolved.kind(), CauseKind::Ambiguous);
    assert_eq!(
        fold_cause(&infer::Error::Unresolved, "exhausted", "invalid"),
        UNRESOLVED
    );
    for error in [
        infer::Error::InvalidIdentity,
        infer::Error::Mismatch,
        infer::Error::Occurs,
        infer::Error::Normalization,
    ] {
        assert_eq!(fold_cause(&error, "exhausted", "invalid"), "invalid");
    }
    let pending = Failure::Pending(Dependency::Header(crate::resolve::model::DefId(0)));
    assert_eq!(fold_cause(&pending, "exhausted", "invalid"), "invalid");
}

#[test]
fn lookahead_records_follow_the_failed_pass_and_change_no_count() {
    let mut census = Census::default();
    let later = Error::at(FileId(0), 9, "recursive type has infinite size");
    census.look_ahead(
        Phase::Declarations,
        "validate_signatures",
        [("complete_layouts", later)],
    );
    assert!(
        census.records().is_empty(),
        "a lookahead needs a terminal record"
    );
    census.checked(Phase::Declarations, 4);
    let failed = Error::at(FileId(0), 3, "invalid canonical trait selection");
    census.stop(Phase::Declarations, &failed);
    let later = [
        (
            "complete_layouts",
            Error::at(FileId(0), 9, "recursive type has infinite size"),
        ),
        (
            "check_api_exposure",
            Error::unlocated("private type in public interface"),
        ),
    ];
    census.look_ahead(Phase::Declarations, "validate_signatures", later);
    let rows: Vec<_> = census
        .records()
        .iter()
        .map(|record| {
            (
                record.category.name(),
                record.item.as_deref(),
                record.blocked_by,
                record.message.as_str(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            (
                "crate",
                Some("pass `validate_signatures`"),
                None,
                "invalid canonical trait selection"
            ),
            (
                "lookahead",
                Some("pass `complete_layouts`"),
                Some(0),
                "recursive type has infinite size"
            ),
            (
                "lookahead",
                Some("pass `check_api_exposure`"),
                Some(0),
                "private type in public interface"
            ),
        ]
    );
    let terminal = census
        .terminal()
        .map(|(index, record)| (index, record.category));
    assert_eq!(terminal, Some((0, Category::Crate)));
    assert_eq!(
        census.counts(Phase::Declarations),
        Counts {
            checked: 4,
            failed: 0,
            blocked: 0
        }
    );
    assert!(!census.has_failures(Phase::Declarations));
}

#[test]
fn partial_lookahead_records_are_blocked_and_counted() {
    let mut census = Census::default();
    census.checked(Phase::Declarations, 4);
    let failed = Error::at(FileId(0), 3, "trait implementation does not satisfy");
    census.stop(Phase::Declarations, &failed);
    let later = [
        (
            "validate_trait_impl_owners",
            Error::at(FileId(0), 3, "requires conformance"),
        ),
        (
            "complete_discriminants",
            Error::at(FileId(0), 7, PARTIAL_EPOCH),
        ),
        (
            "complete_layouts",
            Error::at(FileId(0), 5, "enum layout requires completed discriminants"),
        ),
    ];
    census.look_ahead(Phase::Declarations, "validate_trait_implementations", later);
    let rows: Vec<_> = census
        .records()
        .iter()
        .map(|record| (record.category, record.blocked_by, record.message.as_str()))
        .collect();
    assert_eq!(
        rows,
        [
            (
                Category::Crate,
                None,
                "trait implementation does not satisfy"
            ),
            (Category::Lookahead, Some(0), "requires conformance"),
            (Category::Blocked, Some(0), PARTIAL_EPOCH),
            (
                Category::Lookahead,
                Some(0),
                "enum layout requires completed discriminants"
            ),
        ]
    );
    assert_eq!(
        census.records()[2].location,
        Location::Source { file: 0, offset: 7 }
    );
    assert_eq!(
        census.counts(Phase::Declarations),
        Counts {
            checked: 4,
            failed: 0,
            blocked: 1
        }
    );
    assert_eq!(census.terminal().map(|(index, _)| index), Some(0));
}

#[test]
fn only_the_partial_epoch_text_moves_a_lookahead_count() {
    let near = [
        format!("source {PARTIAL_EPOCH}"),
        format!("{PARTIAL_EPOCH}."),
        PARTIAL_EPOCH.replacen("epoch", "epocs", 1),
        PARTIAL_EPOCH.to_uppercase(),
        "constant execution frame limit exceeded".to_owned(),
        "associated type requires an unambiguous validated implementation".to_owned(),
    ];
    let mut census = Census::default();
    census.stop(
        Phase::Declarations,
        &Error::at(FileId(0), 1, "trait implementation does not satisfy"),
    );
    census.look_ahead(
        Phase::Declarations,
        "validate_trait_implementations",
        near.iter()
            .map(|message| ("complete_discriminants", Error::at(FileId(0), 2, message))),
    );
    assert_eq!(census.records().len(), 1 + near.len());
    assert!(
        census.records()[1..]
            .iter()
            .all(|record| record.category == Category::Lookahead && record.blocked_by == Some(0))
    );
    assert_eq!(census.counts(Phase::Declarations), Counts::default());
}

#[test]
fn diagnostic_observer_failure_is_sticky_without_changing_census_records() {
    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
    let observed = calls.clone();
    let mut census = Census::with_observer(Box::new(move |event| {
        assert!(matches!(event, Progress::Record { index: 0, .. }));
        observed.set(observed.get() + 1);
        Err("diagnostic output failed".into())
    }));
    let error = Error::at(FileId(0), 0, "incompatible types");
    census.failed(Phase::Bodies, "first".into(), &error);
    census.failed(Phase::Bodies, "second".into(), &error);
    assert_eq!(calls.get(), 1);
    assert_eq!(census.progress_error(), Some("diagnostic output failed"));
    assert_eq!(census.records().len(), 2);
    assert_eq!(census.counts(Phase::Bodies).failed, 2);
}
