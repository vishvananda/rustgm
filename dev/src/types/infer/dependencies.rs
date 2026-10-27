//! Resume input queries only when one of their unresolved leaves changes.
mod numeric;
#[cfg(test)]
#[path = "../../../tests/invariants/numeric_frontiers.rs"]
mod numeric_tests;
mod walk;
use super::{Context, Error, Store, TermId, constants::ConstTermId};
use crate::support::hash::HashMap;
use std::collections::VecDeque;

/// Identifies one readiness query in its original inference context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WatchId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Dependency {
    Type(TermId),
    Const(ConstTermId),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Initial,
    Observing,
    Queued,
    Resolved,
    Cancelled,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Visit {
    HiddenDone,
    HiddenNumeric,
    HiddenOther,
    Done,
    Numeric,
    Other,
    Normalization,
}
impl Visit {
    fn hidden(self) -> bool {
        matches!(
            self,
            Self::HiddenDone | Self::HiddenNumeric | Self::HiddenOther
        )
    }
}
#[derive(Clone, Copy)]
enum Fallback {
    Disabled,
    Predicate,
    Projection(TermId),
}
impl Fallback {
    fn enabled(self) -> bool {
        !matches!(self, Self::Disabled)
    }
}
struct Watch {
    remaining: usize,
    seen: HashMap<Dependency, Visit>,
    fallback: Fallback,
    numeric: usize,
    blockers: usize,
    walking: usize,
    numeric_queued: bool,
    numeric_open: bool,
    changes: bool,
    state: State,
}

#[derive(Default)]
pub(super) struct Storage {
    pub(super) root_changes: Option<Vec<(TermId, TermId)>>,
    pub(super) equations: super::deferred::Storage,
    watches: Vec<Watch>,
    waiting: HashMap<Dependency, Vec<WatchId>>,
    work: VecDeque<(WatchId, Dependency, bool)>,
    numeric_ready: VecDeque<WatchId>,
    fallback_watches: usize,
    ready: VecDeque<WatchId>,
    changed: VecDeque<WatchId>,
}
impl Storage {
    fn has_pending_resolution(&self) -> bool {
        !self.ready.is_empty() || !self.work.is_empty() || !self.changed.is_empty()
    }
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        if let Some(changes) = &self.root_changes {
            total.vector(changes);
        }
        total.include(self.equations.allocation());
        total.vector(&self.watches);
        for watch in &self.watches {
            total.map(&watch.seen);
        }
        total.map(&self.waiting);
        for waiting in self.waiting.values() {
            total.vector(waiting);
        }
        total.deque(&self.work);
        total.deque(&self.numeric_ready);
        total.deque(&self.ready);
        total.deque(&self.changed);
        total
    }

    fn notify(&mut self, context: &mut Context, id: WatchId) -> Result<(), Error> {
        let watch = &mut self.watches[id.0 as usize];
        if matches!(watch.state, State::Initial | State::Observing) {
            Self::prepay(
                context,
                crate::support::retained::Growth::vector::<WatchId>(
                    self.changed.len(),
                    self.changed.capacity(),
                    1,
                ),
            )?;
            watch.state = State::Queued;
            self.changed.push_back(id);
        }
        Ok(())
    }
    fn enqueue(
        &mut self,
        context: &mut Context,
        id: WatchId,
        dependency: Dependency,
        visible: bool,
    ) -> Result<(), Error> {
        Self::prepay(
            context,
            crate::support::retained::Growth::vector::<(WatchId, Dependency, bool)>(
                self.work.len(),
                self.work.capacity(),
                1,
            ),
        )?;
        let watch = &mut self.watches[id.0 as usize];
        watch.remaining = watch.remaining.checked_add(1).ok_or(Error::Overflow)?;
        watch.walking = watch.walking.checked_add(1).ok_or(Error::Overflow)?;
        self.work.push_back((id, dependency, visible));
        Ok(())
    }

    fn register(
        &mut self,
        context: &mut Context,
        types: &[TermId],
        constants: &[ConstTermId],
        changes: bool,
        fallback: Fallback,
    ) -> Result<WatchId, Error> {
        Self::prepay(
            context,
            crate::support::retained::Growth::vector::<Watch>(
                self.watches.len(),
                self.watches.capacity(),
                1,
            ),
        )?;
        let id = WatchId(u32::try_from(self.watches.len()).map_err(|_| Error::Overflow)?);
        self.watches.push(Watch {
            remaining: 0,
            seen: HashMap::default(),
            fallback,
            numeric: 0,
            blockers: 0,
            walking: 0,
            numeric_queued: false,
            numeric_open: false,
            changes,
            state: State::Initial,
        });
        self.fallback_watches += usize::from(fallback.enabled());
        for dependency in types
            .iter()
            .copied()
            .map(Dependency::Type)
            .chain(constants.iter().copied().map(Dependency::Const))
        {
            self.enqueue(context, id, dependency, fallback.enabled())?;
        }
        if types.is_empty() && constants.is_empty() {
            self.disable_fallback(id);
            self.watches[id.0 as usize].state = State::Resolved;
            self.ready.push_back(id);
        }
        Ok(id)
    }

    fn complete(&mut self, context: &mut Context, id: WatchId) -> Result<(), Error> {
        let watch = &mut self.watches[id.0 as usize];
        watch.remaining = watch
            .remaining
            .checked_sub(1)
            .ok_or(Error::InvalidIdentity)?;
        if watch.remaining == 0 {
            Self::prepay(
                context,
                crate::support::retained::Growth::vector::<WatchId>(
                    self.ready.len(),
                    self.ready.capacity(),
                    1,
                ),
            )?;
            self.disable_fallback(id);
            let watch = &mut self.watches[id.0 as usize];
            watch.seen = HashMap::default();
            watch.state = State::Resolved;
            self.ready.push_back(id);
        }
        Ok(())
    }
}

