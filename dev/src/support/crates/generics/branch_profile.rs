//! Earlier source profiles cannot consume or silently erase two-way/failure exits.
use super::{Profile, SourceRows, Work};

pub(super) fn require_profile(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
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
