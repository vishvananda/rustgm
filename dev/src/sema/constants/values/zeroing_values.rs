//! A reached zeroing intrinsic authenticates its source declaration before any byte is written.
use super::{Evaluator, Failure, Reader};
use crate::{
    resolve::body_origin::BodyOrigin, sema::compiler_items::zeroing_contract::Selection,
    types::instance::Instance,
};
impl Reader<'_> {
    pub(crate) fn source_zeroing(
        self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<Selection>, &'static str> {
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        catalog.zeroing(self.facts, instance, origin)
    }
}
impl Evaluator<'_> {
    pub(crate) fn checked_zeroing(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<Selection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.zeroing(self.facts(), instance, origin)? else {
            return Ok(None);
        };
        catalog.require_zeroing_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
}
