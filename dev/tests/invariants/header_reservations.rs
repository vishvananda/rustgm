//! Aggregate reservation fails before publishing or allocating a header row.
use super::*;
#[test]
fn header_storage_preflight_keeps_failed_growth_and_no_partial_rows() {
    let mut headers = Headers {
        base: crate::support::crates::MAX_BYTES - 1,
        ..Headers::default()
    };
    assert!(headers.reserve(2).as_ref().is_err_and(|error| matches!(
        &error.file,
        crate::resolve::package::DiagnosticFile::Unlocated
    )));
    assert_eq!(headers.bytes, 2);
    assert_eq!(headers.entries.capacity(), 0);
    assert_eq!(headers.graphs.capacity(), 0);
    let bound = Environment::builtin_allocation_bound(usize::MAX, 1, 1, 1);
    assert_eq!(bound, usize::MAX);
    assert!(headers.reserve(bound).is_err());
    assert_eq!(headers.bytes, usize::MAX);
    assert_eq!(headers.observed.bytes, 0);
}
