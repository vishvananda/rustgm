//! Earlier source profiles cannot consume or silently erase two-way/failure exits
//! or the primitive Binary operations that only version thirteen encodes.
use super::{Profile, SourceRows, Work};
use crate::support::crates::generics::source::ValueKind;

pub(super) fn require_profile(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    if !profile.body_nodes() {
        for original in &rows.originals {
            for value in &original.values {
                work.charge(1)?;
                if matches!(value.kind, ValueKind::Binary { .. }) {
                    return Err(
                        "source Binary operations require the version-thirteen body IR profile",
                    );
                }
            }
        }
    }
    if profile.branches() {
        return Ok(());
    }
    for original in &rows.originals {
        for block in &original.blocks {
            work.charge(1)?;
            if block.terminator.requires_branch_profile() {
                return Err("source Branch/Panic CFG exits require the version-twelve profile");
            }
        }
    }
    Ok(())
}