impl Context {
    /// Watch a set of shared type/const inputs without constraining or defaulting them.
    /// Readiness means no type/const holes remain. Canonicalization, associated
    /// projection normalization and their proofs are separate caller operations.
    /// The caller retains the source, proof scope and original input terms.
    ///
    /// # Errors
    /// Rejects invalid graph identities, compact overflow and exhausted work.
    pub fn watch_resolution(
        &mut self,
        types: &[TermId],
        constants: &[ConstTermId],
    ) -> Result<WatchId, Error> {
        self.watch_inputs(types, constants, false, Fallback::Disabled)
    }

    /// Observe initial partial inputs and later changes to only their dependency leaves.
    /// A notification carries no proof and does not imply complete resolution.
    /// Keep the ticket until selection completes, then cancel it explicitly.
    /// # Errors
    /// Rejects invalid graph identities, compact overflow and exhausted work.
    pub fn watch_changes(
        &mut self,
        types: &[TermId],
        constants: &[ConstTermId],
    ) -> Result<WatchId, Error> {
        self.watch_inputs(types, constants, true, Fallback::Disabled)
    }

    /// Observe every type/const input of a complete predicate, including equality results.
    /// Lifetime identities remain proof obligations, not inference dependencies.
    /// # Errors
    /// Rejects malformed inputs and exhausted work, as `Self::watch_changes` does.
    pub fn watch_predicate(
        &mut self,
        predicate: &super::predicates::Predicate,
    ) -> Result<WatchId, Error> {
        let mut types = Vec::new();
        let mut constants = Vec::new();
        for argument in predicate.arguments() {
            match argument {
                super::arguments::Argument::Type(term) => types.push(term),
                super::arguments::Argument::Const(term) => constants.push(term),
                super::arguments::Argument::Lifetime(_) => {}
            }
        }
        self.watch_inputs(&types, &constants, true, Fallback::Predicate)
    }

    fn watch_inputs(
        &mut self,
        types: &[TermId],
        constants: &[ConstTermId],
        changes: bool,
        fallback: Fallback,
    ) -> Result<WatchId, Error> {
        self.account_work(
            types
                .len()
                .saturating_add(constants.len())
                .saturating_add(1),
        )?;
        for &term in types {
            self.root(term)?;
        }
        for &term in constants {
            self.constants().ty(term)?;
        }
        let mut storage = self.dependencies.take().unwrap_or_default();
        let result = storage.register(self, types, constants, changes, fallback);
        self.dependencies = Some(storage);
        result
    }

