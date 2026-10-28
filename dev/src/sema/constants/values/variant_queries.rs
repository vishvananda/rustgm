//! Reached source enum operations borrow the existing actual catalog.
use super::{Evaluator, Failure, Reader};
use crate::{
    resolve::body_origin::BodyOrigin, sema::compiler_items::DiscriminantSelection,
    types::instance::Instance,
};

impl Reader<'_> {
    pub(crate) fn source_discriminant(
        self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<DiscriminantSelection>, &'static str> {
        self.facts
            .source_bindings
            .as_deref()
            .map(|c| c.discriminant_selection(self.facts, instance, origin))
            .transpose()
            .map(Option::flatten)
    }
}
impl Evaluator<'_> {
    pub(crate) fn check_source_offset_report(
        &self,
        definition: crate::resolve::model::DefId,
    ) -> Result<(), &'static str> {
        self.facts()
            .source_bindings
            .as_deref()
            .ok_or("source offset_of catalog disappeared")?
            .require_offset_report(self.queries.report, self.facts(), definition)
    }
    pub(crate) fn checked_source_discriminant(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<DiscriminantSelection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.discriminant_selection(self.facts(), instance, origin)? else {
            return Ok(None);
        };
        catalog.require_discriminant_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
}
