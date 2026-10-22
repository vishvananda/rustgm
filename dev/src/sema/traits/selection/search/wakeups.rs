//! Retry a condition only after one of its own type or const inputs changes.
use super::{Declarations, Trial, inference};
use crate::support::hash::{HashMap, HashSet};
use crate::types::infer::dependencies::WatchId;

#[derive(Default)]
struct Data {
    children: HashMap<usize, usize>,
    by_watch: HashMap<WatchId, usize>,
    by_goal: HashMap<usize, WatchId>,
    queued: HashSet<usize>,
}
#[derive(Default)]
pub(super) struct Wakeups {
    data: Option<Box<Data>>,
}
impl Wakeups {
    pub fn waiting(&self) -> bool {
        self.data
            .as_ref()
            .is_some_and(|data| !data.by_goal.is_empty())
    }
    pub fn start(&mut self, index: usize) {
        if let Some(data) = &mut self.data {
            data.queued.remove(&index);
        }
    }
    pub fn take_child(&mut self, index: usize) -> Option<usize> {
        self.data.as_mut()?.children.remove(&index)
    }
    pub fn retain_child(&mut self, index: usize, slot: usize) {
        self.data
            .get_or_insert_with(Default::default)
            .children
            .insert(index, slot);
    }
}
impl Trial {
    pub(super) fn defer_condition(
        &mut self,
        facts: &Declarations,
        index: usize,
    ) -> Result<(), String> {
        let data = self.wakeups.data.get_or_insert_with(Default::default);
        if let std::collections::hash_map::Entry::Vacant(entry) = data.by_goal.entry(index) {
            let goal = self
                .instance
                .goals
                .get(index)
                .ok_or("missing deferred condition")?;
            let watch = self
                .inputs
                .context
                .watch_predicate(goal)
                .map_err(inference)?;
            entry.insert(watch);
            data.by_watch.insert(watch, index);
        }
        // This condition has consumed its current inputs, including any progress
        // it just published. Preserve notifications for its other dependencies.
        self.poll_condition_changes(facts, Some(index))
    }

    pub(super) fn complete_condition(&mut self, index: usize) -> Result<(), String> {
        if let Some(data) = &mut self.wakeups.data
            && let Some(watch) = data.by_goal.remove(&index)
        {
            data.by_watch.remove(&watch);
            self.inputs.context.cancel_watch(watch).map_err(inference)?;
        }
        Ok(())
    }

    pub(super) fn poll_condition_changes(
        &mut self,
        facts: &Declarations,
        consumed: Option<usize>,
    ) -> Result<(), String> {
        let Some(data) = &mut self.wakeups.data else {
            return Ok(());
        };
        while let Some(watch) = self
            .inputs
            .context
            .ready_resolution(&facts.types)
            .map_err(inference)?
        {
            let index = *data.by_watch.get(&watch).ok_or("missing condition watch")?;
            if consumed != Some(index) && data.queued.insert(index) {
                self.pending.push(index);
            }
        }
        Ok(())
    }
}
