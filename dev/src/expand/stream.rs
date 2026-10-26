//! A demanded expansion owns a spelling image and classified tokens, never relexed text.
#[path = "stream/image_interval.rs"]
mod source_interval;
use super::macros::{
    capture::CaptureId,
    input::Input,
    matcher::Matched,
    transcribe::{Origin as PieceOrigin, Piece, Transcription},
};
use super::marks::{ExpansionId, SyntaxContextId};
use crate::{
    lex::token::{Kind, LexError, Token},
    support::{edition::Edition, source::FileId},
    syntax::{
        cursor::TokenPosition,
        interpolation::Splice,
        token_tree::{Span, error},
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub file: FileId,
    pub span: Span,
}
/// Copied intervals preserve byte offsets; synthesized vocabulary has a real
/// source anchor and its producer's edition, without claiming copied bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginKind {
    Copied,
    Synthetic(Edition),
}
#[derive(Clone, Copy, Debug)]
pub struct Origin {
    pub generated: Span,
    pub original: Location,
    /// None inherits hygiene from the corresponding offset in the parent image.
    pub context: Option<SyntaxContextId>,
    /// Immediate literal introduction, independent of captured token hygiene.
    pub introduction: ExpansionId,
    /// Synthetic tokens can specify spacing independently of their physical origin.
    pub joint: Option<bool>,
    pub kind: OriginKind,
}
#[derive(Clone, Copy)]
pub struct Source<'a> {
    pub file: FileId,
    pub text: &'a str,
}
pub struct Environment<'a> {
    pub definition: Source<'a>,
    pub invocation: Source<'a>,
    pub context: SyntaxContextId,
    pub introduction: ExpansionId,
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
        Self::transcribe_with(transcription, matched, input, environment, |_, origin| {
            Ok((origin == PieceOrigin::Definition).then_some(environment.context))
        })
    }
    /// Crate expansion supplies the actual context of each definition token.
    /// Captures retain their own contexts; opaque slices can contain several.
    /// # Errors
    /// Rejects invalid source contexts, captured ranges and arena overflow.
    pub fn transcribe_with(
        transcription: &Transcription,
        matched: &Matched,
        input: &Input,
        environment: &Environment<'_>,
        mut contexts: impl FnMut(Location, PieceOrigin) -> Result<Option<SyntaxContextId>, LexError>,
    ) -> Result<Self, LexError> {
        let mut output = Self::default();
        for piece in &transcription.pieces {
            match *piece {
                Piece::Token { token, origin } => {
                    output.copy_token(token, origin, environment, &mut contexts)?;
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
                        ExpansionId(0),
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
                        contexts(
                            Location {
                                file: environment.definition.file,
                                span,
                            },
                            PieceOrigin::Definition,
                        )?,
                        environment.introduction,
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
                        contexts(
                            Location {
                                file: environment.definition.file,
                                span,
                            },
                            PieceOrigin::Definition,
                        )?,
                        environment.introduction,
                    )?;
                }
            }
        }
        Ok(output)
    }
    /// One demanded lexical token keeps its capture/definition provenance.
    fn copy_token(
        &mut self,
        token: Token,
        origin: PieceOrigin,
        environment: &Environment<'_>,
        contexts: &mut impl FnMut(Location, PieceOrigin) -> Result<Option<SyntaxContextId>, LexError>,
    ) -> Result<(), LexError> {
        let source = match origin {
            PieceOrigin::Definition => environment.definition,
            PieceOrigin::Invocation => environment.invocation,
        };
        let span = Span {
            start: token.start,
            end: token.end,
        };
        let kind = if matches!(token.kind, Kind::Interpolated(_)) {
            Kind::interpolated(
                u32::try_from(self.opaque.len())
                    .map_err(|_| error(span.start, "too many forwarded inputs"))?,
            )?
        } else {
            token.kind
        };
        let location = Location {
            file: source.file,
            span,
        };
        let opaque = matches!(token.kind, Kind::Interpolated(_));
        let context = if opaque {
            None
        } else {
            contexts(location, origin)?
        };
        let introduction = if origin == PieceOrigin::Definition {
            environment.introduction
        } else {
            ExpansionId(0)
        };
        let destination = self.token(
            kind,
            text(source.text, span)?,
            location,
            context,
            introduction,
        )?;
        if let Kind::Interpolated(index) = token.kind {
            self.opaque.push(Opaque {
                payload: OpaqueSource::Forwarded {
                    index: index.get(),
                    origin,
                },
                original: span,
                destination,
            });
        }
        Ok(())
    }
    fn token(
        &mut self,
        kind: Kind,
        spelling: &str,
        original: Location,
        context: Option<SyntaxContextId>,
        introduction: ExpansionId,
    ) -> Result<Span, LexError> {
        let generated = self.image(spelling, original, context, introduction)?;
        self.tokens.push(Token {
            kind,
            start: generated.start,
            end: generated.end,
        });
        Ok(generated)
    }
    /// A builtin can construct typed syntax over a diagnostic image directly.
    pub(crate) fn image(
        &mut self,
        spelling: &str,
        original: Location,
        context: Option<SyntaxContextId>,
        introduction: ExpansionId,
    ) -> Result<Span, LexError> {
        let start = u32::try_from(self.source.len())
            .map_err(|_| error(original.span.start, "expansion source exceeds 32 bits"))?;
        self.source.push_str(spelling);
        let end = u32::try_from(self.source.len())
            .map_err(|_| error(original.span.start, "expansion source exceeds 32 bits"))?;
        let generated = Span { start, end };
        self.origins.push(Origin {
            generated,
            original,
            context,
            introduction,
            joint: None,
            kind: OriginKind::Copied,
        });
        self.source.push(' ');
        Ok(generated)
    }
    /// An authenticated compiler-generated expansion supplies its actual selected
    /// source definition's edition. This data never grants macro allowances.
    pub(crate) fn synthetic_image(
        &mut self,
        spelling: &str,
        anchor: Location,
        context: SyntaxContextId,
        introduction: ExpansionId,
        edition: Edition,
    ) -> Result<Span, LexError> {
        if self.registered
            || context == SyntaxContextId::ROOT
            || introduction.0 == 0
            || anchor.span.start >= anchor.span.end
        {
            return Err(error(
                anchor.span.start,
                "synthetic vocabulary requires its actual source introduction",
            ));
        }
        let span = self.image(spelling, anchor, Some(context), introduction)?;
        self.origins
            .last_mut()
            .ok_or_else(|| error(anchor.span.start, "missing synthetic origin"))?
            .kind = OriginKind::Synthetic(edition);
        Ok(span)
    }
    /// Forward actual classified input without matching or manufacturing captures.
    pub(crate) fn forward_input(
        &mut self,
        input: &Input,
        source: Source<'_>,
        mut provenance: impl FnMut(Token) -> Result<(SyntaxContextId, bool), LexError>,
    ) -> Result<(), LexError> {
        let environment = Environment {
            definition: source,
            invocation: source,
            context: SyntaxContextId::ROOT,
            introduction: ExpansionId(0),
            crate_id: 0,
        };
        for &token in &input.tokens {
            let (context, joint) = provenance(token)?;
            self.copy_token(token, PieceOrigin::Invocation, &environment, &mut |_, _| {
                Ok(Some(context))
            })?;
            self.origins
                .last_mut()
                .ok_or_else(|| error(token.start, "forwarded token has no origin"))?
                .joint = Some(joint);
        }
        Ok(())
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
    /// Resolve one complete generated token to its immediate source provenance.
    /// A caller owning generated source files must continue through that file's
    /// origin map; hygiene context does not replace token source identity.
    ///
    /// # Errors
    /// Rejects missing origins and ranges that do not identify a complete token.
    pub fn token_origin(&self, token: Token) -> Result<Location, LexError> {
        let origin = self
            .origin(token.start)
            .filter(|origin| {
                origin.generated.start == token.start && origin.generated.end == token.end
            })
            .ok_or_else(|| error(token.start, "missing complete token origin"))?;
        Ok(origin.original)
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

const _: () = assert!(size_of::<Origin>() == 32);
