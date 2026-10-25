//! Unique warning ownership survives projection and repeated bounded error merges.
use super::{BYTES, Error, Warning};
use crate::{resolve::unit::FileId, support::diagnostics::WarningCode};
fn warning(offset: u32) -> Warning {
    Warning { code: WarningCode::UnusedAttributes, diagnostic: Error::at(FileId(0), offset, "declared warning") }
}
#[test]
fn first_error_transfer_moves_the_actual_vec_and_preserves_primary_display() {
    let warnings = vec![warning(7), warning(9)];
    let pointer = warnings.as_ptr();
    let primary = Error::at(FileId(0), 31, "primary failure");
    let before = primary.to_string();
    let error = primary.retain_warnings(warnings);
    assert_eq!(size_of::<Error>(), size_of::<usize>());
    assert_eq!(error.warnings().as_ptr(), pointer);
    assert_eq!(error.to_string(), before);
    assert_eq!(error.warnings().iter().map(|warning| warning.diagnostic.span.start).collect::<Vec<_>>(), [7, 9]);
    let error = error.retain_warnings(Vec::new());
    assert_eq!(error.warnings().len(), 2);
    assert_eq!(error.to_string(), before);
}
#[test]
fn repeated_single_warning_merges_visit_only_new_records() {
    let mut error = Error::at(FileId(0), 31, "primary failure");
    for offset in 0..4096 { error = error.retain_warnings(vec![warning(offset)]); }
    assert_eq!(error.warnings().len(), 4096);
    assert_eq!(error.warning_work, 4096);
    assert!(error.warnings().iter().enumerate().all(|(index, warning)| warning.diagnostic.span.start as usize == index));
    assert!(!error.warning_limit_note);
}
#[test]
fn retention_limit_keeps_the_original_failure_and_one_bounded_note() {
    let mut error = Error::at(FileId(0), 31, "primary failure").retain_warnings(vec![warning(7)]);
    error.warning_bytes = BYTES;
    let error = error.retain_warnings(vec![warning(9)]).retain_warnings(vec![warning(11)]);
    assert_eq!(error.warnings().len(), 1);
    assert_eq!(error.message, "primary failure");
    assert_eq!(error.span.start, 31);
    assert_eq!(error.to_string().matches("note: retained warning").count(), 1);
}

#[test]
fn repeated_source_projection_keeps_the_same_owned_warning_sink() -> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve("fn ordinary(){}", true)?;
    let error = Error::at(FileId(0), 7, "primary failure").retain_warnings(vec![warning(9)]);
    let pointer = error.warnings().as_ptr();
    let bytes = error.warning_bytes;
    let error = report.diagnostic(report.diagnostic(error));
    assert_eq!(error.warnings().len(), 1);
    assert_eq!(error.warnings().as_ptr(), pointer);
    assert_eq!(error.warning_bytes, bytes);
    assert_eq!(error.to_string(), "file 0: 7: primary failure");
    Ok(())
}
