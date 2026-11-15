//! A reached type identity authenticates its source declaration before any value is built.
use super::{Evaluator, Failure};
use crate::{
    resolve::body_origin::BodyOrigin, sema::compiler_items::identity_words::Selection,
    types::instance::Instance,
};
impl Evaluator<'_> {
    pub(crate) fn checked_type_identity(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<Selection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.type_identity(self.facts(), instance, origin)? else {
            return Ok(None);
        };
        catalog.require_type_identity_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
}
