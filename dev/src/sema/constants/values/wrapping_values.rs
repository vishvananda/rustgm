//! Actual source wrapping calls share declaration selection, never caller permission.
use super::{Evaluator, Failure, Reader};
use crate::{
    resolve::body_origin::BodyOrigin, sema::compiler_items::WrappingOffsetSelection,
    types::instance::Instance,
};
impl Reader<'_> {
    pub(crate) fn source_wrapping_offset(
        self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<WrappingOffsetSelection>, &'static str> {
        self.facts
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.wrapping_offset_selection(self.facts, instance, origin))
            .transpose()
            .map(Option::flatten)
    }
}
impl Evaluator<'_> {
    pub(crate) fn checked_source_wrapping_offset(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<WrappingOffsetSelection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.wrapping_offset_selection(self.facts(), instance, origin)?
        else {
            return Ok(None);
        };
        catalog.require_wrapping_offset_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
}
