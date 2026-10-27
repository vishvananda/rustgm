//! Counted callers prepay the same fold's actual queues, maps and binder frames.
use super::{Key, Kind, Normalizer, Resolution, Store, Suspended, TypeId};
use crate::support::retained::Growth;
impl Normalizer {
    pub(super) fn pay_queue(
        &self,
        count: usize,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let growth = Growth::vector::<(Key, bool)>(
            self.frame.fold.work.len(),
            self.frame.fold.work.capacity(),
            count,
        );
        charge(growth.work, growth.bytes)
    }
    pub(super) fn pay_complete(
        &self,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let growth = Growth::map::<Key, TypeId>(
            self.frame.fold.completed.len(),
            self.frame.fold.completed.capacity(),
            1,
        );
        charge(growth.work, growth.bytes)
    }
    pub(super) fn pay_active(
        &self,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let growth =
            Growth::map::<Key, ()>(self.frame.active.len(), self.frame.active.capacity(), 1);
        charge(growth.work, growth.bytes)
    }
    pub(super) fn pay_response(
        &self,
        resolution: Resolution,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        charge(1, 0)?;
        match resolution {
            Resolution::Opaque | Resolution::Deferred => self.pay_complete(charge),
            Resolution::Type(_) => {
                self.pay_queue(2, charge)?;
                let growth = Growth::map::<Key, Key>(
                    self.frame.redirects.len(),
                    self.frame.redirects.capacity(),
                    1,
                );
                charge(growth.work, growth.bytes)
            }
        }
    }
    pub(super) fn pay_node(
        &self,
        store: &Store,
        kind: Kind,
        finish: bool,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        charge(super::super::bounded::normalization_work(store, kind)?, 0)?;
        super::super::bounded::charge_prepare(&self.frame.fold, store, kind, finish, charge)
    }
    pub(super) fn pay_binder(
        &self,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let mut growth =
            Growth::vector::<(super::ScopeId, u32)>(self.binders.len(), self.binders.capacity(), 1);
        growth.include(Growth::vector::<Suspended>(
            self.suspended.len(),
            self.suspended.capacity(),
            1,
        ));
        charge(growth.work, growth.bytes)
    }
}
