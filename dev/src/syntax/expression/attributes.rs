//! Grammar owners attach captured attributes; expansion evaluates configuration.
use super::{Kind, LexError, NodeId, P, Parser, TokenKind, error};
use crate::syntax::{
    attribute::{self, Style},
    item::{AttributeCapture, AttributeRange},
    token_tree::ForestBuilder,
};

impl Parser<'_> {
    pub(super) fn attributes(&mut self, inner: bool) -> Result<AttributeRange, LexError> {
        let start = u32::try_from(self.ast.attributes.len())
            .map_err(|_| error(self.cursor.end(), "too many attributes"))?;
        while let Some(token) = self.cursor.peek(0)? {
            let is_inner = match token.kind {
                TokenKind::InnerLineDoc | TokenKind::InnerBlockDoc => true,
                TokenKind::OuterLineDoc | TokenKind::OuterBlockDoc => false,
                TokenKind::Punctuation(P::Pound) => self
                    .cursor
                    .peek(1)?
                    .is_some_and(|t| t.kind == TokenKind::Punctuation(P::Bang)),
                _ => break,
            };
            if is_inner != inner {
                break;
            }
            let mut builder = ForestBuilder::default();
            builder.push(self.cursor.source, self.required()?)?;
            if token.kind == TokenKind::Punctuation(P::Pound) {
                if is_inner {
                    builder.push(self.cursor.source, self.expect(P::Bang)?)?;
                }
                builder.push(self.cursor.source, self.expect(P::OpenBracket)?)?;
                while !builder.balanced() {
                    builder.push(self.cursor.source, self.required()?)?;
                }
            }
            let forest = builder.finish(self.cursor.end())?;
            let (attribute, _) = attribute::at(&forest, self.cursor.source, 0)?
                .ok_or_else(|| error(token.start, "expected attribute"))?;
            if (attribute.style == Style::Inner) != inner {
                return Err(error(token.start, "attribute style disagrees with owner"));
            }
            self.ast
                .attributes
                .push(AttributeCapture { forest, attribute });
        }
        let end = u32::try_from(self.ast.attributes.len())
            .map_err(|_| error(self.cursor.end(), "too many attributes"))?;
        Ok(AttributeRange { start, end })
    }

    pub(super) fn attach(
        &mut self,
        range: AttributeRange,
        value: NodeId,
    ) -> Result<NodeId, LexError> {
        if range.start == range.end {
            return Ok(value);
        }
        let start = self.ast.node(value).span.start.min(
            self.ast.attributes[range.start as usize]
                .attribute
                .span
                .start,
        );
        self.node(
            start,
            Kind::Attributed {
                attributes: range,
                value,
            },
        )
    }
}
