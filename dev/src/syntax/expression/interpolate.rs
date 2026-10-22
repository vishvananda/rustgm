//! Grammar entry points splice opaque ASTs without examining their rendered spelling.
use super::fragment::FragmentKind;
use super::{LexError, NodeId, Parser, TokenKind};
impl<'inputs> Parser<'_, 'inputs> {
    // Forced only without debug assertions: debug builds keep the call so
    // recursive-descent frames stay small enough for the nesting limit.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-entry check that ordinary source never takes; callgrind shows calls dominate"
        )
    )]
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
    /// Ordinary source has no opaque inputs; only expansions reach the import.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-entry check that ordinary source never takes; callgrind shows calls dominate"
        )
    )]
    pub(super) fn interpolated(
        &mut self,
        kinds: &[FragmentKind],
    ) -> Result<Option<NodeId>, LexError> {
        if !self.has_opaque {
            return Ok(None);
        }
        self.import_interpolated(kinds)
    }
    fn import_interpolated(&mut self, kinds: &[FragmentKind]) -> Result<Option<NodeId>, LexError> {
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
