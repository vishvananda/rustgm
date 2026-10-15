//! Resume canonical-input queries only when one of their unresolved leaves changes.
mod walk;
use super::{Context, Error, Store, TermId, constants::ConstTermId};
use std::collections::{HashMap, HashSet, VecDeque};

/// Identifies one readiness query in its original inference context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WatchId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Dependency {
    Type(TermId),
    Const(ConstTermId),
}

#[derive(Default)]
struct Watch {
    remaining: usize,
    seen: HashSet<Dependency>,
}

#[derive(Default)]
pub(super) struct Storage {
    watches: Vec<Watch>,
    waiting: HashMap<Dependency, Vec<WatchId>>,
    work: VecDeque<(WatchId, Dependency)>,
    ready: VecDeque<WatchId>,
}
impl Storage {
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
        storage.watches.push(Watch::default());
        for dependency in types
            .iter()
            .copied()
            .map(Dependency::Type)
            .chain(constants.iter().copied().map(Dependency::Const))
        {
            storage.enqueue(id, dependency)?;
        }
        if types.is_empty() && constants.is_empty() {
            storage.ready.push_back(id);
        }
        Ok(id)
    }

    /// Return each ready query once, traversing only newly supplied or awakened work.
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
            storage.watches[id.0 as usize].seen.remove(&dependency);
            storage.work.push_back((id, dependency));
        }
        self.account_work(count)
    }
}
