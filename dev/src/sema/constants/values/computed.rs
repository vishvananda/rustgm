//! Source values retain their sealed normalized result for final escape validation.
use super::execution::SourceComputation;
use super::{Constant, Error, Evaluator, Query, Work};
impl Evaluator<'_> {
    pub(crate) fn complete_source_associated(
        &mut self,
        instance: super::Instance,
        value: Constant,
    ) -> Result<(), &'static str> {
        instance.publish(self.queries.typing.facts, None, value)
    }

    pub(crate) fn checked_source(
        &mut self,
        source: SourceComputation,
    ) -> Result<(super::Instance, crate::types::TypeId), &'static str> {
        let result = source.result(self.facts())?;
        let selected = source.resolve(self.queries.typing.facts, result)?;
        Ok((selected.selected, result))
    }

    pub(crate) fn source_associated(
        &mut self,
        source: SourceComputation,
    ) -> Result<Constant, Error> {
        let result = source
            .result(self.facts())
            .map_err(|e| self.queries.error(Query::Source(source), e))?;
        source
            .validate(self.queries.typing.facts, result)
            .map_err(|e| self.queries.error(Query::Source(source), e))?;
        let query = Query::Source(source);
        self.queries.work.push(Work::Enter(query));
        self.queries.drain()?;
        let value = Constant(self.queries.value(query)?);
        self.checked_reference_result(query, result, value)
    }
}

impl super::Reader<'_> {
    pub(crate) fn source_associated(
        self,
        source: SourceComputation,
    ) -> Result<(super::Instance, Constant), &'static str> {
        let (instance, _) = source.identity(self.facts)?;
        let value = self
            .associated(instance)
            .ok_or("source associated constant is not complete")?;
        Ok((instance, value))
    }
}
