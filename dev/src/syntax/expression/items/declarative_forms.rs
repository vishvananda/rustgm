//! Both declarative spellings retain each delimited token forest exactly once.
use super::{ItemKind, LexError, P, Parser, TokenKind, error};

impl Parser<'_, '_> {
    pub(super) fn macro_definition(&mut self, scoped: bool) -> Result<ItemKind, LexError> {
        let mut first = self.required()?;
        let parameters = if scoped && first.kind == TokenKind::Punctuation(P::OpenParen) {
            let parameters = self.macro_capture(first)?;
            first = self.required()?;
            Some(parameters)
        } else {
            None
        };
        if scoped && first.kind != TokenKind::Punctuation(P::OpenBrace) {
            return Err(error(first.start, "scoped macro requires a braced body"));
        }
        if !matches!(
            first.kind,
            TokenKind::Punctuation(P::OpenBrace | P::OpenParen | P::OpenBracket)
        ) {
            return Err(error(first.start, "expected macro definition delimiter"));
        }
        let capture = self.macro_capture(first)?;
        if scoped {
            Ok(ItemKind::DeclarativeMacro {
                parameters,
                capture,
            })
        } else {
            if first.kind != TokenKind::Punctuation(P::OpenBrace) {
                self.expect(P::Semicolon)?;
            }
            Ok(ItemKind::MacroRules { capture })
        }
    }

    fn macro_capture(&mut self, first: crate::lex::token::Token) -> Result<u32, LexError> {
        let forest = self.cursor.capture(first)?;
        let capture = u32::try_from(self.ast.captures.len())
            .map_err(|_| error(first.start, "too many captures"))?;
        self.ast.captures.push(forest);
        Ok(capture)
    }
}
