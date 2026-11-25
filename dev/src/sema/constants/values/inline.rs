//! Required inline values use their already typed anonymous body and the checked service.
use super::{
    Constant, Error, Evaluator, Query, Reader, Work,
    execution::{Progress, Request, Target},
};
use crate::{resolve::body_origin::BodyOrigin, sema::constants::instances::ValueScope};

impl Evaluator<'_> {
    /// Demand one closed inline constant without inventing a declaration or storage.
    /// # Errors
    /// Preserves eligibility, required value faults and explicit specialization boundaries.
    pub fn inline(&mut self, origin: BodyOrigin) -> Result<Constant, Error> {
        if let Some(value) = Reader::new(self.queries.typing.facts).inline(origin) {
            return Ok(value);
        }
        let result = self
            .queries
            .typing
            .facts
            .inline_constants
            .as_deref()
            .and_then(|registry| registry.get_origin(origin))
            .and_then(|body| body.result)
            .ok_or_else(|| {
                self.queries
                    .report
                    .body_error(origin, "inline constant result is not ready")
            })?;
        self.queries
            .typing
            .facts
            .inline_constants
            .as_mut()
            .ok_or_else(|| {
                self.queries
                    .report
                    .body_error(origin, "missing inline registry")
            })?
            .require_origin(origin, result)
            .map_err(|e| self.queries.report.body_error(origin, e))?;
        let BodyOrigin::Source(source) = origin else {
            let mut executor = self
                .queries
                .typing
                .facts
                .constant_executor
                .take()
                .ok_or_else(|| {
                    self.queries
                        .report
                        .body_error(origin, "missing anonymous constant execution service")
                })?;
            let progress = executor.advance(
                self.queries.report,
                self.queries.typing.facts,
                &Request {
                    target: Target::Expression {
                        source: origin,
                        result,
                        context: None,
                    },
                    arguments: Vec::new(),
                },
            );
            self.queries.typing.facts.constant_executor = Some(executor);
            let Progress::Complete(value) = progress? else {
                return Err(self.queries.report.body_error(
                    origin,
                    "imported inline constant has an unresolved declaration dependency",
                ));
            };
            let value = self.checked_reference_result_origin(origin, result, value)?;
            self.complete_inline(origin, value)
                .map_err(|e| self.queries.report.body_error(origin, e))?;
            return Ok(value);
        };
        let query = Query::Node(source);
        let scope = self.queries.value_scope.replace(ValueScope::Anonymous {
            body: source,
            ty: result,
        });
        self.queries.work.push(Work::Execute(
            crate::sema::constants::Execution::Expression(source),
        ));
        let value = self
            .queries
            .drain()
            .and_then(|()| self.queries.value(query).map(Constant));
        self.queries.value_scope = scope;
        let value = self.checked_reference_result(query, result, value?)?;
        self.complete_inline(origin, value)
            .map_err(|e| self.queries.error(query, e))?;
        Ok(value)
    }

    pub(crate) fn require_inline(
        &mut self,
        source: BodyOrigin,
        result: crate::types::TypeId,
    ) -> Result<(), &'static str> {
        self.queries
            .typing
            .facts
            .inline_constants
            .as_mut()
            .ok_or("missing required inline registry")?
            .require_origin(source, result)
    }

    pub(crate) fn complete_inline(
        &mut self,
        source: BodyOrigin,
        value: Constant,
    ) -> Result<(), &'static str> {
        self.queries
            .typing
            .facts
            .inline_constants
            .as_mut()
            .ok_or("missing completed inline registry")?
            .publish_origin(source, value)
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
        let origin = instance.origin(self.facts()).ok_or_else(|| {
            Error::unlocated("anonymous instance lost its retained typed template")
        })?;
        let mut executor = self
            .queries
            .typing
            .facts
            .constant_executor
            .take()
            .ok_or_else(|| {
                self.queries
                    .report
                    .body_error(origin, "missing anonymous constant execution service")
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
            return Err(self.queries.report.body_error(
                origin,
                "anonymous instance has an unresolved declaration dependency",
            ));
        };
        let value = self.checked_reference_result_origin(origin, result, value)?;
        self.complete_inline_instance(instance, result, value)
            .map_err(|e| self.queries.report.body_error(origin, e))?;
        Ok(value)
    }
}
