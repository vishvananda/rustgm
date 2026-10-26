//! One graph budget survives suspension, nested proof queries and sealing.
use std::{cell::Cell, rc::Rc};
#[derive(Clone, Copy)]
struct Work {
    steps: usize,
    bytes: usize,
}
pub(super) struct Budget(Rc<Cell<Work>>);
impl Budget {
    pub(super) fn new(owners: usize) -> Result<Self, &'static str> {
        // Conservative logical bookkeeping, not an allocator-usage measurement.
        let work = Work {
            steps: 1,
            bytes: owners
                .saturating_add(2 * size_of::<usize>())
                .saturating_add(size_of::<Cell<Work>>()),
        };
        check(work)?;
        Ok(Self(Rc::new(Cell::new(work))))
    }
    pub(super) fn alias(&self) -> Result<Self, &'static str> {
        self.charge(1, 0)?;
        Ok(Self(Rc::clone(&self.0)))
    }
    pub(super) fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        let old = self.0.get();
        let work = Work {
            steps: old.steps.saturating_add(steps),
            bytes: old.bytes.saturating_add(bytes),
        };
        self.0.set(work);
        check(work)
    }
    pub(super) fn same_owner(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
    #[cfg(test)]
    pub(super) fn usage(&self) -> (usize, usize) {
        let work = self.0.get();
        (work.steps, work.bytes)
    }
}
fn check(work: Work) -> Result<(), &'static str> {
    if work.steps > 10_000_000 || work.bytes > 64 * 1024 * 1024 {
        return Err("specialization graph work or retention budget exceeded");
    }
    Ok(())
}

/// Proof-local counters always reconcile on failure; aliases share one envelope.
pub(super) trait Counter {
    fn steps(&self) -> usize;
}
impl Counter for crate::types::infer::Context {
    fn steps(&self) -> usize {
        self.steps
    }
}
impl Counter for crate::types::substitute::Bound {
    fn steps(&self) -> usize {
        self.steps
    }
}
pub(super) struct Counted<T: Counter> {
    value: T,
    budget: Budget,
    paid: usize,
}
impl<T: Counter> Counted<T> {
    pub(super) fn new(value: T, budget: &Budget) -> Result<Self, &'static str> {
        Ok(Self {
            value,
            budget: budget.alias()?,
            paid: 0,
        })
    }
    pub(super) fn pay(&mut self) -> Result<(), &'static str> {
        let attempted = self.value.steps();
        let delta = attempted.saturating_sub(self.paid);
        self.paid = attempted;
        self.budget.charge(delta, 0)
    }
}
impl<T: Counter> std::ops::Deref for Counted<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl<T: Counter> std::ops::DerefMut for Counted<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
impl<T: Counter> Drop for Counted<T> {
    fn drop(&mut self) {
        // Only failed/abandoned paths can retain an unpaid delta. The shared
        // envelope records it even when its limit is already exhausted; the
        // original failure remains primary and no result is published here.
        let _ = self.pay();
    }
}
