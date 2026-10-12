//! Bounded ordinary-source cursor; only explicit captures retain token forests.
use super::token_tree::{Forest, ForestBuilder, error};
use crate::lex::{
    Lexer,
    token::{Kind, LexError, Punctuation, Token},
};
use std::collections::VecDeque;

enum Input<'source> {
    Source(Lexer<'source>),
    Tokens(&'source [Token]),
}
#[derive(Clone, Copy)]
struct Pending {
    token: Token,
    ordinal: usize,
}

/// A parser fragment may stop partway through a compound punctuation token.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TokenPosition {
    pub index: usize,
    pub offset: u32,
}

pub struct Cursor<'source> {
    pub source: &'source str,
    input: Input<'source>,
    pending: VecDeque<Pending>,
    produced: usize,
    last_ordinal: usize,
    consumed_end: u32,
    last_kind: Option<Kind>,
    pub high_water: usize,
}

impl<'source> Cursor<'source> {
    /// # Errors
    /// Applies PA1's source size and shebang checks.
    pub fn new(source: &'source str) -> Result<Self, LexError> {
        Self::source(source, true)
    }

    pub(crate) fn source(source: &'source str, strip_shebang: bool) -> Result<Self, LexError> {
        Ok(Self {
            source,
            input: Input::Source(Lexer::source(source, strip_shebang)?),
            produced: 0,
            last_ordinal: 0,
            pending: VecDeque::with_capacity(3),
            consumed_end: 0,
            last_kind: None,
            high_water: 0,
        })
    }

    /// Consume already classified macro tokens with the ordinary bounded parser.
    /// Token spellings are borrowed; this path never invokes the lexer.
    #[must_use]
    pub fn from_tokens(source: &'source str, tokens: &'source [Token]) -> Self {
        Self {
            source,
            input: Input::Tokens(tokens),
            pending: VecDeque::with_capacity(3),
            produced: 0,
            last_ordinal: 0,
            consumed_end: tokens.first().map_or(0, |t| t.start),
            last_kind: None,
            high_water: 0,
        }
    }

    /// Resume a macro fragment after a prior grammar split, without copying its tokens.
    ///
    /// # Errors
    /// Rejects invalid token positions and unsupported manually constructed splits.
    pub fn from_position(
        source: &'source str,
        tokens: &'source [Token],
        position: TokenPosition,
    ) -> Result<Self, LexError> {
        let mut cursor = Self::from_tokens(source, tokens);
        cursor.produced = position.index;
        cursor.consumed_end = tokens.get(position.index).map_or_else(
            || tokens.last().map_or(0, |t| t.end),
            |t| t.start.saturating_add(position.offset),
        );
        if position.offset == 0 {
            if position.index > tokens.len() {
                return Err(error(cursor.consumed_end, "invalid macro token position"));
            }
            return Ok(cursor);
        }
        let mut next = position;
        while let Some(token) = token_at(source, tokens, next)? {
            cursor.pending.push_back(Pending {
                token,
                ordinal: position.index,
            });
            if cursor.pending.len() > 3 {
                return Err(error(token.start, "split token exceeds lookahead bound"));
            }
            let original = tokens[position.index];
            if token.end == original.end {
                break;
            }
            next.offset = token.end - original.start;
        }
        cursor.produced += 1;
        cursor.high_water = cursor.pending.len();
        Ok(cursor)
    }

    /// # Errors
    /// Reports malformed source ranges or lexical failures while filling lookahead.
    pub fn token_position(&mut self) -> Result<TokenPosition, LexError> {
        self.peek(0)?;
        let Some(next) = self.pending.front() else {
            return Ok(TokenPosition {
                index: self.produced,
                offset: 0,
            });
        };
        let offset = match self.input {
            Input::Tokens(tokens) => next.token.start.saturating_sub(tokens[next.ordinal].start),
            Input::Source(_) => 0,
        };
        Ok(TokenPosition {
            index: next.ordinal,
            offset,
        })
    }

    fn next_input(&mut self) -> Result<Option<Pending>, LexError> {
        let token = match &mut self.input {
            Input::Source(lexer) => lexer.next_token()?,
            Input::Tokens(tokens) => {
                let token = tokens.get(self.produced).copied();
                if let Some(token) = token
                    && (token.start > token.end
                        || self
                            .source
                            .get(token.start as usize..token.end as usize)
                            .is_none())
                {
                    return Err(error(token.start, "invalid parser token source range"));
                }
                token
            }
        };
        let Some(token) = token else {
            return Ok(None);
        };
        let ordinal = self.produced;
        self.produced += 1;
        Ok(Some(Pending { token, ordinal }))
    }

