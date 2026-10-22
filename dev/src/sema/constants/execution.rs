//! General expression demands yield exact body requirements to declaration work.
use super::values::execution::{Progress, Request, Requirement, Target};
use super::{Error, NodeKey, Queries, Query};

impl Queries<'_> {
    fn execution_target(
        &mut self,
        source: NodeKey,
        result: crate::types::TypeId,
    ) -> Result<Target, Error> {
        let result = self.substituted_type(source, result)?;
        let context = self.execution_context();
        if let Some(instance) = context
            && self
                .typing
                .facts
                .initializer_index(instance.definition)
                .and_then(|index| self.typing.facts.initializers.get(index))
                .is_some_and(|initializer| initializer.value == source)
        {
            if !self
                .typing
                .facts
                .types
                .arguments_are_concrete(instance.arguments)
                .map_err(|e| self.error(Query::Node(source), e))?
            {
                let original = self
                    .typing
                    .facts
                    .initializer_index(instance.definition)
                    .and_then(|index| self.typing.facts.initializers.get(index))
                    .map(crate::sema::model::Initializer::semantic_source)
                    .ok_or_else(|| self.error(Query::Node(source), "missing source initializer"))?;
                return self.source_computation(instance, original, result);
            }
            let result = self.complete_type(source, result)?;
            return Ok(Target::Associated { instance, result });
        }
        let result = self.complete_type(source, result)?;
        if let Some(initializer) = context
            && self
                .typing
                .facts
                .inline_constants
                .as_deref()
                .and_then(|r| r.get(source))
                .is_some()
        {
            let context = if self
                .typing
                .facts
                .types
                .arguments_are_concrete(initializer.arguments)
                .map_err(|e| self.error(Query::Node(source), e))?
            {
                crate::sema::inline_constants::Context::Initializer(initializer)
            } else {
                let parent = self
                    .typing
                    .facts
                    .initializer_index(initializer.definition)
                    .and_then(|index| self.typing.facts.initializers.get(index))
                    .copied()
                    .ok_or_else(|| {
                        self.error(Query::Node(source), "missing inline initializer context")
                    })?;
                let requested = self.substituted_type(source, parent.ty)?;
                match self.source_computation(initializer, parent.semantic_source(), requested)? {
                    Target::SourceAssociated { source, .. } => {
                        crate::sema::inline_constants::Context::SourceInitializer(source)
                    }
                    Target::Associated { instance, .. } => {
                        crate::sema::inline_constants::Context::Initializer(instance)
                    }
                    _ => {
                        return Err(self.error(
                            Query::Node(source),
                            "inline parent selection changed target kind",
                        ));
                    }
                }
            };
            let instance = crate::sema::inline_constants::InlineInstance::select(
                self.typing.facts,
                source,
                context,
            )
            .map_err(|e| self.error(Query::Node(source), e))?;
            return Ok(Target::Anonymous { instance, result });
        }
        if context.is_none()
            && let Some(super::instances::ValueScope::Initializer { index, .. }) = self.value_scope
        {
            let initializer = self.typing.facts.initializers[index];
            if initializer.value == source {
                return Ok(Target::Initializer(initializer.definition));
            }
        }
        Ok(Target::Expression {
            source,
            result,
            context,
        })
    }
    pub(super) fn check_execution(&mut self, source: NodeKey) -> Result<(), Error> {
        let result = self
            .typing
            .facts
            .inline_constants
            .as_deref()
            .and_then(|registry| registry.get(source))
            .and_then(|body| body.result)
            .or_else(|| {
                self.typing
                    .facts
                    .node(source)
                    .and_then(|node| node.adjustment.map(|a| a.target).or(node.ty))
            })
            .ok_or_else(|| self.error(Query::Node(source), "missing constant eligibility type"))?;
        let target = self.execution_target(source, result)?;
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
            .inline_constants
            .as_deref()
            .and_then(|registry| registry.get(source))
            .and_then(|body| body.result)
            .or_else(|| {
                self.typing
                    .facts
                    .node(source)
                    .and_then(|node| node.adjustment.map(|a| a.target).or(node.ty))
            })
            .ok_or_else(|| self.error(Query::Node(source), "missing constant execution type"))?;
        let request = Request {
            target: self.execution_target(source, result)?,
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

#[cfg(test)]
#[path = "../../../tests/invariants/root_requests.rs"]
mod tests;