    /// Stop a query after its owner consumes the result; queued events become inert.
    /// # Errors
    /// Rejects an invalid watch identity and exhausted work.
    pub fn cancel_watch(&mut self, id: WatchId) -> Result<(), Error> {
        self.step()?;
        let Some(mut storage) = self.dependencies.take() else {
            return Err(Error::InvalidIdentity);
        };
        let result = storage.cancel(self, id);
        self.dependencies = Some(storage);
        result?;
        Ok(())
    }

    /// Revisit an observed query after its owner publishes a required normal form.
    /// This is a notification only: the original input counters remain intact.
    /// Cancelled/completed tickets are inert, and repeated requests coalesce.
    /// # Errors
    /// Rejects invalid tickets and exhausted inference work.
    pub fn retry_watch(&mut self, id: WatchId) -> Result<(), Error> {
        self.step()?;
        let Some(mut storage) = self.dependencies.take() else {
            return Err(Error::InvalidIdentity);
        };
        let result = if let Some(watch) = storage.watches.get(id.0 as usize) {
            if watch.remaining != 0 {
                storage.notify(self, id)
            } else {
                Ok(())
            }
        } else {
            Err(Error::InvalidIdentity)
        };
        self.dependencies = Some(storage);
        result?;
        Ok(())
    }

    /// Whether the incremental walk has proved all inputs free of type/const holes.
    /// A change notification may arrive before this proof is complete.
    /// # Errors
    /// Rejects invalid or cancelled tickets and exhausted work.
    pub fn watch_resolved(&mut self, id: WatchId) -> Result<bool, Error> {
        self.step()?;
        let watch = self
            .dependencies
            .as_ref()
            .and_then(|s| s.watches.get(id.0 as usize))
            .filter(|watch| watch.state != State::Cancelled)
            .ok_or(Error::InvalidIdentity)?;
        Ok(watch.remaining == 0)
    }

    /// After consuming a partial notification, wait only for canonical inputs.
    /// The owner must establish that further partial changes cannot complete its query.
    /// # Errors
    /// Rejects tickets with an unconsumed notification, cancelled tickets and exhausted work.
    pub fn await_resolution(&mut self, id: WatchId) -> Result<(), Error> {
        self.step()?;
        let watch = self
            .dependencies
            .as_mut()
            .and_then(|s| s.watches.get_mut(id.0 as usize))
            .filter(|watch| watch.state == State::Observing)
            .ok_or(Error::InvalidIdentity)?;
        watch.changes = false;
        Ok(())
    }

    /// Whether `ready_resolution` or `ready_equation` could return anything:
    /// queued readiness, dependency work, partial changes or woken equations.
    #[inline]
    #[must_use]
    pub fn has_pending_notifications(&self) -> bool {
        self.dependencies
            .as_ref()
            .is_some_and(|s| s.has_pending_resolution() || s.equations.has_ready())
    }

    /// Return a ready notification, traversing only newly supplied or awakened work.
    /// Resolution tickets notify once; change tickets can notify repeatedly.
    /// Variables newly bound to compound types expose their unresolved children;
    /// unrelated equations never cause a scan of all pending queries.
    ///
    /// # Errors
    /// Rejects malformed graphs and exhausted inference resources.
    pub fn ready_resolution(&mut self, store: &Store) -> Result<Option<WatchId>, Error> {
        if !self
            .dependencies
            .as_ref()
            .is_some_and(|s| s.has_pending_resolution())
        {
            return Ok(None);
        }
        let Some(mut storage) = self.dependencies.take() else {
            return Ok(None);
        };
        let result = storage.poll(self, store);
        self.dependencies = Some(storage);
        result
    }

    pub(super) fn wake_resolution(&mut self, dependency: Dependency) -> Result<(), Error> {
        let Some(mut storage) = self.dependencies.take() else {
            return Ok(());
        };
        let result = storage.wake(self, dependency);
        self.dependencies = Some(storage);
        result
    }
}
