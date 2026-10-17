//! A declaration query preserves its evaluation graph while one initializer becomes ready.
use super::{DefId, Error, Expression, NodeKey, Queries, Query, Scratch, State, Work, instances};
use std::collections::HashMap;

pub(in crate::sema) struct Continuation {
    expressions: HashMap<(instances::Context, NodeKey), Expression>,
    instances: instances::Instances,
    states: HashMap<(instances::Context, Query), State>,
    work: instances::Worklist,
    steps: usize,
    limit: usize,
}

impl Queries<'_> {
    /// Suspension occurs between complete body checks, never during inference.
    pub(super) fn suspend(mut self) -> (Scratch, Box<Continuation>) {
        let continuation = Continuation {
            expressions: std::mem::take(&mut self.expressions),
            instances: std::mem::take(&mut self.instances),
            states: std::mem::take(&mut self.states),
            work: std::mem::take(&mut self.work),
            steps: self.steps,
            limit: self.limit,
        };
        (self.into_scratch(), Box::new(continuation))
    }

    pub(super) fn resume(&mut self, continuation: Continuation) {
        self.expressions = continuation.expressions;
        self.instances = continuation.instances;
        self.states = continuation.states;
        self.work = continuation.work;
        self.steps = continuation.steps;
        self.limit = self.limit.max(continuation.limit);
    }

    pub(super) fn required_initializer(&self, work: &Work) -> Result<Option<DefId>, Error> {
        let index = match *work {
            Work::Enter(Query::Constant(index) | Query::Static(index)) => index,
            Work::Enter(Query::Instance(instance)) | Work::PrepareInstance(instance) => self
                .constants
                .get(instance.definition.0 as usize)
                .copied()
                .flatten()
                .ok_or_else(|| {
                    self.error(
                        Query::Instance(instance),
                        "selected constant has no initializer",
                    )
                })?,
            _ => return Ok(None),
        };
        let initializer = self.typing.facts.initializers[index];
        Ok((!initializer.type_checked).then_some(initializer.definition))
    }
}
