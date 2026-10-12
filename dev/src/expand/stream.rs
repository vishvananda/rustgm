//! A demanded expansion owns a spelling image and classified tokens, never relexed text.
use super::macros::{
    capture::CaptureId,
    input::Input,
    matcher::Matched,
    transcribe::{Origin as PieceOrigin, Piece, Transcription},
};
use crate::{
    lex::token::{Kind, LexError, Token},
    support::source::FileId,
    syntax::{
        cursor::TokenPosition,
        interpolation::Splice,
        token_tree::{Span, error},
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContextId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub file: FileId,
    pub span: Span,
}
#[derive(Clone, Copy, Debug)]
pub struct Origin {
    pub generated: Span,
    pub original: Location,
    /// None inherits hygiene from the corresponding offset in the parent image.
    pub context: Option<ContextId>,
    /// Synthetic tokens can specify spacing independently of their physical origin.
    pub joint: Option<bool>,
}
#[derive(Clone, Copy)]
pub struct Source<'a> {
    pub file: FileId,
    pub text: &'a str,
}
pub struct Environment<'a> {
    pub definition: Source<'a>,
    pub invocation: Source<'a>,
    pub context: ContextId,
    pub crate_id: u32,
}
pub struct Opaque {
    pub payload: OpaqueSource,
    pub original: Span,
    pub destination: Span,
}
#[derive(Clone, Copy)]
pub enum OpaqueSource {
    Capture(CaptureId),
    Forwarded { index: u32, origin: PieceOrigin },
}
#[derive(Default)]
pub struct Stream {
    pub source: String,
    pub tokens: Vec<Token>,
    pub origins: Vec<Origin>,
    pub opaque: Vec<Opaque>,
    pub(crate) registered: bool,
}
impl Stream {
    /// Copy demanded token spellings and opaque source slices once. Classified
    /// punctuation cannot accidentally glue across a substituted fragment.
    ///
    /// # Errors
    /// Rejects invalid captured ranges, source spans and arena overflow.
    pub fn transcribe(
        transcription: &Transcription,
        matched: &Matched,
        input: &Input,
        environment: &Environment<'_>,
    ) -> Result<Self, LexError> {
        let mut output = Self::default();
        for piece in &transcription.pieces {
            match *piece {
                Piece::Token { token, origin } => {
                    let (source, context) = match origin {
                        PieceOrigin::Definition => {
                            (environment.definition, Some(environment.context))
                        }
                        PieceOrigin::Invocation => (environment.invocation, None),
                    };
                    let span = Span {
                        start: token.start,
                        end: token.end,
                    };
                    let kind = if matches!(token.kind, Kind::Interpolated(_)) {
                        Kind::interpolated(
                            u32::try_from(output.opaque.len())
                                .map_err(|_| error(span.start, "too many forwarded inputs"))?,
                        )?
                    } else {
                        token.kind
                    };
                    let destination = output.token(
                        kind,
                        text(source.text, span)?,
                        Location {
                            file: source.file,
                            span,
                        },
                        context,
                    )?;
                    if let Kind::Interpolated(index) = token.kind {
                        output.opaque.push(Opaque {
                            payload: OpaqueSource::Forwarded {
                                index: index.get(),
                                origin,
                            },
                            original: span,
                            destination,
                        });
                    }
                }
                Piece::Opaque(capture) => {
                    let value = matched
                        .captures
                        .values
                        .get(capture.0 as usize)
                        .ok_or_else(|| error(0, "invalid opaque capture identity"))?;
                    let original = captured_span(input, value.start, value.end)?;
                    let index = u32::try_from(output.opaque.len())
                        .map_err(|_| error(original.start, "too many opaque expansion inputs"))?;
                    let source = environment.invocation;
                    let destination = output.token(
                        Kind::interpolated(index)?,
                        text(source.text, original)?,
                        Location {
                            file: source.file,
                            span: original,
                        },
                        None,
                    )?;
                    output.opaque.push(Opaque {
                        payload: OpaqueSource::Capture(capture),
                        original,
                        destination,
                    });
                }
                Piece::Crate(span) => {
                    output.token(
                        Kind::crate_root(environment.crate_id)?,
                        "$crate",
                        Location {
                            file: environment.definition.file,
                            span,
                        },
                        Some(environment.context),
                    )?;
                }
                Piece::Doc { inner, body, span } => {
                    output.token(
                        crate::syntax::token_tree::doc_kind(inner, span, body),
                        text(environment.definition.text, span)?,
                        Location {
                            file: environment.definition.file,
                            span,
                        },
                        Some(environment.context),
                    )?;
                }
            }
        }
        Ok(output)
    }
    fn token(
        &mut self,
        kind: Kind,
        spelling: &str,
        original: Location,
        context: Option<ContextId>,
    ) -> Result<Span, LexError> {
        let start = u32::try_from(self.source.len())
            .map_err(|_| error(original.span.start, "expansion source exceeds 32 bits"))?;
        self.source.push_str(spelling);
        let end = u32::try_from(self.source.len())
            .map_err(|_| error(original.span.start, "expansion source exceeds 32 bits"))?;
        let generated = Span { start, end };
        self.tokens.push(Token { kind, start, end });
        self.origins.push(Origin {
            generated,
            original,
            context,
            joint: None,
        });
        self.source.push(' ');
        Ok(generated)
    }
    /// Borrow opaque syntax for the parser without allocating or interpreting its tokens.
    ///
    /// # Errors
    /// Rejects invalid capture identities or non-syntax payloads at this interface.
    pub fn syntax_inputs<'a>(
        &'a self,
        matched: &'a Matched,
        original_source: &'a str,
    ) -> Result<Vec<Splice<'a>>, LexError> {
        self.syntax_inputs_with_parents(matched, original_source, &[], &[])
    }
    /// Remap forwarded opaque handles while keeping their original typed payload.
    ///
    /// # Errors
    /// Rejects inconsistent captures or missing parent input identities.
    pub fn syntax_inputs_with_parents<'a>(
        &'a self,
        matched: &'a Matched,
        original_source: &'a str,
        definition_inputs: &[Splice<'a>],
        invocation_inputs: &[Splice<'a>],
    ) -> Result<Vec<Splice<'a>>, LexError> {
        if self.registered {
            return Err(error(
                0,
                "registered expansion requires its persistent input registry",
            ));
        }
        let mut result = Vec::with_capacity(self.opaque.len());
        for opaque in &self.opaque {
            let capture = match opaque.payload {
                OpaqueSource::Forwarded { index, origin } => {
                    let inputs = match origin {
                        PieceOrigin::Definition => definition_inputs,
                        PieceOrigin::Invocation => invocation_inputs,
                    };
                    let parent = inputs.get(index as usize).ok_or_else(|| {
                        error(opaque.original.start, "missing parent opaque fragment")
                    })?;
                    if parent.destination != opaque.original {
                        return Err(error(
                            opaque.original.start,
                            "forwarded input source range disagrees",
                        ));
                    }
                    result.push(Splice {
                        destination: opaque.destination,
                        ..*parent
                    });
                    continue;
                }
                OpaqueSource::Capture(capture) => capture,
            };
            let value = matched
                .captures
                .values
                .get(capture.0 as usize)
                .ok_or_else(|| error(opaque.original.start, "invalid opaque capture identity"))?;
            let fragment = value.payload.nonterminal(value.fragment)?;
            result.push(Splice {
                fragment,
                original_source,
                original: opaque.original,
                destination: opaque.destination,
            });
        }
        Ok(result)
    }
    /// Locate the typed parent-origin mapping of a generated offset. A copied
    /// opaque slice may contain multiple contexts; None delegates to its parent.
    #[must_use]
    pub fn origin(&self, offset: u32) -> Option<Origin> {
        let index = self.origins.partition_point(|entry| {
            entry.generated.end <= offset && entry.generated.start != offset
        });
        self.origins
            .get(index)
            .copied()
            .filter(|entry| entry.generated.start <= offset && offset <= entry.generated.end)
    }
}
fn text(source: &str, span: Span) -> Result<&str, LexError> {
    source
        .get(span.start as usize..span.end as usize)
        .ok_or_else(|| error(span.start, "invalid expansion spelling range"))
}
fn captured_span(
    input: &Input,
    start: TokenPosition,
    end: TokenPosition,
) -> Result<Span, LexError> {
    let position = |p: TokenPosition| -> Result<u32, LexError> {
        if p.index == input.tokens.len() && p.offset == 0 {
            return Ok(input.tokens.last().map_or(0, |t| t.end));
        }
        let token = input
            .tokens
            .get(p.index)
            .ok_or_else(|| error(0, "invalid opaque token position"))?;
        let offset = token
            .start
            .checked_add(p.offset)
            .filter(|n| *n <= token.end)
            .ok_or_else(|| error(token.start, "invalid opaque partial-token offset"))?;
        Ok(offset)
    };
    let first = position(start)?;
    let last = if end.offset == 0 && end.index > start.index {
        input
            .tokens
            .get(end.index - 1)
            .ok_or_else(|| error(first, "invalid opaque end token"))?
            .end
    } else {
        position(end)?
    };
    if first > last {
        return Err(error(first, "reversed opaque capture range"));
    }
    Ok(Span {
        start: first,
        end: last,
    })
}
