//! Rollback visits only exact keys newly written by the failing active scope.
use super::{InlineInstance, Registry};
use crate::support::hash::HashMap;
use crate::{sema::values::Constant, types::TypeId};
type Key = (InlineInstance, TypeId);
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Lease(u64);
#[derive(Default)]
pub(super) struct Journals {
    next: u64,
    rows: Vec<Row>,
}
struct Row {
    lease: Lease,
    keys: HashMap<Key, Entry>,
    bytes: usize,
}
struct Entry {
    prior: Option<Constant>,
    written: Option<Constant>,
}
impl Journals {
    pub(super) fn record(&mut self, key: Key, value: Constant) {
        if let Some(entry) = self.rows.last_mut().and_then(|row| row.keys.get_mut(&key))
            && entry.prior.is_none()
        {
            entry.written = Some(value);
        }
    }
}
impl Registry {
    pub(crate) fn current_journal(&self, lease: Lease) -> bool {
        self.instances
            .as_deref()
            .and_then(|values| values.journals.as_deref())
            .and_then(|journals| journals.rows.last())
            .is_some_and(|row| row.lease == lease)
    }

    pub(crate) fn begin_journal(
        &mut self,
        keys: impl ExactSizeIterator<Item = Key>,
    ) -> Result<Lease, &'static str> {
        let count = keys.len();
        let old = self
            .instances
            .as_deref()
            .and_then(|values| values.journals.as_deref());
        if old.is_some_and(|journals| journals.rows.len() >= 128) {
            return Err("conditional value scopes exceed depth budget");
        }
        let header = usize::from(self.instances.is_none()) * size_of::<super::instances::Values>()
            + usize::from(old.is_none()) * size_of::<Journals>();
        let growth = if old.is_none_or(|journals| journals.rows.len() == journals.rows.capacity()) {
            old.map_or(4, |journals| journals.rows.capacity().max(4)) * size_of::<Row>()
        } else {
            0
        };
        let bytes = crate::support::retained::new_map::<Key, Entry>(count);
        if self
            .retained_bytes
            .saturating_add(bytes)
            .saturating_add(header)
            .saturating_add(growth)
            > 64 * 1024 * 1024
            || count > 1_000_000
        {
            return Err("conditional value journal exceeds storage budget");
        }
        self.retained_bytes = self
            .retained_bytes
            .saturating_add(header)
            .saturating_add(growth);
        let values = self.instances.get_or_insert_with(Default::default);
        let entries: HashMap<_, _> = keys
            .map(|key| {
                (
                    key,
                    Entry {
                        prior: values.completed.get(&key).copied(),
                        written: None,
                    },
                )
            })
            .collect();
        let journals = values.journals.get_or_insert_with(Default::default);
        if journals.rows.len() >= 128 {
            return Err("conditional value scopes exceed depth budget");
        }
        journals.next = journals
            .next
            .checked_add(1)
            .ok_or("conditional value scope identity exhausted")?;
        let lease = Lease(journals.next);
        self.preparation_visits = self.preparation_visits.saturating_add(entries.len());
        journals.rows.push(Row {
            lease,
            keys: entries,
            bytes,
        });
        self.retained_bytes = self.retained_bytes.saturating_add(bytes);
        Ok(lease)
    }
    pub(crate) fn finish_journal(
        &mut self,
        lease: Lease,
        commit: bool,
    ) -> Result<(), &'static str> {
        let values = self
            .instances
            .as_mut()
            .ok_or("conditional values disappeared")?;
        let journals = values
            .journals
            .as_mut()
            .ok_or("conditional value journal disappeared")?;
        if !journals.rows.last().is_some_and(|row| row.lease == lease) {
            return Err("conditional value retirement has another owner");
        }
        let row = journals
            .rows
            .pop()
            .ok_or("conditional value journal disappeared")?;
        self.retained_bytes = self.retained_bytes.saturating_sub(row.bytes);
        let mut outcome = Ok(());
        for (key, entry) in row.keys {
            self.preparation_visits = self.preparation_visits.saturating_add(1);
            if !commit && let Some(written) = entry.written {
                if values.completed.get(&key) == Some(&written) {
                    values.completed.remove(&key);
                } else {
                    outcome = Err("conditional rollback found a foreign value publication");
                }
            }
        }
        outcome
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/conditional_journals.rs"]
mod tests;
