//! Required inline values use their already typed anonymous body and the checked service.
use super::{
    Constant, Error, Evaluator, Query, Reader, Work,
    execution::{Progress, Request, Target},
};
use crate::{resolve::unit::NodeKey, sema::constants::instances::ValueScope};

impl Evaluator<'_> {
    /// Demand one closed inline constant without inventing a declaration or storage.
    /// # Errors
    /// Preserves eligibility, required value faults and explicit specialization boundaries.
    pub fn inline(&mut self, source: NodeKey) -> Result<Constant, Error> {
        let query = Query::Node(source);
        if let Some(value) = Reader::new(self.queries.typing.facts).inline(source) {
            return Ok(value);
        }
        let result = self
            .queries
            .typing
            .facts
            .inline_constants
            .as_deref()
            .and_then(|registry| registry.get(source))
            .and_then(|body| body.result)
            .ok_or_else(|| {
                self.queries
                    .error(query, "inline constant result is not ready")
            })?;
        self.queries
            .typing
            .facts
            .inline_constants
            .as_mut()
            .ok_or_else(|| {
                super::super::Declarations::error(
                    self.queries.report,
                    source,
                    "missing inline registry",
                )
            })?
            .require(source, result)
            .map_err(|e| super::super::Declarations::error(self.queries.report, source, e))?;
        let scope = self.queries.value_scope.replace(ValueScope::Anonymous {
            body: source,
            ty: result,
        });
        self.queries.work.push(Work::Execute(source));
        let value = self
            .queries
            .drain()
            .and_then(|()| self.queries.value(query).map(Constant));
        self.queries.value_scope = scope;
        let value = self.checked_reference_result(query, result, value?)?;
        self.complete_inline(source, value)
            .map_err(|e| self.queries.error(query, e))?;
        Ok(value)
    }

    pub(crate) fn require_inline(
        &mut self,
        source: NodeKey,
        result: crate::types::TypeId,
    ) -> Result<(), &'static str> {
        self.queries
            .typing
            .facts
            .inline_constants
            .as_mut()
            .ok_or("missing required inline registry")?
            .require(source, result)
    }

    pub(crate) fn complete_inline(
        &mut self,
        source: NodeKey,
        value: Constant,
    ) -> Result<(), &'static str> {
        self.queries
            .typing
            .facts
            .inline_constants
            .as_mut()
            .ok_or("missing completed inline registry")?
            .publish(source, value)
    }
}

impl Evaluator<'_> {
    pub(crate) fn complete_inline_instance(
        &mut self,
        instance: crate::sema::inline_constants::InlineInstance,
        result: crate::types::TypeId,
        value: Constant,
    ) -> Result<(), &'static str> {
        self.queries
            .typing
            .facts
            .inline_constants
            .as_mut()
            .ok_or("missing anonymous instance registry")?
            .publish_instance(instance, result, value)
    }
    pub(crate) fn inline_instance(
        &mut self,
        instance: crate::sema::inline_constants::InlineInstance,
        result: crate::types::TypeId,
    ) -> Result<Constant, Error> {
        if let Some(value) =
            Reader::new(self.queries.typing.facts).inline_instance(instance, result)
        {
            return Ok(value);
        }
        let query = Query::Node(instance.source());
        let mut executor = self
            .queries
            .typing
            .facts
            .constant_executor
            .take()
            .ok_or_else(|| {
                self.queries
                    .error(query, "missing anonymous constant execution service")
            })?;
        let progress = executor.advance(
            self.queries.report,
            self.queries.typing.facts,
            &Request {
                target: Target::Anonymous { instance, result },
                arguments: Vec::new(),
            },
        );
        self.queries.typing.facts.constant_executor = Some(executor);
        let Progress::Complete(value) = progress? else {
            return Err(self.queries.error(
                query,
                "anonymous instance has an unresolved declaration dependency",
            ));
        };
        let value = self.checked_reference_result(query, result, value)?;
        self.complete_inline_instance(instance, result, value)
            .map_err(|e| self.queries.error(query, e))?;
        Ok(value)
    }
}
