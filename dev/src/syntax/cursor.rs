//! Bounded ordinary-source cursor; only explicit captures retain token forests.
use super::token_tree::{Forest, ForestBuilder, error};
use crate::lex::{
    Lexer,
    token::{Kind, LexError, Punctuation, Token},
};
use std::collections::VecDeque;

pub struct Cursor<'source> {
    pub source: &'source str,
    lexer: Lexer<'source>,
    pending: VecDeque<Token>,
    consumed_end: u32,
    last_kind: Option<Kind>,
    pub high_water: usize,
}

impl<'source> Cursor<'source> {
    /// # Errors
    /// Applies PA1's source size and shebang checks.
    pub fn new(source: &'source str) -> Result<Self, LexError> {
        Ok(Self {
            source,
            lexer: Lexer::new(source)?,
            pending: VecDeque::with_capacity(3),
            consumed_end: 0,
            last_kind: None,
            high_water: 0,
        })
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
            let Some(token) = self.lexer.next_token()? else {
                break;
            };
            if matches!(
                token.kind,
                Kind::Whitespace | Kind::LineComment | Kind::BlockComment
            ) {
                continue;
            }
            self.pending.push_back(token);
            self.high_water = self.high_water.max(self.pending.len());
        }
        Ok(self.pending.get(ahead).copied())
    }

    /// # Errors
    /// Returns lexical failures.
    pub fn take(&mut self) -> Result<Option<Token>, LexError> {
        self.peek(0)?;
        let token = self.pending.pop_front();
        if let Some(token) = token {
            self.consumed_end = token.end;
            self.last_kind = Some(token.kind);
        }
        Ok(token)
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
            self.pending.push_front(Token {
                kind: Kind::Integer,
                start: dot + 1,
                end: token.end,
            });
        }
        self.pending.push_front(Token {
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
        self.pending.push_front(Token {
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
