//! Discover dependency leaves, including const slots that are not type edges.
use super::{Dependency, Storage, WatchId};
use crate::types::infer::{Constructor, Context, Error, Store, Value, arguments::Slot};

impl Storage {
    pub(super) fn poll(
        &mut self,
        context: &mut Context,
        store: &Store,
    ) -> Result<Option<WatchId>, Error> {
        loop {
            if let Some(id) = self.ready.pop_front() {
                return Ok(Some(id));
            }
            let Some((id, dependency)) = self.work.pop_front() else {
                return Ok(None);
            };
            context.step()?;
            let dependency = match dependency {
                Dependency::Type(term) => Dependency::Type(context.root(term)?),
                Dependency::Const(term) => Dependency::Const(context.constants().root(term)?),
            };
            if !self.watches[id.0 as usize].seen.insert(dependency) {
                self.complete(id)?;
                continue;
            }
            if self.visit(context, store, id, dependency)? {
                self.complete(id)?;
            } else {
                self.waiting.entry(dependency).or_default().push(id);
            }
        }
    }

    fn visit(
        &mut self,
        context: &mut Context,
        store: &Store,
        id: WatchId,
        dependency: Dependency,
    ) -> Result<bool, Error> {
        let term = match dependency {
            Dependency::Const(term) => return Ok(context.constants().value(term)?.is_some()),
            Dependency::Type(term) => context.terms[term.0 as usize],
        };
        let kind = match term.value {
            Value::Variable(_) => return Ok(false),
            Value::Known(ty) => {
                store.kind(ty).ok_or(Error::InvalidIdentity)?;
                return Ok(true);
            }
            Value::Construct(kind) => kind,
        };
        context.account_work((term.end - term.start) as usize)?;
        for &child in &context.edges[term.start as usize..term.end as usize] {
            self.enqueue(id, Dependency::Type(child))?;
        }
        if let Constructor::ArrayInfer(length) = kind {
            self.enqueue(id, Dependency::Const(length))?;
        }
        if let Some((arguments, _)) = kind.application() {
            context.account_work(context.slots(arguments)?.len())?;
            for slot in context.slots(arguments)? {
                if let Slot::Const(term) = *slot {
                    self.enqueue(id, Dependency::Const(term))?;
                }
            }
        }
        Ok(true)
    }
}