    fn prepend(&mut self, token: Token) {
        self.pending.push_front(Pending {
            token,
            ordinal: self.last_ordinal,
        });
    }

    /// # Errors
    /// Returns lexical errors or a request exceeding the three-token bound.
    pub fn peek(&mut self, ahead: usize) -> Result<Option<Token>, LexError> {
        if ahead > 2 {
            return Err(error(
                self.consumed_end,
                "parser lookahead exceeds three tokens",
            ));
        }
        while self.pending.len() <= ahead {
            let Some(next) = self.next_input()? else {
                break;
            };
            if matches!(
                next.token.kind,
                Kind::Whitespace | Kind::LineComment | Kind::BlockComment
            ) {
                continue;
            }
            self.pending.push_back(next);
            self.high_water = self.high_water.max(self.pending.len());
        }
        Ok(self.pending.get(ahead).map(|pending| pending.token))
    }

    /// # Errors
    /// Returns lexical failures.
    pub fn take(&mut self) -> Result<Option<Token>, LexError> {
        self.peek(0)?;
        let pending = self.pending.pop_front();
        if let Some(pending) = pending {
            self.consumed_end = pending.token.end;
            self.last_kind = Some(pending.token.kind);
            self.last_ordinal = pending.ordinal;
        }
        Ok(pending.map(|pending| pending.token))
    }

    #[must_use]
    pub const fn end(&self) -> u32 {
        self.consumed_end
    }

    #[must_use]
    pub const fn last_kind(&self) -> Option<Kind> {
        self.last_kind
    }

    /// # Errors
    /// Returns lexical failures.
    pub fn at(&mut self, punctuation: Punctuation) -> Result<bool, LexError> {
        Ok(self
            .peek(0)?
            .is_some_and(|token| token.kind == Kind::Punctuation(punctuation)))
    }

    /// # Errors
    /// Returns lexical failures.
    pub fn word(&mut self, word: &str) -> Result<bool, LexError> {
        Ok(self
            .peek(0)?
            .is_some_and(|token| token.kind == Kind::Identifier && self.spelling(token) == word))
    }

    #[must_use]
    pub fn spelling(&self, token: Token) -> &'source str {
        &self.source[token.start as usize..token.end as usize]
    }

    /// # Errors
    /// Rejects mismatched or unterminated captured groups.
    pub fn capture(&mut self, first: Token) -> Result<Forest, LexError> {
        let mut builder = ForestBuilder::default();
        builder.push(self.source, first)?;
        while !builder.balanced() {
            let Some(token) = self.take()? else {
                return Err(error(first.start, "unclosed captured delimiter"));
            };
            builder.push(self.source, token)?;
        }
        builder.finish(self.end())
    }

    /// Advance over a group whose typed trees will be retained by its owner.
    ///
    /// # Errors
    /// Rejects a nonopening token, mismatched delimiters or premature end of input.
    pub fn skip_group(&mut self, first: Token) -> Result<(), LexError> {
        let Some((opening, false)) = super::token_tree::delimiter(first.kind) else {
            return Err(error(first.start, "expected opening delimiter"));
        };
        let mut open = vec![opening];
        while !open.is_empty() {
            let token = self
                .take()?
                .ok_or_else(|| error(first.start, "unclosed captured delimiter"))?;
            if let Some((delimiter, closing)) = super::token_tree::delimiter(token.kind) {
                if closing {
                    if open.pop() != Some(delimiter) {
                        return Err(error(token.start, "mismatched closing delimiter"));
                    }
                } else {
                    open.push(delimiter);
                }
            }
        }
        Ok(())
    }

    /// Tuple projection can split a float-shaped token (`x.0.1`) without
    /// lexing source again. Only decimal tuple-index pieces are accepted here.
    ///
    /// # Errors
    /// Returns lexical errors or an invalid/full-buffer projection split.
    pub fn projection(&mut self) -> Result<Token, LexError> {
        let token = self
            .take()?
            .ok_or_else(|| error(self.end(), "expected field name"))?;
        if token.kind != Kind::Float {
            return Ok(token);
        }
        let spelling = self.spelling(token);
        let Some((left, right)) = spelling.split_once('.') else {
            return Ok(token);
        };
        if left.is_empty()
            || !left
                .bytes()
                .chain(right.bytes())
                .all(|b| b.is_ascii_digit())
        {
            return Ok(token);
        }
        let dot = token.start
            + u32::try_from(left.len()).map_err(|_| error(token.start, "tuple index too long"))?;
        let additions = 1 + usize::from(!right.is_empty());
        if self.pending.len() + additions > 3 {
            return Err(error(
                token.start,
                "projection split exceeds lookahead bound",
            ));
        }
        if !right.is_empty() {
            self.prepend(Token {
                kind: Kind::Integer,
                start: dot + 1,
                end: token.end,
            });
        }
        self.prepend(Token {
            kind: Kind::Punctuation(Punctuation::Dot),
            start: dot,
            end: dot + 1,
        });
        self.consumed_end = dot;
        self.last_kind = Some(Kind::Integer);
        self.high_water = self.high_water.max(self.pending.len());
        Ok(Token {
            kind: Kind::Integer,
            start: token.start,
            end: dot,
        })
    }

    /// Split a compound punctuation token at a grammar-owned boundary. This
    /// operates on the typed token and spans, never relexes text.
    ///
    /// # Errors
    /// Rejects a nonmatching token or a full lookahead buffer.
    pub fn split(&mut self, first: Punctuation, rest: Punctuation) -> Result<Token, LexError> {
        let token = self
            .take()?
            .ok_or_else(|| error(self.end(), "expected compound token"))?;
        if token.end - token.start < 2
            || !matches!(token.kind, Kind::Punctuation(_))
            || self.pending.len() >= 3
        {
            return Err(error(token.start, "invalid compound token split"));
        }
        self.prepend(Token {
            kind: Kind::Punctuation(rest),
            start: token.start + 1,
            end: token.end,
        });
        self.consumed_end = token.start + 1;
        self.last_kind = Some(Kind::Punctuation(first));
        self.high_water = self.high_water.max(self.pending.len());
        Ok(Token {
            kind: Kind::Punctuation(first),
            start: token.start,
            end: token.start + 1,
        })
    }
}

