//! A retained invocation exposes typed tokens and precomputed delimiter jumps.
use crate::{
    lex::token::{Kind, LexError, Punctuation as P, Token},
    syntax::{
        cursor::{TokenPosition, token_at},
        token_tree::{Delimiter, Forest, TreeKind, TreeRange, error},
    },
};
#[derive(Default)]
pub struct Input {
    pub tokens: Vec<Token>,
    pub after: Vec<usize>,
}
enum Work {
    Range(TreeRange),
    Close { token: Token, opening: usize },
}
impl Input {
    /// Flatten only a demanded macro capture, never an ordinary source file.
    ///
    /// # Errors
    /// Rejects invalid caller-supplied tree ranges.
    pub fn from_forest(forest: &Forest, range: TreeRange) -> Result<Self, LexError> {
        let mut result = Self::default();
        let mut work = vec![Work::Range(range)];
        while let Some(task) = work.pop() {
            match task {
                Work::Close { token, opening } => {
                    result.push(token);
                    result.after[opening] = result.tokens.len();
                }
                Work::Range(range) if range.start < range.end => {
                    let tree = forest.tree(range.start).ok_or_else(|| {
                        error(forest.offset(range.start), "invalid macro input tree")
                    })?;
                    let after = forest.after(range.start);
                    if after > range.end {
                        return Err(error(
                            tree.span.start,
                            "macro input range cuts through a delimiter",
                        ));
                    }
                    work.push(Work::Range(TreeRange {
                        start: after,
                        end: range.end,
                    }));
                    match tree.kind {
                        TreeKind::Token(kind) => result.push(Token {
                            kind,
                            start: tree.span.start,
                            end: tree.span.end,
                        }),
                        TreeKind::Doc { .. } => {
                            return Err(error(
                                tree.span.start,
                                "macro documentation input requires attribute desugaring",
                            ));
                        }
                        TreeKind::Group { delimiter, .. } => {
                            let (open, close) = delimiters(delimiter);
                            let opening = result.tokens.len();
                            result.push(Token {
                                kind: Kind::Punctuation(open),
                                start: tree.span.start,
                                end: tree.span.start + 1,
                            });
                            work.push(Work::Close {
                                token: Token {
                                    kind: Kind::Punctuation(close),
                                    start: tree.span.end - 1,
                                    end: tree.span.end,
                                },
                                opening,
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
        Ok(result)
    }
    fn push(&mut self, token: Token) {
        self.tokens.push(token);
        self.after.push(self.tokens.len());
    }
    /// # Errors
    /// Rejects invalid source spans and partial-token positions.
    pub fn token(&self, source: &str, position: TokenPosition) -> Result<Option<Token>, LexError> {
        token_at(source, &self.tokens, position)
    }
    /// # Errors
    /// Rejects positions that do not identify a token.
    pub fn advance(
        &self,
        source: &str,
        position: TokenPosition,
    ) -> Result<TokenPosition, LexError> {
        let token = self
            .token(source, position)?
            .ok_or_else(|| error(0, "cannot advance beyond macro input"))?;
        let original = self.tokens[position.index];
        Ok(if token.end == original.end {
            TokenPosition {
                index: position.index + 1,
                offset: 0,
            }
        } else {
            TokenPosition {
                index: position.index,
                offset: token.end - original.start,
            }
        })
    }
}
#[must_use]
pub const fn delimiters(delimiter: Delimiter) -> (P, P) {
    match delimiter {
        Delimiter::Parenthesis => (P::OpenParen, P::CloseParen),
        Delimiter::Bracket => (P::OpenBracket, P::CloseBracket),
        Delimiter::Brace => (P::OpenBrace, P::CloseBrace),
    }
}
