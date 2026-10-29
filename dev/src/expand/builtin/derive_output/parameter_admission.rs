//! The source const-parameter marker remains an ordinary checked implementation.
use super::{LexError, NodeId, Output, error};
impl Output<'_> {
    pub(super) fn parameter_marker(&mut self, fields: &[NodeId]) -> Result<(), LexError> {
        if !self.select(9) {
            return Ok(());
        }
        (self.charge)(1, 0)?;
        if !fields.is_empty() || !self.site.ast.list(self.site.item.generics).is_empty() {
            return Err(error(
                self.site.original.start,
                "source ConstParamTy payload or generic derives require a wider field-proof profile",
            ));
        }
        self.header("crate::marker::ConstParamTy_", false)?;
        self.text("{}")
    }
}
