//! Stringification walks classified tokens and retained opaque captures, never a lexer.
use crate::{
    lex::{
        keyword,
        token::{ExpansionId, Kind, LexError, Punctuation as P, Token},
    },
    support::source::FileId,
    syntax::token_tree::error,
};
use unicode_normalization::UnicodeNormalization;

/// Source spelling, token spacing provenance and stable opaque-token identities.
pub trait Sources {
    fn source(&self, file: FileId) -> &str;
    /// # Errors
    /// Rejects invalid or cyclic source provenance.
    fn joint(&self, file: FileId, token: Token) -> Result<bool, LexError>;
    /// # Errors
    /// Rejects an unknown capture handle.
    fn capture(&self, id: ExpansionId) -> Result<(FileId, &[Token]), LexError>;
}

#[derive(Clone, Copy)]
struct Printed {
    file: FileId,
    token: Token,
    joint: bool,
}
struct Frame<'a> {
    file: FileId,
    tokens: &'a [Token],
    index: usize,
    previous: Option<Printed>,
    group: Option<Token>,
    padding: bool,
}
impl<'a> Frame<'a> {
    fn new(file: FileId, tokens: &'a [Token]) -> Self {
        Self {
            file,
            tokens,
            index: 0,
            previous: None,
            group: None,
            padding: false,
        }
    }
}

/// Produce the builtin's value, with explicit stacks for delimiters and opaque inputs.
///
/// # Errors
/// Rejects inconsistent token streams, missing captures and excessive output work.
#[expect(
    clippy::too_many_lines,
    reason = "one iterative token-tree traversal owns delimiter and opaque-capture continuations"
)]
pub fn render(file: FileId, tokens: &[Token], sources: &impl Sources) -> Result<String, LexError> {
    let mut output = Layout::default();
    let mut frames = vec![Frame::new(file, tokens)];
    let mut work = 0_u32;
    while let Some(frame) = frames.last_mut() {
        let Some(&token) = frame.tokens.get(frame.index) else {
            if frame.group.is_some() {
                return Err(error(0, "unclosed stringify delimiter"));
            }
            if frames.len() > 1 {
                output.end()?;
            }
            frames.pop();
            continue;
        };
        work += 1;
        if work > 1_000_000 || frames.len() > 4096 {
            return Err(error(token.start, "stringify work limit exceeded"));
        }
        // Reborrow after checking the stack's depth.
        let Some(frame) = frames.last_mut() else {
            break;
        };
        frame.index += 1;
        if matches!(
            token.kind,
            Kind::Punctuation(P::CloseParen | P::CloseBracket | P::CloseBrace)
        ) {
            let opening = frame
                .group
                .ok_or_else(|| error(token.start, "unexpected stringify delimiter"))?;
            let expected = match opening.kind {
                Kind::Punctuation(P::OpenParen) => P::CloseParen,
                Kind::Punctuation(P::OpenBracket) => P::CloseBracket,
                _ => P::CloseBrace,
            };
            if token.kind != Kind::Punctuation(expected) {
                return Err(error(token.start, "mismatched stringify delimiter"));
            }
            output.end()?;
            if frame.padding {
                output.space(-4);
            }
            spelling(&mut output, frame.file, token, sources)?;
            if expected == P::CloseBrace {
                output.end()?;
            }
            let previous = Printed {
                file: frame.file,
                token: opening,
                joint: sources.joint(frame.file, token)?,
            };
            let next = frame.index;
            frames.pop();
            let parent = frames
                .last_mut()
                .ok_or_else(|| error(token.start, "missing stringify parent"))?;
            parent.index = next;
            parent.previous = Some(previous);
            continue;
        }
        let current = Printed {
            file: frame.file,
            token,
            joint: sources.joint(frame.file, token)?,
        };
        if let Some(previous) = frame.previous
            && needs_space(previous, current, sources)?
        {
            output.space(0);
        }
        frame.previous = Some(current);
        if let Kind::Interpolated(id) = token.kind {
            // An invisible nonterminal is one tree to its enclosing sequence,
            // including empty visibility. Its original tokens retain their own spacing.
            frame.previous = Some(Printed {
                joint: false,
                ..current
            });
            let (file, tokens) = sources.capture(id)?;
            output.begin(false, 0);
            frames.push(Frame::new(file, tokens));
        } else if matches!(
            token.kind,
            Kind::Punctuation(P::OpenParen | P::OpenBracket | P::OpenBrace)
        ) {
            if token.kind == Kind::Punctuation(P::OpenBrace) {
                output.begin(true, 4);
            }
            spelling(&mut output, frame.file, token, sources)?;
            let padding = token.kind == Kind::Punctuation(P::OpenBrace)
                && !current.joint
                && frame
                    .tokens
                    .get(frame.index)
                    .is_some_and(|t| t.kind != Kind::Punctuation(P::CloseBrace));
            if padding {
                output.space(0);
            }
            output.begin(false, 0);
            let child = Frame {
                group: Some(token),
                index: frame.index,
                padding,
                ..Frame::new(frame.file, frame.tokens)
            };
            frames.push(child);
        } else {
            spelling(&mut output, frame.file, token, sources)?;
            if matches!(
                token.kind,
                Kind::OuterLineDoc | Kind::InnerLineDoc | Kind::OuterBlockDoc | Kind::InnerBlockDoc
            ) {
                output.hardbreak();
            }
        }
    }
    output.finish()
}

