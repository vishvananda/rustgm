//! Actual source integer calls share declaration selection, never caller permission.
use super::{Evaluator, Failure, Reader};
use crate::{
    resolve::body_origin::BodyOrigin, sema::compiler_items::SubtractionSelection,
    types::instance::Instance,
};
impl Reader<'_> {
    pub(crate) fn source_subtraction(
        self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<SubtractionSelection>, &'static str> {
        self.facts
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.subtraction_selection(self.facts, instance, origin))
            .transpose()
            .map(Option::flatten)
    }
}
impl Evaluator<'_> {
    pub(crate) fn checked_source_subtraction(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<SubtractionSelection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.subtraction_selection(self.facts(), instance, origin)? else {
            return Ok(None);
        };
        catalog.require_subtraction_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
}
