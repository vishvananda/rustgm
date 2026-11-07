//! A reached bit count authenticates its source declaration before any value is read.
use super::{Evaluator, Failure};
use crate::{
    resolve::body_origin::BodyOrigin, sema::compiler_items::bit_counts::Selection,
    types::instance::Instance,
};
impl Evaluator<'_> {
    pub(crate) fn checked_bit_count(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<Selection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.bit_count(self.facts(), instance, origin)? else {
            return Ok(None);
        };
        catalog.require_bit_count_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
}