fn text(file: FileId, token: Token, sources: &impl Sources) -> Result<&str, LexError> {
    sources
        .source(file)
        .get(token.start as usize..token.end as usize)
        .ok_or_else(|| error(token.start, "invalid stringify source span"))
}
fn spelling(
    output: &mut Layout,
    file: FileId,
    token: Token,
    sources: &impl Sources,
) -> Result<(), LexError> {
    let spelling = text(file, token, sources)?;
    output.word(
        spelling,
        matches!(
            token.kind,
            Kind::Identifier | Kind::RawIdentifier | Kind::Lifetime | Kind::RawLifetime
        ),
    );
    Ok(())
}
fn identifier(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Identifier
            | Kind::RawIdentifier
            | Kind::CrateRoot(_)
            | Kind::Punctuation(P::Underscore)
    )
}
fn punctuation(kind: Kind) -> bool {
    matches!(kind, Kind::Punctuation(p) if !matches!(p, P::OpenParen | P::OpenBracket | P::OpenBrace | P::Underscore))
}
fn needs_space(left: Printed, right: Printed, sources: &impl Sources) -> Result<bool, LexError> {
    let a = left.token.kind;
    let b = right.token.kind;
    if matches!(a, Kind::InnerLineDoc | Kind::OuterLineDoc) {
        return Ok(false);
    }
    if left.joint {
        return Ok(identifier(a) && identifier(b));
    }
    if (a == Kind::Punctuation(P::Dot) && !punctuation(b))
        || (a == Kind::Punctuation(P::Dollar) && identifier(b))
        || (!punctuation(a) && matches!(b, Kind::Punctuation(P::Comma | P::Semicolon | P::Dot)))
        || (a == Kind::Punctuation(P::Pound) && b == Kind::Punctuation(P::OpenBracket))
    {
        return Ok(false);
    }
    if identifier(a) && matches!(b, Kind::Punctuation(P::Bang | P::OpenParen)) {
        let name = text(left.file, left.token, sources)?;
        if a == Kind::RawIdentifier
            || !keyword::reserved(name)
            || (b == Kind::Punctuation(P::OpenParen) && matches!(name, "fn" | "Self" | "pub"))
        {
            return Ok(false);
        }
    }
    Ok(true)
}

