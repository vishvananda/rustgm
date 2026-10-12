//! Doc comments create attribute tokens once, with original spans kept beside them.
//! Ordinary captures borrow their original spelling image without copying.
use super::input::delimiters;
use crate::{
    lex::token::{Kind, LexError, Punctuation as P, Token},
    syntax::token_tree::{Forest, ForestBuilder, Span, TreeKind, TreeRange, error},
};
pub struct Origin {
    pub generated: Span,
    pub original: Span,
}
pub struct Prepared {
    pub source: String,
    pub forest: Forest,
    pub origins: Vec<Origin>,
}
struct Builder {
    source: String,
    forest: ForestBuilder,
    origins: Vec<Origin>,
}
enum Work {
    Range(TreeRange),
    Close { kind: P, span: Span },
}
/// # Errors
/// Reports invalid source spans, malformed caller-supplied forests and arena overflow.
pub fn if_needed(source: &str, forest: &Forest) -> Result<Option<Prepared>, LexError> {
    if !forest
        .nodes()
        .iter()
        .any(|tree| matches!(tree.kind, TreeKind::Doc { .. }))
    {
        return Ok(None);
    }
    let mut builder = Builder {
        source: String::new(),
        forest: ForestBuilder::default(),
        origins: Vec::new(),
    };
    let mut work = vec![Work::Range(forest.roots())];
    while let Some(task) = work.pop() {
        match task {
            Work::Close { kind, span } => {
                builder.token(Kind::Punctuation(kind), text(source, span)?, span)?;
            }
            Work::Range(range) if range.start < range.end => {
                let tree = forest
                    .tree(range.start)
                    .ok_or_else(|| error(0, "invalid macro tree"))?;
                let after = forest.after(range.start);
                work.push(Work::Range(TreeRange {
                    start: after,
                    end: range.end,
                }));
                match tree.kind {
                    TreeKind::Token(kind) => {
                        builder.token(kind, text(source, tree.span)?, tree.span)?;
                    }
                    TreeKind::Doc { inner, body } => builder.doc(source, tree.span, inner, body)?,
                    TreeKind::Group { delimiter, .. } => {
                        let (open, close) = delimiters(delimiter);
                        let first = Span {
                            start: tree.span.start,
                            end: tree.span.start + 1,
                        };
                        builder.token(Kind::Punctuation(open), text(source, first)?, first)?;
                        work.push(Work::Close {
                            kind: close,
                            span: Span {
                                start: tree.span.end - 1,
                                end: tree.span.end,
                            },
                        });
                        work.push(Work::Range(TreeRange {
                            start: range.start + 1,
                            end: after,
                        }));
                    }
                }
            }
            Work::Range(_) => {}
        }
    }
    let end = u32::try_from(builder.source.len())
        .map_err(|_| error(0, "macro spelling image exceeds 32 bits"))?;
    Ok(Some(Prepared {
        source: builder.source,
        forest: builder.forest.finish(end)?,
        origins: builder.origins,
    }))
}
fn text(source: &str, span: Span) -> Result<&str, LexError> {
    source
        .get(span.start as usize..span.end as usize)
        .ok_or_else(|| error(span.start, "invalid documentation source span"))
}
impl Builder {
    fn token(&mut self, kind: Kind, spelling: &str, original: Span) -> Result<(), LexError> {
        let start = u32::try_from(self.source.len())
            .map_err(|_| error(original.start, "macro spelling image exceeds 32 bits"))?;
        self.source.push_str(spelling);
        let end = u32::try_from(self.source.len())
            .map_err(|_| error(original.start, "macro spelling image exceeds 32 bits"))?;
        self.forest.push(&self.source, Token { kind, start, end })?;
        self.origins.push(Origin {
            generated: Span { start, end },
            original,
        });
        self.source.push(' ');
        Ok(())
    }
    fn doc(
        &mut self,
        source: &str,
        original: Span,
        inner: bool,
        body: Span,
    ) -> Result<(), LexError> {
        doc_tokens(inner, text(source, body)?, |kind, spelling| {
            self.token(kind, spelling, original)
        })
    }
}
pub(crate) fn doc_tokens(
    inner: bool,
    body: &str,
    mut emit: impl FnMut(Kind, &str) -> Result<(), LexError>,
) -> Result<(), LexError> {
    emit(Kind::Punctuation(P::Pound), "#")?;
    if inner {
        emit(Kind::Punctuation(P::Bang), "!")?;
    }
    emit(Kind::Punctuation(P::OpenBracket), "[")?;
    emit(Kind::Identifier, "doc")?;
    emit(Kind::Punctuation(P::Equal), "=")?;
    let mut hashes = 0;
    let mut after_quote = None;
    for byte in body.bytes() {
        if byte == b'"' {
            hashes = hashes.max(1);
            after_quote = Some(0);
        } else if byte == b'#'
            && let Some(count) = after_quote.as_mut()
        {
            *count += 1;
            hashes = hashes.max(*count + 1);
        } else {
            after_quote = None;
        }
    }
    let marks = "#".repeat(hashes);
    let spelling = format!("r{marks}\"{body}\"{marks}");
    emit(Kind::RawString, &spelling)?;
    emit(Kind::Punctuation(P::CloseBracket), "]")
}
impl Prepared {
    #[must_use]
    pub fn original_offset(&self, generated: u32) -> u32 {
        let index = self
            .origins
            .partition_point(|entry| entry.generated.end <= generated);
        self.origins
            .get(index)
            .or_else(|| self.origins.last())
            .map_or(0, |entry| entry.original.start)
    }
}
