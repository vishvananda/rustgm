//! Numeric fallback consumes one dependency-ready frontier, then yields to proof work.
use super::{Context, Dependency, Error, Fallback, State, Storage, Visit, Watch, WatchId};
use crate::support::retained::Growth;
use crate::types::infer::{Domain, TermId, Value};

impl Watch {
    fn numeric_ready(&self) -> bool {
        self.fallback.enabled() && self.walking == 0 && self.numeric != 0 && self.blockers == 0
    }
}
impl Storage {
    pub(super) fn disable_fallback(&mut self, id: WatchId) {
        let watch = &mut self.watches[id.0 as usize];
        if watch.fallback.enabled() {
            self.fallback_watches -= 1;
            watch.fallback = Fallback::Disabled;
        }
    }
    pub(super) fn prepay(context: &mut Context, growth: Growth) -> Result<(), Error> {
        // This owner has a work budget, not a byte counter. Charge copying and
        // rehashing before growth; report actual retained capacities separately.
        context.account_work(growth.work)
    }
    fn queue_numeric(&mut self, context: &mut Context, id: WatchId) -> Result<(), Error> {
        let watch = &self.watches[id.0 as usize];
        if watch.numeric_queued || !watch.numeric_ready() {
            return Ok(());
        }
        Self::prepay(
            context,
            Growth::vector::<WatchId>(self.numeric_ready.len(), self.numeric_ready.capacity(), 1),
        )?;
        self.watches[id.0 as usize].numeric_queued = true;
        self.numeric_ready.push_back(id);
        Ok(())
    }
    pub(super) fn refresh_numeric(
        &mut self,
        context: &mut Context,
        id: WatchId,
    ) -> Result<(), Error> {
        if !self.watches[id.0 as usize].fallback.enabled()
            || self.watches[id.0 as usize].walking != 0
        {
            return Ok(());
        }
        let open = self.watches[id.0 as usize].numeric_ready();
        let was_open = std::mem::replace(&mut self.watches[id.0 as usize].numeric_open, open);
        self.queue_numeric(context, id)?;
        if !open || was_open {
            return Ok(());
        }
        self.notify_numeric_peers(context, id)
    }
    fn notify_numeric_peers(&mut self, context: &mut Context, id: WatchId) -> Result<(), Error> {
        // A previously blocked shared numeric leaf may now be eligible. Only
        // this changed watch's actual frontier is visited, never all watches.
        let mut affected = Vec::new();
        for (&dependency, &visit) in &self.watches[id.0 as usize].seen {
            context.step()?;
            if visit != Visit::Numeric {
                continue;
            }
            if let Some(waiters) = self.waiting.get(&dependency) {
                Self::prepay(
                    context,
                    Growth::vector::<WatchId>(affected.len(), affected.capacity(), waiters.len()),
                )?;
                context.account_work(waiters.len())?;
                affected.extend_from_slice(waiters);
            }
        }
        for other in affected {
            self.queue_numeric(context, other)?;
        }
        Ok(())
    }
    pub(super) fn cancel(&mut self, context: &mut Context, id: WatchId) -> Result<(), Error> {
        let watch = self
            .watches
            .get(id.0 as usize)
            .ok_or(Error::InvalidIdentity)?;
        if watch.fallback.enabled() {
            self.notify_numeric_peers(context, id)?;
        }
        self.disable_fallback(id);
        let watch = &mut self.watches[id.0 as usize];
        watch.state = State::Cancelled;
        watch.seen = crate::support::hash::HashMap::default();
        Ok(())
    }
    pub(super) fn wait(
        &mut self,
        context: &mut Context,
        dependency: Dependency,
        id: WatchId,
    ) -> Result<(), Error> {
        let additional = usize::from(!self.waiting.contains_key(&dependency));
        Self::prepay(
            context,
            Growth::map::<Dependency, Vec<WatchId>>(
                self.waiting.len(),
                self.waiting.capacity(),
                additional,
            ),
        )?;
        let values = self.waiting.entry(dependency).or_default();
        Self::prepay(
            context,
            Growth::vector::<WatchId>(values.len(), values.capacity(), 1),
        )?;
        values.push(id);
        Ok(())
    }
    pub(super) fn record(&mut self, id: WatchId, dependency: Dependency, visit: Visit) {
        let watch = &mut self.watches[id.0 as usize];
        watch.seen.insert(dependency, visit);
        watch.numeric += usize::from(visit == Visit::Numeric);
        watch.blockers += usize::from(matches!(visit, Visit::Other | Visit::Normalization));
    }
    pub(super) fn wake(
        &mut self,
        context: &mut Context,
        dependency: Dependency,
    ) -> Result<(), Error> {
        let Some(watches) = self.waiting.remove(&dependency) else {
            return Ok(());
        };
        context.account_work(watches.len())?;
        for id in watches {
            let watch = &mut self.watches[id.0 as usize];
            if watch.state == State::Cancelled {
                continue;
            }
            let Some(visit) = watch.seen.remove(&dependency) else {
                continue;
            };
            watch.numeric -= usize::from(visit == Visit::Numeric);
            watch.blockers -= usize::from(matches!(visit, Visit::Other | Visit::Normalization));
            let visible = !visit.hidden();
            // Normalization barriers already completed their ordinary walk.
            if visit == Visit::Normalization {
                self.enqueue(context, id, dependency, visible)?;
            } else {
                Self::prepay(
                    context,
                    Growth::vector::<(WatchId, Dependency, bool)>(
                        self.work.len(),
                        self.work.capacity(),
                        1,
                    ),
                )?;
                watch.walking = watch.walking.checked_add(1).ok_or(Error::Overflow)?;
                self.work.push_back((id, dependency, visible));
            }
            if self.watches[id.0 as usize].changes {
                self.notify(context, id)?;
            }
        }
        Ok(())
    }
    pub(super) fn default_seed(&mut self, context: &mut Context) -> Result<bool, Error> {
        context.step()?;
        if self.has_pending_resolution() {
            return Ok(false);
        }
        while let Some(id) = self.numeric_ready.pop_front() {
            context.step()?;
            self.watches[id.0 as usize].numeric_queued = false;
            if !self.watches[id.0 as usize].numeric_ready() {
                continue;
            }
            let mut seeds = Vec::new();
            let mut blocked = false;
            for (&dependency, &visit) in &self.watches[id.0 as usize].seen {
                context.step()?;
                if visit != Visit::Numeric {
                    continue;
                }
                if let Some(waiters) = self.waiting.get(&dependency) {
                    for &other in waiters {
                        context.step()?;
                        let watch = &self.watches[other.0 as usize];
                        if watch.seen.get(&dependency) == Some(&Visit::Numeric)
                            && !watch.numeric_ready()
                        {
                            blocked = true;
                        }
                    }
                }
                let Dependency::Type(term) = dependency else {
                    return Err(Error::InvalidIdentity);
                };
                Self::prepay(
                    context,
                    Growth::vector::<TermId>(seeds.len(), seeds.capacity(), 1),
                )?;
                seeds.push(term);
            }
            if blocked {
                continue;
            }
            for term in seeds {
                let root = context.root(term)?;
                let primitive = match context.terms[root.0 as usize].value {
                    Value::Variable(Domain::Integer) => crate::types::Primitive::I32,
                    Value::Variable(Domain::Float) => crate::types::Primitive::F64,
                    _ => return Err(Error::InvalidIdentity),
                };
                context.terms[root.0 as usize].value =
                    Value::Known(crate::types::TypeId::primitive(primitive));
                self.wake(context, Dependency::Type(root))?;
            }
            return Ok(true);
        }
        Ok(false)
    }
}
impl Context {
    /// Observe a normalization's inputs, excluding only its own result root
    /// from input-normalization barriers. The ticket supplies no proof.
    pub(crate) fn watch_normalization(&mut self, input: TermId) -> Result<WatchId, Error> {
        self.watch_inputs(&[input], &[], true, Fallback::Projection(input))
    }
    pub(super) fn default_numeric_frontier(&mut self) -> Result<Option<bool>, Error> {
        if !self
            .dependencies
            .as_ref()
            .is_some_and(|s| s.fallback_watches != 0)
        {
            return Ok(None);
        }
        let Some(mut storage) = self.dependencies.take() else {
            return Err(Error::InvalidIdentity);
        };
        let result = storage.default_seed(self).map(Some);
        self.dependencies = Some(storage);
        result
    }
}
