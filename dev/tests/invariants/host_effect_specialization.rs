//! rustc 1.98's `min_specialization` compares `predicates_of`, which holds a
//! `[const] Tr` bound as the ordinary `Tr` clause. Core's `SpecNextChunk` pair
//! (`array/mod.rs:1044` and `1062`) therefore adds only its `TrustedLen` marker,
//! although the marker's `[const] Iterator` supertrait elaborates to the
//! ordinary `I: Iterator` beside the parent's `[const]` bound. Constness is
//! still checked where `specializes` checks it: a parent that is const in more
//! cases than its child orders nothing, and a `const` marker bound is a
//! host-effect clause, never a marker clause.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/const-specialization-bounds";
const UNORDERED: &str = "overlapping implementation headers have no specialization ordering";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 7] = [
    ("core-next-chunk", None),
    ("plain-marker-supertrait", None),
    ("child-only-const", None),
    ("ordinary-pair", None),
    ("child-not-const", Some(UNORDERED)),
    ("child-weaker-const", Some(UNORDERED)),
    (
        "const-marker-bound",
        Some("minimal specialization introduces a non-marker predicate"),
    ),
];

#[test]
fn const_bounds_specialize_by_their_ordinary_clauses_and_keep_their_conditions() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "const_specialization_bounds")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
