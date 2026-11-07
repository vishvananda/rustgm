//! A reached `assume` authenticates its source declaration before its condition is read.
use super::{Evaluator, Failure};
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
