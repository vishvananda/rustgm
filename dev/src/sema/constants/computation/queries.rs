//! Prepared requests reuse the existing active query graph and iterative provider.
use super::SourceComputation;
use crate::sema::{
    constants::{Error, Queries, Query},
    values::execution::{Progress, Request, Requirement, Target},
};
impl Queries<'_> {
    pub(in crate::sema::constants) fn execute_source(
        &mut self,
        source: SourceComputation,
    ) -> Result<Option<Requirement>, Error> {
        let query = Query::Source(source);
        let result = source
            .result(self.typing.facts)
            .map_err(|e| self.error(query, e))?;
        let selected = source
            .resolve(self.typing.facts, result)
            .map_err(|e| self.error(query, e))?;
        let mut executor = self
            .typing
            .facts
            .constant_executor
            .take()
            .ok_or_else(|| self.error(query, "missing checked source computation service"))?;
        let request = Request {
            target: Target::SourceAssociated { source, result },
            arguments: Vec::new(),
        };
        let progress = (|| {
            executor.check(self.report, self.typing.facts, request.target)?;
            if let Some(value) =
                crate::sema::values::Reader::new(self.typing.facts).associated(selected.selected)
            {
                return Ok(Progress::Complete(value));
            }
            executor.advance(self.report, self.typing.facts, &request)
        })();
        self.typing.facts.constant_executor = Some(executor);
        match progress? {
            Progress::Complete(value) => {
                self.complete_query(query, value.0)?;
                Ok(None)
            }
            Progress::Prepare(requirement) => Ok(Some(requirement)),
        }
    }
}
