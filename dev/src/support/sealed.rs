//! Sealed tables are checked once at publication and then read as plain fields.
//! Repeating those checks on every read is an explicit self-check mode.
use std::sync::atomic::{AtomicUsize, Ordering};

/// The number of live requests; it is process-wide so that an ordinary
/// compilation carries no self-check state in any table.
static REQUESTS: AtomicUsize = AtomicUsize::new(0);

/// Repeat every sealed table's publication checks on read for the rest of
/// this process. The driver selects it before loading any source.
pub fn request_self_check() {
    REQUESTS.fetch_add(1, Ordering::Relaxed);
}

/// Whether a read of a sealed table must repeat its publication checks.
#[must_use]
pub fn self_check() -> bool {
    REQUESTS.load(Ordering::Relaxed) != 0
}

/// Hold the self-check mode for one scope. Overlapping holders compose, so
/// concurrent tests cannot switch each other's checks off.
#[cfg(test)]
pub(crate) struct SelfCheck(());
#[cfg(test)]
impl SelfCheck {
    pub(crate) fn hold() -> Self {
        request_self_check();
        Self(())
    }
}
#[cfg(test)]
impl Drop for SelfCheck {
    fn drop(&mut self) {
        REQUESTS.fetch_sub(1, Ordering::Relaxed);
    }
}
