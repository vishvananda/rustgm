//! Route shared inference notifications to body projections or retained trait goals.
#[cfg(test)]
#[path = "../../../tests/invariants/notification_routing.rs"]
mod tests;
use crate::types::{
    Store,
    infer::{Context, Error, dependencies::WatchId, predicates::Predicate},
};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Default)]
struct Data {
    by_watch: HashMap<WatchId, usize>,
    by_goal: HashMap<usize, WatchId>,
    goals: VecDeque<usize>,
    queued: HashSet<usize>,
    projections: VecDeque<WatchId>,
}
#[derive(Default)]
pub(super) struct Notifications {
    data: Option<Box<Data>>,
}
impl Notifications {
    pub fn waiting(&self) -> bool {
        self.data
            .as_ref()
            .is_some_and(|data| !data.by_goal.is_empty())
    }
    pub fn first_pending(&self) -> Option<usize> {
        self.data.as_ref()?.by_goal.keys().min().copied()
    }
    pub fn next_goal(&mut self, context: &mut Context) -> Result<Option<usize>, Error> {
        let Some(data) = &mut self.data else {
            return Ok(None);
        };
        while let Some(index) = data.goals.pop_front() {
            context.account_work(1)?;
            if data.queued.remove(&index) && data.by_goal.contains_key(&index) {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
    pub fn observe(
        &mut self,
        context: &mut Context,
        store: &Store,
        index: usize,
        goal: &Predicate,
    ) -> Result<(), Error> {
        let data = self.data.get_or_insert_with(Default::default);
        if let std::collections::hash_map::Entry::Vacant(entry) = data.by_goal.entry(index) {
            let watch = context.watch_predicate(goal)?;
            entry.insert(watch);
            data.by_watch.insert(watch, index);
        }
        while let Some(watch) = context.ready_resolution(store)? {
            context.account_work(1)?;
            if let Some(&owner) = data.by_watch.get(&watch) {
                if owner != index && data.queued.insert(owner) {
                    data.goals.push_back(owner);
                }
            } else {
                data.projections.push_back(watch);
            }
        }
        Ok(())
    }
    pub fn complete(&mut self, context: &mut Context, index: usize) -> Result<(), Error> {
        if let Some(data) = &mut self.data
            && let Some(watch) = data.by_goal.remove(&index)
        {
            data.by_watch.remove(&watch);
            data.queued.remove(&index);
            context.cancel_watch(watch)?;
        }
        Ok(())
    }
    pub fn next_projection(
        &mut self,
        context: &mut Context,
        store: &Store,
    ) -> Result<Option<WatchId>, Error> {
        if let Some(watch) = self
            .data
            .as_mut()
            .and_then(|data| data.projections.pop_front())
        {
            return Ok(Some(watch));
        }
        while let Some(watch) = context.ready_resolution(store)? {
            let Some(data) = &mut self.data else {
                return Ok(Some(watch));
            };
            let Some(&index) = data.by_watch.get(&watch) else {
                return Ok(Some(watch));
            };
            context.account_work(1)?;
            if data.queued.insert(index) {
                data.goals.push_back(index);
            }
        }
        Ok(None)
    }
}
