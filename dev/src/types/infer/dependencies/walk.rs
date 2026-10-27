//! Discover dependency leaves, including const slots that are not type edges.
use super::{Dependency, State, Storage, Visit, WatchId};
use crate::types::infer::{Constructor, Context, Domain, Error, Store, Value, arguments::Slot};

impl Storage {
    pub(super) fn poll(
        &mut self,
        context: &mut Context,
        store: &Store,
    ) -> Result<Option<WatchId>, Error> {
        loop {
            if let Some(id) = self.ready.pop_front() {
                let watch = &mut self.watches[id.0 as usize];
                if watch.state == State::Cancelled {
                    context.step()?;
                    continue;
                }
                watch.state = State::Observing;
                return Ok(Some(id));
            }
            let Some((id, dependency, visible)) = self.work.pop_front() else {
                break;
            };
            context.step()?;
            if self.watches[id.0 as usize].state == State::Cancelled {
                continue;
            }
            let dependency = match dependency {
                Dependency::Type(term) => Dependency::Type(context.root(term)?),
                Dependency::Const(term) => Dependency::Const(context.constants().root(term)?),
            };
            self.watches[id.0 as usize].walking -= 1;
            let previous = self.watches[id.0 as usize].seen.get(&dependency).copied();
            if previous.is_some_and(|visit| !visible || !visit.hidden()) {
                self.complete(context, id)?;
            } else {
                Self::prepay(
                    context,
                    crate::support::retained::Growth::map::<Dependency, Visit>(
                        self.watches[id.0 as usize].seen.len(),
                        self.watches[id.0 as usize].seen.capacity(),
                        usize::from(previous.is_none()),
                    ),
                )?;
                let done = self.visit(context, store, id, dependency, visible)?;
                if done || previous.is_some() {
                    self.complete(context, id)?;
                } else {
                    self.wait(context, dependency, id)?;
                    let watch = &self.watches[id.0 as usize];
                    if watch.changes && watch.state == State::Initial {
                        self.notify(context, id)?;
                    }
                }
            }
            self.refresh_numeric(context, id)?;
        }
        // Keep canonical readiness in its original completion order. Deliver
        // partial notifications only after all pending dependency updates.
        while let Some(id) = self.changed.pop_front() {
            let watch = &mut self.watches[id.0 as usize];
            if watch.state != State::Queued {
                context.step()?;
                continue;
            }
            watch.state = State::Observing;
            return Ok(Some(id));
        }
        Ok(None)
    }

    fn visit(
        &mut self,
        context: &mut Context,
        store: &Store,
        id: WatchId,
        dependency: Dependency,
        visible: bool,
    ) -> Result<bool, Error> {
        let visible = visible && self.watches[id.0 as usize].fallback.enabled();
        let visit = self.classify(context, store, id, dependency, visible)?;
        self.record(id, dependency, visit);
        if matches!(
            visit,
            Visit::Numeric | Visit::HiddenNumeric | Visit::Other | Visit::HiddenOther
        ) {
            return Ok(false);
        }
        if visit == Visit::Normalization {
            self.wait(context, dependency, id)?;
        }
        let Dependency::Type(root) = dependency else {
            return Ok(true);
        };
        let term = context.terms[root.0 as usize];
        let kind = match term.value {
            Value::Construct(kind) => kind,
            Value::Known(ty) => {
                store.kind(ty).ok_or(Error::InvalidIdentity)?;
                return Ok(true);
            }
            Value::Variable(_) => return Err(Error::InvalidIdentity),
        };
        let children_visible = visible && visit != Visit::Normalization;
        context.account_work((term.end - term.start) as usize)?;
        for index in term.start..term.end {
            let child = context.edges[index as usize];
            self.enqueue(context, id, Dependency::Type(child), children_visible)?;
        }
        if let Constructor::ArrayInfer(length) = kind {
            self.enqueue(context, id, Dependency::Const(length), children_visible)?;
        }
        if let Some((arguments, _)) = kind.application() {
            context.account_work(context.slots(arguments)?.len())?;
            for index in 0..context.slots(arguments)?.len() {
                if let Slot::Const(term) = context.slots(arguments)?[index] {
                    self.enqueue(context, id, Dependency::Const(term), children_visible)?;
                }
            }
        }
        Ok(true)
    }
    fn classify(
        &self,
        context: &mut Context,
        store: &Store,
        id: WatchId,
        dependency: Dependency,
        visible: bool,
    ) -> Result<Visit, Error> {
        let domain = match dependency {
            Dependency::Const(term) => {
                if context.constants().value(term)?.is_none() {
                    Some(Domain::Any)
                } else {
                    None
                }
            }
            Dependency::Type(term) => match context.terms[term.0 as usize].value {
                Value::Variable(domain) => Some(domain),
                _ => {
                    let own = match self.watches[id.0 as usize].fallback {
                        super::Fallback::Projection(root) => context.root(root)? == term,
                        _ => false,
                    };
                    if visible && !own && context.needs_normalization(store, term)? {
                        return Ok(Visit::Normalization);
                    }
                    None
                }
            },
        };
        Ok(match (domain, visible) {
            (Some(Domain::Integer | Domain::Float), true) => Visit::Numeric,
            (Some(Domain::Integer | Domain::Float), false) => Visit::HiddenNumeric,
            (Some(Domain::Any), true) => Visit::Other,
            (Some(Domain::Any), false) => Visit::HiddenOther,
            (None, true) => Visit::Done,
            (None, false) => Visit::HiddenDone,
        })
    }
}
