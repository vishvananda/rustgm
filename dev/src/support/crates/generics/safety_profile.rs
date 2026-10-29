//! Earlier source profiles cannot consume or silently erase unsafe-call contexts.
use super::{Profile, SourceRows, Work};
use crate::support::crates::generics::source::CallSafety;

pub(super) fn require_profile(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    if profile.call_safety() {
        return Ok(());
    }
    for original in &rows.originals {
        for request in &original.calls {
            work.charge(1)?;
            if request.safety != CallSafety::Safe {
                return Err("unsafe source call contexts require the version-eight profile");
            }
        }
    }
    Ok(())
}
