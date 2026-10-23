//! Per-entry context histories: direct newest lookup and logarithmic predecessors.
use super::{Binding, ContextId, Local};
use crate::support::hash::HashMap;

/// The newest local is inline. Most contexts never shadow and allocate no vector.
struct History {
    latest: u32,
    older: Vec<u32>,
}
#[derive(Default)]
pub(super) struct Index {
    contexts: HashMap<(u32, ContextId), History>,
    #[cfg(test)]
    comparisons: std::cell::Cell<usize>,
}
impl Index {
    pub fn contains(&self, entry: u32, context: ContextId) -> bool {
        self.contexts.contains_key(&(entry, context))
    }
    /// BindingTable checks increasing declaration identities before publication.
    pub fn push(&mut self, entry: u32, context: ContextId, local: u32) {
        match self.contexts.entry((entry, context)) {
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(History {
                    latest: local,
                    older: Vec::new(),
                });
            }
            std::collections::hash_map::Entry::Occupied(mut slot) => {
                let history = slot.get_mut();
                history
                    .older
                    .push(std::mem::replace(&mut history.latest, local));
            }
        }
    }
    pub fn get(
        &self,
        entry: u32,
        context: ContextId,
        visible: usize,
        locals: &[Local],
    ) -> Option<Binding> {
        let history = self.contexts.get(&(entry, context))?;
        let before = |binding: Binding| {
            #[cfg(test)]
            self.comparisons.set(self.comparisons.get() + 1);
            (binding.target.0 as usize) < visible
        };
        let latest = locals[history.latest as usize].binding;
        if before(latest) {
            return Some(latest);
        }
        let end = history
            .older
            .partition_point(|id| before(locals[*id as usize].binding));
        let id = history.older.get(end.checked_sub(1)?)?;
        Some(locals[*id as usize].binding)
    }
    #[cfg(test)]
    pub fn take_comparisons(&self) -> usize {
        self.comparisons.replace(0)
    }
}
