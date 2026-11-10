//! Constant value knowledge consumes the exact source declaration of this report.
use super::{Evaluator, Failure};
use crate::{
    resolve::body_origin::BodyOrigin, sema::compiler_items::knowledge::Selection,
    types::instance::Instance,
};
impl Evaluator<'_> {
    pub(crate) fn checked_static_knowledge(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<Selection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.static_knowledge(self.facts(), instance, origin)? else {
            return Ok(None);
        };
        catalog.require_knowledge_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
}
