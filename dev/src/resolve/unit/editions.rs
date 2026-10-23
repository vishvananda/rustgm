//! Lexical editions follow bounded, exact parent spans in retained source maps.
use super::Unit;
use crate::{
    expand::stream::{Location, Origin},
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
        location =
            parent(&source.origins, span).map_err(|failure| LexError { offset, ..failure })?;
    }
    Err(error(offset, "token edition origin chain exceeds 4096"))
}

/// A token inside a copied opaque slice retains its exact subrange. Never clamp
/// an invalid range to its neighbor or infer source identity from hygiene.
pub(crate) fn parent(origins: &[Origin], span: Span) -> Result<Location, LexError> {
    let index = origins.partition_point(|origin| origin.generated.end <= span.start);
    let origin = origins
        .get(index)
        .filter(|origin| {
            origin.generated.start <= span.start
                && span.start < span.end
                && span.end <= origin.generated.end
                && origin.original.span.start <= origin.original.span.end
                && origin.generated.end - origin.generated.start
                    == origin.original.span.end - origin.original.span.start
        })
        .ok_or_else(|| error(span.start, "missing exact token edition origin"))?;
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
