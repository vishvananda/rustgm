//! A declaration query preserves its evaluation graph while one initializer becomes ready.
use super::{DefId, Error, Expression, NodeKey, Queries, Query, Scratch, State, Work, instances};
use crate::support::hash::HashMap;

pub(in crate::sema) enum Dependency {
    Body(crate::sema::bodies::Dependency),
    Initializer(DefId),
    Members(Vec<crate::sema::traits::implementations::ConstantRequest>),
    Execution(super::values::execution::Requirement),
}

pub(in crate::sema) struct Continuation {
    typing: Option<Box<crate::sema::bodies::State>>,
    pattern_scope: Option<super::PatternScope>,
    pattern_dispatch: Option<crate::sema::dispatch::Dispatcher>,
    value_scope: Option<instances::ValueScope>,
    pattern_frames: Option<Box<super::pending_patterns::Frames>>,
    expressions: HashMap<(instances::Context, NodeKey), Expression>,
    instances: instances::Instances,
    states: HashMap<(instances::Context, Query), State>,
    work: instances::Worklist,
    steps: usize,
    limit: usize,
    checked_members: crate::support::hash::HashSet<Query>,
}

impl Queries<'_> {
    /// Retain unfinished inference as well as the enclosing value query graph.
    pub(super) fn suspend(mut self) -> Result<(Scratch, Box<Continuation>), Error> {
        self.capture_pending_patterns()?;
        self.resume_pattern_dispatch()?;
        self.reserve_pattern_dispatch()?;
        self.check_pattern_storage(0)?;
        self.retain_pattern_dispatcher(64 * 1024 * 1024)?;
        let mut continuation = Continuation {
            typing: None,
            pattern_scope: self.pattern_scope.take(),
            pattern_dispatch: self.pattern_dispatch.take(),
            value_scope: self.value_scope.take(),
            pattern_frames: self.pattern_frames.take(),
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
        Ok((scratch, Box::new(continuation)))
    }

    fn retain_pattern_dispatcher(&mut self, limit: usize) -> Result<(), Error> {
        if let Some(dispatch) = &mut self.pattern_dispatch {
            let (bytes, visits) = dispatch.retained_allocation();
            if bytes > limit {
                return Err(Error::at(
                    super::FileId(0),
                    0,
                    "suspended pattern dispatcher exceeds retained storage budget",
                ));
            }
            self.steps = self.steps.saturating_add(visits);
            self.budget()?;
        }
        Ok(())
    }

    pub(super) fn resume(&mut self, continuation: Continuation) {
        if let Some(typing) = continuation.typing {
            self.typing.restore_body(*typing);
        }
        self.pattern_scope = continuation.pattern_scope;
        self.pattern_dispatch = continuation.pattern_dispatch;
        self.value_scope = continuation.value_scope;
        self.pattern_frames = continuation.pattern_frames;
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
                .typing
                .facts
                .initializer_index(instance.definition)
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

#[cfg(test)]
#[path = "../../../tests/invariants/pattern_continuations.rs"]
mod tests;
