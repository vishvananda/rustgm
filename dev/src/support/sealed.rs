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
    REQUESTS.load(Ordering::Relaxed) != 0 || held()
}

#[cfg(not(test))]
const fn held() -> bool {
    false
}

#[cfg(test)]
thread_local! {
    /// Test holders on this thread. Each test runs on its own thread, so a
    /// holder never switches the checks on for a test running beside it.
    static HELD: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
fn held() -> bool {
    HELD.with(|held| held.get() != 0)
}

/// Hold the self-check mode for one scope of the current thread. Nested
/// holders compose; a thread the test spawns does not inherit it.
#[cfg(test)]
pub(crate) struct SelfCheck(());
#[cfg(test)]
impl SelfCheck {
    pub(crate) fn hold() -> Self {
        HELD.with(|held| held.set(held.get() + 1));
        Self(())
    }
}
#[cfg(test)]
impl Drop for SelfCheck {
    fn drop(&mut self) {
        HELD.with(|held| held.set(held.get() - 1));
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/check_holders.rs"]
mod tests;
