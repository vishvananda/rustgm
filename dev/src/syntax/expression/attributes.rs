//! Grammar owners attach captured attributes; expansion evaluates configuration.
use super::{Kind, LexError, NodeId, P, Parser, TokenKind, error};
use crate::syntax::{
    attribute::{self, Style},
    interpolation::Nonterminal,
    item::{AttributeCapture, AttributeRange},
    token_tree::ForestBuilder,
};

impl Parser<'_, '_> {
    /// This is syntax admission only. Configuration and resolved source binding
    /// ownership decide whether an enabled declaration may omit its body.
    pub(super) fn item_attributes(&mut self, range: AttributeRange) -> bool {
        let mut intrinsic = false;
        for capture in &self.ast.attributes[range.start as usize..range.end as usize] {
            match capture.attribute.name(&capture.forest, self.cursor.source) {
                Some("cfg" | "cfg_attr") => self.configured_items = true,
                Some("rustc_intrinsic") => intrinsic = true,
                _ => {}
            }
        }
        intrinsic
    }
    /// Most owners have no attributes; only a doc comment or `#` enters the scan.
    // Forced only without debug assertions: debug builds keep the call so
    // recursive-descent frames stay small enough for the nesting limit.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-owner check that almost always finds no attribute; callgrind shows calls dominate"
        )
    )]
    pub(super) fn attributes(&mut self, inner: bool) -> Result<AttributeRange, LexError> {
        let start = u32::try_from(self.ast.attributes.len())
            .map_err(|_| error(self.cursor.end(), "too many attributes"))?;
        if !self.cursor.peek(0)?.is_some_and(|token| {
            matches!(
                token.kind,
                TokenKind::InnerLineDoc
                    | TokenKind::InnerBlockDoc
                    | TokenKind::OuterLineDoc
                    | TokenKind::OuterBlockDoc
                    | TokenKind::Punctuation(P::Pound)
            )
        }) {
            return Ok(AttributeRange { start, end: start });
        }
        self.attribute_list(start, inner)
    }

    fn attribute_list(&mut self, start: u32, inner: bool) -> Result<AttributeRange, LexError> {
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
            let capture = self.attribute_capture(is_inner)?;
            if (capture.attribute.style == Style::Inner) != inner {
                return Err(error(token.start, "attribute style disagrees with owner"));
            }
            self.ast.attributes.push(capture);
        }
        let end = u32::try_from(self.ast.attributes.len())
            .map_err(|_| error(self.cursor.end(), "too many attributes"))?;
        Ok(AttributeRange { start, end })
    }

    fn attribute_capture(&mut self, inner: bool) -> Result<AttributeCapture, LexError> {
        let token = self.required()?;
        let mut builder = ForestBuilder::default();
        builder.push(self.cursor.source, token)?;
        if token.kind == TokenKind::Punctuation(P::Pound) {
            if inner {
                builder.push(self.cursor.source, self.expect(P::Bang)?)?;
            }
            builder.push(self.cursor.source, self.expect(P::OpenBracket)?)?;
            if let Some(splice) = self.splice()?
                && matches!(splice.fragment, Nonterminal::Metadata { .. })
                && self
                    .cursor
                    .peek(1)?
                    .is_some_and(|t| t.kind == TokenKind::Punctuation(P::CloseBracket))
            {
                self.required()?;
                let close = self.expect(P::CloseBracket)?;
                let mut capture = splice.metadata()?;
                capture.attribute.style = if inner { Style::Inner } else { Style::Outer };
                capture.attribute.span = crate::syntax::token_tree::Span {
                    start: token.start,
                    end: close.end,
                };
                return Ok(capture);
            }
            while !builder.balanced() {
                if let Some(splice) = self.splice()? {
                    if let Nonterminal::Metadata { forest, .. } = splice.fragment {
                        self.required()?;
                        builder.append(forest, |span| splice.span(span))?;
                        continue;
                    }
                    if splice.append_attribute_path(self.cursor.source, &mut builder)? {
                        self.required()?;
                        continue;
                    }
                }
                builder.push(self.cursor.source, self.required()?)?;
            }
        }
        let forest = builder.finish(self.cursor.end())?;
        let (attribute, _) = attribute::at(&forest, self.cursor.source, 0)?
            .ok_or_else(|| error(token.start, "expected attribute"))?;
        Ok(AttributeCapture { forest, attribute })
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-owner check that almost always finds no attribute; callgrind shows calls dominate"
        )
    )]
    pub(super) fn attach(
        &mut self,
        range: AttributeRange,
        value: NodeId,
    ) -> Result<NodeId, LexError> {
        if range.start == range.end {
            return Ok(value);
        }
        self.attributed(range, value)
    }

    fn attributed(&mut self, range: AttributeRange, value: NodeId) -> Result<NodeId, LexError> {
        let start = self.ast.node(value).span.start.min(
            self.ast.attributes[range.start as usize]
                .attribute
                .span
                .start,
        );
        let owner = self.node(
            start,
            Kind::Attributed {
                attributes: range,
                value,
            },
        )?;
        self.ast.pending.owners.push(owner);
        Ok(owner)
    }
}
