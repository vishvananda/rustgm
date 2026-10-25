//! Restricted trait introducers share the ordinary, single-parse module path.
use super::{Kind, LexError, NodeId, P, Parser, TokenKind, Word};

impl Parser<'_, '_> {
    pub(super) fn impl_restriction(&mut self) -> Result<Option<NodeId>, LexError> {
        if !self.cursor.at_word(Word::Impl)?
            || !self
                .cursor
                .peek(1)?
                .is_some_and(|t| t.kind == TokenKind::Punctuation(P::OpenParen))
            || !matches!(
                self.cursor.peek_word(2)?,
                Word::In | Word::SelfValue | Word::Super | Word::Crate
            )
        {
            return Ok(None);
        }
        let start = self.required()?.start;
        self.expect(P::OpenParen)?;
        let explicit = self.eat_word(Word::In)?;
        let path = self.simple_path()?;
        if !explicit
            && matches!(self.ast.node(path).kind, Kind::Path { segments, .. } if self.ast.list(segments).len() != 1)
        {
            return Err(self.error("nested implementation restriction requires in")?);
        }
        self.expect(P::CloseParen)?;
        self.node(start, Kind::ImplRestriction { path }).map(Some)
    }
}
