//! The source const-parameter marker remains an ordinary checked implementation.
use super::{LexError, Output};
impl Output<'_> {
    pub(super) fn parameter_marker(&mut self) -> Result<(), LexError> {
        if !self.select(9) {
            return Ok(());
        }
        (self.charge)(1, 0)?;
        // Ordinary generic bounds are emitted by the shared header writer.
        // Semantic conformance separately proves every actual substituted field.
        self.header("crate::marker::ConstParamTy_", false)?;
        self.text("{}")
    }
}
