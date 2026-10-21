//! A declaration query preserves its evaluation graph while one initializer becomes ready.
use super::{DefId, Error, Expression, NodeKey, Queries, Query, Scratch, State, Work, instances};
use std::collections::HashMap;

pub(in crate::sema) enum Dependency {
    Body(crate::sema::bodies::Dependency),
    Initializer(DefId),
    Members(Vec<crate::sema::traits::implementations::ConstantRequest>),
    Execution(super::values::execution::Requirement),
}

pub(in crate::sema) struct Continuation {
    typing: Option<Box<crate::sema::bodies::State>>,
    expressions: HashMap<(instances::Context, NodeKey), Expression>,
    instances: instances::Instances,
    states: HashMap<(instances::Context, Query), State>,
    work: instances::Worklist,
    steps: usize,
    limit: usize,
    checked_members: std::collections::HashSet<Query>,
}

impl Queries<'_> {
    /// Retain unfinished inference as well as the enclosing value query graph.
    pub(super) fn suspend(mut self) -> (Scratch, Box<Continuation>) {
        let mut continuation = Continuation {
            typing: None,
            expressions: std::mem::take(&mut self.expressions),
            instances: std::mem::take(&mut self.instances),
            states: std::mem::take(&mut self.states),
            work: std::mem::take(&mut self.work),
            steps: self.steps,
            limit: self.limit,
            checked_members: std::mem::take(&mut self.checked_members),
        };
        let (scratch, typing) = self.into_storage();
        continuation.typing = typing;
        (scratch, Box::new(continuation))
    }

    pub(super) fn resume(&mut self, continuation: Continuation) {
        if let Some(typing) = continuation.typing {
            self.typing.restore_body(*typing);
        }
        self.expressions = continuation.expressions;
        self.instances = continuation.instances;
        self.states = continuation.states;
        self.work = continuation.work;
        self.steps = continuation.steps;
        self.limit = self.limit.max(continuation.limit);
        self.checked_members = continuation.checked_members;
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
        Ok((!initializer.type_checked()).then_some(initializer.definition))
    }
}
