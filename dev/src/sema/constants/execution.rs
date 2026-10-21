//! General expression demands yield exact body requirements to declaration work.
use super::values::execution::{Progress, Request, Requirement, Target};
use super::{Error, NodeKey, Queries, Query};

impl Queries<'_> {
    pub(super) fn check_execution(&mut self, source: NodeKey) -> Result<(), Error> {
        let result = self
            .typing
            .facts
            .node(source)
            .and_then(|node| node.ty)
            .ok_or_else(|| self.error(Query::Node(source), "missing constant eligibility type"))?;
        let result = self.specialized_type(source, result)?;
        let target = Target::Expression {
            source,
            result,
            context: self.execution_context(),
        };
        let mut executor = self.typing.facts.constant_executor.take().ok_or_else(|| {
            self.error(
                Query::Node(source),
                "missing checked constant eligibility service",
            )
        })?;
        let result = executor.check(self.report, self.typing.facts, target);
        self.typing.facts.constant_executor = Some(executor);
        result
    }
    pub(super) fn execute(&mut self, source: NodeKey) -> Result<Option<Requirement>, Error> {
        let result = self
            .typing
            .facts
            .node(source)
            .and_then(|node| node.ty)
            .ok_or_else(|| self.error(Query::Node(source), "missing constant execution type"))?;
        let result = self.specialized_type(source, result)?;
        let request = Request {
            target: Target::Expression {
                source,
                result,
                context: self.execution_context(),
            },
            arguments: Vec::new(),
        };
        let mut executor = self.typing.facts.constant_executor.take().ok_or_else(|| {
            self.error(
                Query::Node(source),
                "missing checked constant execution service",
            )
        })?;
        let progress = executor.advance(self.report, self.typing.facts, &request);
        self.typing.facts.constant_executor = Some(executor);
        match progress? {
            Progress::Complete(value) => {
                self.complete_query(Query::Node(source), value.0)?;
                Ok(None)
            }
            Progress::Prepare(requirement) => Ok(Some(requirement)),
        }
    }
}
