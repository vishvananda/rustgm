//! A reached `assume` authenticates its source declaration before its condition is read.
use super::{Evaluator, Failure, Reader};
use crate::{
    resolve::body_origin::BodyOrigin, sema::compiler_items::assume_contract::AssumeSelection,
    types::instance::Instance,
};
impl Evaluator<'_> {
    pub(crate) fn checked_assumption(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<AssumeSelection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.assumption(self.facts(), instance, origin)? else {
            return Ok(None);
        };
        catalog.require_assume_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
}

impl Reader<'_> {
    pub(crate) fn source_assumption(
        self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<AssumeSelection>, &'static str> {
        self.facts
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.assumption(self.facts, instance, origin))
            .transpose()
            .map(Option::flatten)
    }
}