/// Inspect the typed remainder of a token split by a grammar callback.
///
/// # Errors
/// Rejects invalid source ranges, positions or splits not produced by this cursor.
pub fn token_at(
    source: &str,
    tokens: &[Token],
    position: TokenPosition,
) -> Result<Option<Token>, LexError> {
    let Some(original) = tokens.get(position.index).copied() else {
        if position.index == tokens.len() && position.offset == 0 {
            return Ok(None);
        }
        return Err(error(0, "invalid macro token position"));
    };
    if original.start > original.end
        || source
            .get(original.start as usize..original.end as usize)
            .is_none()
    {
        return Err(error(original.start, "invalid parser token source range"));
    }
    if position.offset == 0 {
        return Ok(Some(original));
    }
    let start = original
        .start
        .checked_add(position.offset)
        .filter(|start| *start < original.end)
        .ok_or_else(|| error(original.start, "invalid split-token offset"))?;
    let mut end = original.end;
    let kind = match (original.kind, position.offset) {
        (Kind::Punctuation(Punctuation::ShiftRight), 1) => Kind::Punctuation(Punctuation::Greater),
        (Kind::Punctuation(Punctuation::ShiftRightEqual), 1) => {
            Kind::Punctuation(Punctuation::GreaterEqual)
        }
        (Kind::Punctuation(Punctuation::ShiftRightEqual), 2)
        | (Kind::Punctuation(Punctuation::GreaterEqual), 1) => {
            Kind::Punctuation(Punctuation::Equal)
        }
        (Kind::Punctuation(Punctuation::ShiftLeft), 1) => Kind::Punctuation(Punctuation::Less),
        (Kind::Punctuation(Punctuation::LeftArrow), 1) => Kind::Punctuation(Punctuation::Minus),
        (Kind::Punctuation(Punctuation::AndAnd), 1) => Kind::Punctuation(Punctuation::And),
        (Kind::Float, _) => {
            let tail = source
                .get(start as usize..end as usize)
                .ok_or_else(|| error(start, "invalid float projection split"))?;
            if tail.starts_with('.') {
                end = start + 1;
                Kind::Punctuation(Punctuation::Dot)
            } else if tail.bytes().all(|b| b.is_ascii_digit()) {
                Kind::Integer
            } else {
                return Err(error(start, "invalid float projection split"));
            }
        }
        _ => return Err(error(start, "unsupported partial macro token")),
    };
    Ok(Some(Token { kind, start, end }))
}
