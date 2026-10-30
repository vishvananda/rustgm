//! Each suspended body goal owns its selected child's silent fallback tickets.
use super::{Context, Error, HashMap, Notifications, Store, WatchId};
use crate::{support::retained::Growth, types::infer::TermId};

#[derive(Default)]
pub(super) struct Hints {
    goals: HashMap<Owner, Vec<WatchId>>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Owner {
    Goal(usize),
    Projection(WatchId),
}
impl Notifications {
    /// Remove child preferences before selection resumes, changes or completes.
    pub fn clear_hints(&mut self, context: &mut Context, index: usize) -> Result<(), Error> {
        self.clear_owned_hints(context, Owner::Goal(index))
    }
    pub fn clear_projection_hints(
        &mut self,
        context: &mut Context,
        watch: WatchId,
    ) -> Result<(), Error> {
        self.clear_owned_hints(context, Owner::Projection(watch))
    }
    fn clear_owned_hints(&mut self, context: &mut Context, owner: Owner) -> Result<(), Error> {
        let Some(watches) = self
            .data
            .as_mut()
            .and_then(|data| data.hints.as_mut())
            .and_then(|hints| hints.goals.remove(&owner))
        else {
            return Ok(());
        };
        for watch in watches {
            context.cancel_watch(watch)?;
        }
        Ok(())
    }

    /// A mapped hint lends a preference to the real goal's visible frontier.
    /// It neither selects that goal nor schedules its own fallback pass.
    pub fn prefer_floats(
        &mut self,
        context: &mut Context,
        store: &Store,
        index: usize,
        inputs: &[TermId],
    ) -> Result<(), Error> {
        let parent = self
            .data
            .as_ref()
            .and_then(|data| data.by_goal.get(&index))
            .copied();
        if inputs.is_empty() {
            return self.clear_hints(context, index);
        }
        self.prefer_owned_floats(
            context,
            store,
            Owner::Goal(index),
            parent.ok_or(Error::InvalidIdentity)?,
            inputs,
        )
    }
    pub fn prefer_projection_floats(
        &mut self,
        context: &mut Context,
        store: &Store,
        watch: WatchId,
        inputs: &[TermId],
    ) -> Result<(), Error> {
        self.prefer_owned_floats(context, store, Owner::Projection(watch), watch, inputs)
    }
    fn prefer_owned_floats(
        &mut self,
        context: &mut Context,
        store: &Store,
        owner: Owner,
        parent: WatchId,
        inputs: &[TermId],
    ) -> Result<(), Error> {
        self.clear_owned_hints(context, owner)?;
        if inputs.is_empty() {
            return Ok(());
        }
        context.account_work(1)?;
        let data = self.data.get_or_insert_with(Default::default);
        // Publication may expose fresh input leaves. Observe that real parent
        // frontier before deciding whether any mapped child root is visible.
        data.poll(context, store, owner)?;
        let (length, capacity) = data
            .hints
            .as_ref()
            .map_or((0, 0), |hints| (hints.goals.len(), hints.goals.capacity()));
        let growth = Growth::map::<Owner, Vec<WatchId>>(length, capacity, 1);
        context.account_work(inputs.len().saturating_add(growth.work).saturating_add(1))?;
        let mut watches = Vec::with_capacity(inputs.len());
        for &input in inputs {
            if let Some(watch) = context.watch_float_hint(parent, input)? {
                watches.push(watch);
            }
        }
        if !watches.is_empty() {
            data.hints
                .get_or_insert_with(Default::default)
                .goals
                .insert(owner, watches);
        }
        data.poll(context, store, owner)
    }
}
