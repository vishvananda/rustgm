//! A reached type name authenticates its source declaration, then becomes
//! immutable constant text.
use super::{Constant, Evaluator, Failure};
use crate::{
    resolve::body_origin::BodyOrigin,
    sema::compiler_items::printed_names::{self, Selection},
    types::instance::Instance,
};
impl Evaluator<'_> {
    pub(crate) fn checked_type_name(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<Selection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.type_name(self.facts(), instance, origin)? else {
            return Ok(None);
        };
        catalog.require_type_name_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
    /// The subject's name as `&'static str` text, and the bytes it spelled.
    pub(crate) fn type_name_text(
        &mut self,
        selected: &Selection,
    ) -> Result<(Constant, usize), Failure> {
        let name = printed_names::name(self.facts(), self.queries.report, selected.subject())?;
        let text = self.queries.typing.facts.texts.intern(name.as_bytes())?;
        Ok((Constant::string(text), name.len()))
    }
}
