//! Grammar entry points splice opaque ASTs without examining their rendered spelling.
use super::fragment::FragmentKind;
use super::{LexError, NodeId, Parser, TokenKind};
impl<'inputs> Parser<'_, 'inputs> {
    #[inline]
    pub(super) fn splice(
        &mut self,
    ) -> Result<Option<crate::syntax::interpolation::Splice<'inputs>>, LexError> {
        if !self.has_opaque {
            return Ok(None);
        }
        let Some(token) = self.cursor.peek(0)? else {
            return Ok(None);
        };
        if !matches!(token.kind, TokenKind::Interpolated(_)) {
            return Ok(None);
        }
        self.inputs.get(token)
    }
    #[inline]
    pub(super) fn interpolated(
        &mut self,
        kinds: &[FragmentKind],
    ) -> Result<Option<NodeId>, LexError> {
        let Some(splice) = self.splice()? else {
            return Ok(None);
        };
        if !splice
            .syntax_kind()
            .is_some_and(|kind| kinds.contains(&kind))
        {
            return Ok(None);
        }
        self.required()?;
        splice.import(self.cursor.source, &mut self.ast).map(Some)
    }
}
