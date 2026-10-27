//! One verbatim input interval keeps typed subtree spans and token origins aligned.
use super::{Location, Opaque, OpaqueSource, Origin, Source, Stream};
use crate::{
    expand::{
        macros::transcribe::Origin as PieceOrigin,
        marks::{ExpansionId, SyntaxContextId},
    },
    lex::token::{Kind, LexError, Token},
    syntax::token_tree::{Span, error},
};

#[derive(Clone, Copy)]
pub(crate) struct Projection {
    original: Span,
    destination: Span,
}
impl Projection {
    pub(crate) fn span(self, span: Span) -> Result<Span, LexError> {
        if span.start < self.original.start || span.end < span.start || span.end > self.original.end
        {
            return Err(error(
                span.start,
                "typed interval span is outside its original input",
            ));
        }
        let start = self
            .destination
            .start
            .checked_add(span.start - self.original.start);
        let end = self
            .destination
            .start
            .checked_add(span.end - self.original.start);
        match (start, end) {
            (Some(start), Some(end)) if end <= self.destination.end => Ok(Span { start, end }),
            _ => Err(error(
                span.start,
                "typed interval destination exceeds its image",
            )),
        }
    }
}
impl Stream {
    /// This is spelling/provenance transfer only; it grants no macro permission.
    /// The caller has prepaid the selected token/image and coexistence storage.
    pub(crate) fn forward_interval(
        &mut self,
        source: Source<'_>,
        tokens: &[Token],
        span: Span,
        mut provenance: impl FnMut(Token) -> Result<(SyntaxContextId, bool), LexError>,
    ) -> Result<Projection, LexError> {
        if self.registered {
            return Err(error(
                span.start,
                "cannot append to a registered expansion stream",
            ));
        }
        let image = source
            .text
            .get(span.start as usize..span.end as usize)
            .ok_or_else(|| error(span.start, "typed interval is outside its source image"))?;
        let start = u32::try_from(self.source.len())
            .map_err(|_| error(span.start, "typed interval source exceeds 32 bits"))?;
        let end = start
            .checked_add(span.end - span.start)
            .ok_or_else(|| error(span.start, "typed interval image exceeds 32 bits"))?;
        let projection = Projection {
            original: span,
            destination: Span { start, end },
        };
        let mut previous = span.start;
        for &token in tokens {
            if token.start < previous || token.end <= token.start || token.end > span.end {
                return Err(error(
                    token.start,
                    "typed interval tokens are not ordered within input",
                ));
            }
            previous = token.end;
        }
        // Build in a fresh caller-owned draft; any provenance failure discards it.
        self.source.push_str(image);
        self.source.push(' ');
        for &token in tokens {
            let original = Span {
                start: token.start,
                end: token.end,
            };
            let generated = projection.span(original)?;
            let (context, joint) = provenance(token)?;
            let kind = if let Kind::Interpolated(index) = token.kind {
                let slot = u32::try_from(self.opaque.len())
                    .map_err(|_| error(token.start, "typed interval has too many opaque inputs"))?;
                self.opaque.push(Opaque {
                    payload: OpaqueSource::Forwarded {
                        index: index.get(),
                        origin: PieceOrigin::Invocation,
                    },
                    original,
                    destination: generated,
                });
                Kind::interpolated(slot)?
            } else {
                token.kind
            };
            self.tokens.push(Token {
                kind,
                start: generated.start,
                end: generated.end,
            });
            self.origins.push(Origin {
                generated,
                original: Location {
                    file: source.file,
                    span: original,
                },
                context: (!matches!(token.kind, Kind::Interpolated(_))).then_some(context),
                introduction: ExpansionId(0),
                joint: Some(joint),
                kind: crate::expand::stream::OriginKind::Copied,
            });
        }
        Ok(projection)
    }
}
