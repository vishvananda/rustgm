//! Resume input queries only when one of their unresolved leaves changes.
mod walk;
use super::{Context, Error, Store, TermId, constants::ConstTermId};
use std::collections::{HashMap, HashSet, VecDeque};

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
struct Watch {
    remaining: usize,
    seen: HashSet<Dependency>,
    changes: bool,
    state: State,
}

#[derive(Default)]
pub(super) struct Storage {
    pub(super) equations: super::deferred::Storage,
    watches: Vec<Watch>,
    waiting: HashMap<Dependency, Vec<WatchId>>,
    work: VecDeque<(WatchId, Dependency)>,
    ready: VecDeque<WatchId>,
    changed: VecDeque<WatchId>,
}
impl Storage {
    fn notify(&mut self, id: WatchId) {
        let watch = &mut self.watches[id.0 as usize];
        if matches!(watch.state, State::Initial | State::Observing) {
            watch.state = State::Queued;
            self.changed.push_back(id);
        }
    }
    fn enqueue(&mut self, id: WatchId, dependency: Dependency) -> Result<(), Error> {
        let watch = &mut self.watches[id.0 as usize];
        watch.remaining = watch.remaining.checked_add(1).ok_or(Error::Overflow)?;
        self.work.push_back((id, dependency));
        Ok(())
    }

    fn complete(&mut self, id: WatchId) -> Result<(), Error> {
        let watch = &mut self.watches[id.0 as usize];
        watch.remaining = watch
            .remaining
            .checked_sub(1)
            .ok_or(Error::InvalidIdentity)?;
        if watch.remaining == 0 {
            watch.seen = HashSet::new();
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
        self.watch_inputs(types, constants, false)
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
        self.watch_inputs(types, constants, true)
    }

    fn watch_inputs(
        &mut self,
        types: &[TermId],
        constants: &[ConstTermId],
        changes: bool,
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
        let storage = self.dependencies.get_or_insert_with(Default::default);
        let id = WatchId(u32::try_from(storage.watches.len()).map_err(|_| Error::Overflow)?);
        storage.watches.push(Watch {
            remaining: 0,
            seen: HashSet::new(),
            changes,
            state: State::Initial,
        });
        for dependency in types
            .iter()
            .copied()
            .map(Dependency::Type)
            .chain(constants.iter().copied().map(Dependency::Const))
        {
            storage.enqueue(id, dependency)?;
        }
        if types.is_empty() && constants.is_empty() {
            storage.watches[id.0 as usize].state = State::Resolved;
            storage.ready.push_back(id);
        }
        Ok(id)
    }

    /// Stop a query after its owner consumes the result; queued events become inert.
    /// # Errors
    /// Rejects an invalid watch identity and exhausted work.
    pub fn cancel_watch(&mut self, id: WatchId) -> Result<(), Error> {
        self.step()?;
        let watch = self
            .dependencies
            .as_mut()
            .and_then(|s| s.watches.get_mut(id.0 as usize))
            .ok_or(Error::InvalidIdentity)?;
        watch.state = State::Cancelled;
        watch.seen = HashSet::new();
        Ok(())
    }

    /// Revisit an observed query after its owner publishes a required normal form.
    /// This is a notification only: the original input counters remain intact.
    /// Cancelled/completed tickets are inert, and repeated requests coalesce.
    /// # Errors
    /// Rejects invalid tickets and exhausted inference work.
    pub fn retry_watch(&mut self, id: WatchId) -> Result<(), Error> {
        self.step()?;
        let storage = self.dependencies.as_mut().ok_or(Error::InvalidIdentity)?;
        let watch = storage
            .watches
            .get(id.0 as usize)
            .ok_or(Error::InvalidIdentity)?;
        if watch.remaining != 0 {
            storage.notify(id);
        }
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

    /// Return a ready notification, traversing only newly supplied or awakened work.
    /// Resolution tickets notify once; change tickets can notify repeatedly.
    /// Variables newly bound to compound types expose their unresolved children;
    /// unrelated equations never cause a scan of all pending queries.
    ///
    /// # Errors
    /// Rejects malformed graphs and exhausted inference resources.
    pub fn ready_resolution(&mut self, store: &Store) -> Result<Option<WatchId>, Error> {
        let Some(mut storage) = self.dependencies.take() else {
            return Ok(None);
        };
        let result = storage.poll(self, store);
        self.dependencies = Some(storage);
        result
    }

    pub(super) fn wake_resolution(&mut self, dependency: Dependency) -> Result<(), Error> {
        let Some(storage) = &mut self.dependencies else {
            return Ok(());
        };
        let Some(watches) = storage.waiting.remove(&dependency) else {
            return Ok(());
        };
        let count = watches.len();
        for id in watches {
            let watch = &mut storage.watches[id.0 as usize];
            if watch.state == State::Cancelled {
                continue;
            }
            watch.seen.remove(&dependency);
            if watch.changes {
                storage.notify(id);
            }
            storage.work.push_back((id, dependency));
        }
        self.account_work(count)
    }
}
