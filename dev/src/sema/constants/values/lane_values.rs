//! A reached SIMD intrinsic authenticates its source declaration before any value is read.
use super::{Evaluator, Failure};
use crate::{
    resolve::body_origin::BodyOrigin, sema::compiler_items::SimdSelection,
    types::instance::Instance,
};
impl super::Reader<'_> {
    pub(crate) fn source_simd(
        self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<SimdSelection>, &'static str> {
        self.facts
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.simd_selection(self.facts, instance, origin))
            .transpose()
            .map(Option::flatten)
    }
}
impl Evaluator<'_> {
    pub(crate) fn checked_source_simd(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<SimdSelection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.simd_selection(self.facts(), instance, origin)? else {
            return Ok(None);
        };
        catalog.require_provider_report(self.queries.report)?;
        Ok(Some(selected))
    }
}
