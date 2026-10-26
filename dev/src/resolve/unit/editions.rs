//! Lexical editions follow bounded, exact parent spans in retained source maps.
use super::Unit;
use crate::{
    expand::stream::{Location, Origin, OriginKind},
    lex::token::{LexError, Token},
    support::{edition::Edition, source::FileId},
    syntax::token_tree::{Span, error},
};

pub(crate) fn token<'a>(
    file: FileId,
    token: Token,
    unit: impl Fn(FileId) -> Option<&'a Unit>,
) -> Result<Edition, LexError> {
    at(
        Location {
            file,
            span: Span {
                start: token.start,
                end: token.end,
            },
        },
        unit,
    )
}

/// A parsed ASCII introducer selects its own edition, even when the rest of
/// the syntax contains captured tokens from another source. No spelling scan.
pub(crate) fn introducer<'a>(
    file: FileId,
    span: Span,
    unit: impl Fn(FileId) -> Option<&'a Unit>,
) -> Result<Edition, LexError> {
    if span.start >= span.end {
        return Err(error(span.start, "missing syntax edition introducer"));
    }
    at(
        Location {
            file,
            span: Span {
                start: span.start,
                end: span.start + 1,
            },
        },
        unit,
    )
}

pub(crate) fn at<'a>(
    mut location: Location,
    unit: impl Fn(FileId) -> Option<&'a Unit>,
) -> Result<Edition, LexError> {
    let offset = location.span.start;
    for _ in 0..4096 {
        let source =
            unit(location.file).ok_or_else(|| error(offset, "missing token edition source"))?;
        let span = location.span;
        if span.start >= span.end
            || source
                .source
                .text()
                .get(span.start as usize..span.end as usize)
                .is_none()
        {
            return Err(error(offset, "invalid token edition span"));
        }
        if span.end as usize <= source.source.physical_len() {
            return Ok(source.edition);
        }
        let origin =
            covering(&source.origins, span).map_err(|failure| LexError { offset, ..failure })?;
        if let OriginKind::Synthetic(edition) = origin.kind {
            let anchor = unit(origin.original.file)
                .ok_or_else(|| error(offset, "missing synthetic edition anchor source"))?;
            if origin.introduction.0 == 0
                || origin
                    .context
                    .is_none_or(|c| c == crate::expand::marks::SyntaxContextId::ROOT)
                || origin.original.span.start >= origin.original.span.end
                || anchor
                    .source
                    .text()
                    .get(origin.original.span.start as usize..origin.original.span.end as usize)
                    .is_none()
                || source
                    .source
                    .text()
                    .get(origin.generated.start as usize..origin.generated.end as usize)
                    .is_none()
            {
                return Err(error(offset, "invalid synthetic token edition origin"));
            }
            return Ok(edition);
        }
        location = copied(origin, span).map_err(|failure| LexError { offset, ..failure })?;
    }
    Err(error(offset, "token edition origin chain exceeds 4096"))
}

/// A token inside a copied opaque slice retains its exact subrange. Never clamp
/// an invalid range to its neighbor or infer source identity from hygiene.
pub(crate) fn parent(origins: &[Origin], span: Span) -> Result<Location, LexError> {
    copied(covering(origins, span)?, span)
}
fn covering(origins: &[Origin], span: Span) -> Result<&Origin, LexError> {
    let index = origins.partition_point(|origin| origin.generated.end <= span.start);
    origins
        .get(index)
        .filter(|origin| {
            origin.generated.start <= span.start
                && span.start < span.end
                && span.end <= origin.generated.end
        })
        .ok_or_else(|| error(span.start, "missing exact token edition origin"))
}
fn copied(origin: &Origin, span: Span) -> Result<Location, LexError> {
    if origin.kind != OriginKind::Copied
        || origin.original.span.start > origin.original.span.end
        || origin.generated.end - origin.generated.start
            != origin.original.span.end - origin.original.span.start
    {
        return Err(error(span.start, "missing exact token edition origin"));
    }
    let start = origin.original.span.start + (span.start - origin.generated.start);
    Ok(Location {
        file: origin.original.file,
        span: Span {
            start,
            end: start + (span.end - span.start),
        },
    })
}

#[cfg(test)]
#[path = "editions/ancestry.rs"]
mod tests;