// Layout is demanded only by stringify. One spelling arena and flat operations
// permit two linear passes, without allocating strings or recursive boxes per token.
#[derive(Clone, Copy)]
enum Operation {
    Word {
        start: usize,
        end: usize,
    },
    Space {
        following: i64,
        offset: i64,
    },
    Begin {
        width: i64,
        consistent: bool,
        indent: i64,
    },
    End,
    Hard,
}
struct Measure {
    begin: Option<usize>,
    previous: Option<usize>,
}
#[derive(Default)]
struct Layout {
    text: String,
    operations: Vec<Operation>,
    total: i64,
    groups: Vec<Measure>,
}
impl Layout {
    fn word(&mut self, text: &str, normalize: bool) {
        let start = self.text.len();
        if normalize && !text.is_ascii() {
            self.text.extend(text.nfc());
        } else {
            self.text.push_str(text);
        }
        let end = self.text.len();
        self.total += i64::try_from(end - start).unwrap_or(i64::MAX / 2);
        self.operations.push(Operation::Word { start, end });
    }
    fn close_break(&mut self) {
        if let Some(previous) = self.groups.last_mut().and_then(|g| g.previous.take())
            && let Operation::Space { following, .. } = &mut self.operations[previous]
        {
            *following += self.total;
        }
    }
    fn space(&mut self, offset: i64) {
        self.close_break();
        let index = self.operations.len();
        self.operations.push(Operation::Space {
            following: -self.total,
            offset,
        });
        self.total += 1;
        if let Some(group) = self.groups.last_mut() {
            group.previous = Some(index);
        } else {
            self.groups.push(Measure {
                begin: None,
                previous: Some(index),
            });
        }
    }
    fn hardbreak(&mut self) {
        self.close_break();
        self.operations.push(Operation::Hard);
        self.total += 1_000_000_000;
    }
    fn begin(&mut self, consistent: bool, indent: i64) {
        let index = self.operations.len();
        self.operations.push(Operation::Begin {
            width: -self.total,
            consistent,
            indent,
        });
        self.groups.push(Measure {
            begin: Some(index),
            previous: None,
        });
    }
    fn end(&mut self) -> Result<(), LexError> {
        self.close_break();
        let group = self
            .groups
            .pop()
            .ok_or_else(|| error(0, "missing stringify layout group"))?;
        let index = group
            .begin
            .ok_or_else(|| error(0, "unbalanced stringify layout group"))?;
        if let Operation::Begin { width, .. } = &mut self.operations[index] {
            *width += self.total;
        }
        self.operations.push(Operation::End);
        Ok(())
    }
    fn finish(mut self) -> Result<String, LexError> {
        self.close_break();
        let mut output = String::with_capacity(self.text.len());
        let mut remaining = 78_i64;
        let mut indent = 0_i64;
        let mut pending = 0_i64;
        let mut groups = Vec::new();
        for operation in self.operations {
            match operation {
                Operation::Word { start, end } => {
                    let count = usize::try_from(pending)
                        .map_err(|_| error(0, "invalid stringify indentation"))?;
                    output.extend(std::iter::repeat_n(' ', count));
                    pending = 0;
                    output.push_str(&self.text[start..end]);
                    remaining -= i64::try_from(end - start)
                        .map_err(|_| error(0, "stringify output too large"))?;
                }
                Operation::Begin {
                    width,
                    consistent,
                    indent: offset,
                } => {
                    let fits = width <= remaining;
                    groups.push((fits, consistent, indent));
                    if !fits {
                        indent += offset;
                    }
                }
                Operation::End => {
                    let (_, _, previous) = groups
                        .pop()
                        .ok_or_else(|| error(0, "missing stringify layout parent"))?;
                    indent = previous;
                }
                Operation::Hard => {
                    output.push('\n');
                    pending = indent;
                    remaining = (78 - pending).max(60);
                }
                Operation::Space { following, offset } => {
                    let (fits, consistent, _) = groups.last().copied().unwrap_or((false, false, 0));
                    if fits || (!consistent && following <= remaining) {
                        pending += 1;
                        remaining -= 1;
                    } else {
                        output.push('\n');
                        pending = indent + offset;
                        remaining = (78 - pending).max(60);
                    }
                }
            }
        }
        Ok(output)
    }
}
