//! Whole-arm contracts retain exact typed codes, counts, order and byte locations.
use rustgm::support::diagnostics::WarningCode;
use std::fmt::Write;
pub(crate) fn unreachable(
    actual: &[u8],
    offsets: &[u32],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut expected = String::new();
    for offset in offsets {
        writeln!(
            expected,
            "warning[{}]: file 0: {offset}: unreachable pattern",
            WarningCode::UnreachablePatterns.name()
        )?;
    }
    if actual != expected.as_bytes() {
        return Err(format!(
            "exact unreachable warning contract differs: expected {expected:?}, actual {:?}",
            String::from_utf8_lossy(actual)
        )
        .into());
    }
    Ok(())
}
