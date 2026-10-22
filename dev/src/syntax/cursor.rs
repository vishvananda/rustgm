//! Bounded ordinary-source cursor; only explicit captures retain token forests.
use super::token_tree::{Forest, ForestBuilder, error};
use crate::lex::{
    Lexer,
    keyword::Word,
    token::{Kind, LexError, Punctuation, Token},
};

enum Input<'source> {
    Source(Lexer<'source>),
    Tokens(&'source [Token]),
}
/// One buffered significant token. Identifier words are classified once on
/// entry, so grammar keyword probes compare small integers, not spellings.
/// Input ordinals live in a parallel ring, keeping this hot entry at 16 bytes.
#[derive(Clone, Copy)]
struct Pending {
    token: Token,
    word: Word,
}
const VACANT: Pending = Pending {
    token: Token {
        kind: Kind::Whitespace,
        start: 0,
        end: 0,
    },
    word: Word::Other,
};
/// Grammar lookahead never exceeds three tokens. Source tokens are lexed in
/// batches ahead of that demand; the ring keeps room for grammar splits.
const SLOTS: usize = 32;
const MASK: usize = SLOTS - 1;
const BOUND: usize = 3;
const BATCH: usize = SLOTS - 8;

/// A parser fragment may stop partway through a compound punctuation token.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TokenPosition {
    pub index: usize,
    pub offset: u32,
}

pub struct Cursor<'source> {
    pub source: &'source str,
    input: Input<'source>,
    ring: [Pending; SLOTS],
    ordinals: [usize; SLOTS],
    head: usize,
    /// Tokens the grammar has demanded; only these count as lookahead.
    len: usize,
    /// Buffered tokens, including those lexed ahead of demand.
    ready: usize,
    /// No input remains after the buffered tokens.
    exhausted: bool,
    /// A failure met while lexing ahead, reported when its position is demanded.
    deferred: Option<LexError>,
    produced: usize,
    last_ordinal: usize,
    consumed_end: u32,
    last_kind: Option<Kind>,
    last_word: Word,
    /// The failure for the probe that is currently returning one.
    failure: Option<LexError>,
    pub high_water: usize,
}

impl<'source> Cursor<'source> {
    /// # Errors
    /// Applies PA1's source size and shebang checks.
    pub fn new(source: &'source str) -> Result<Self, LexError> {
        Self::source(source, true)
    }

    pub(crate) fn source(source: &'source str, strip_shebang: bool) -> Result<Self, LexError> {
        Ok(Self::with_input(
            source,
            Input::Source(Lexer::source(source, strip_shebang)?),
            0,
        ))
    }

    fn with_input(source: &'source str, input: Input<'source>, consumed_end: u32) -> Self {
        Self {
            source,
            input,
            ring: [VACANT; SLOTS],
            ordinals: [0; SLOTS],
            head: 0,
            len: 0,
            ready: 0,
            exhausted: false,
            deferred: None,
            produced: 0,
            last_ordinal: 0,
            consumed_end,
            last_kind: None,
            last_word: Word::Other,
            failure: None,
            high_water: 0,
        }
    }

    /// Consume already classified macro tokens with the ordinary bounded parser.
    /// Token spellings are borrowed; this path never invokes the lexer.
    #[must_use]
    pub fn from_tokens(source: &'source str, tokens: &'source [Token]) -> Self {
        Self::with_input(
            source,
            Input::Tokens(tokens),
            tokens.first().map_or(0, |t| t.start),
        )
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
            if cursor.len >= BOUND {
                return Err(error(token.start, "split token exceeds lookahead bound"));
            }
            let word = cursor.classify(token);
            cursor.push_back(token, word, position.index);
            let original = tokens[position.index];
            if token.end == original.end {
                break;
            }
            next.offset = token.end - original.start;
        }
        cursor.produced += 1;
        cursor.high_water = cursor.len;
        Ok(cursor)
    }

    /// # Errors
    /// Reports malformed source ranges or lexical failures while filling lookahead.
    pub fn token_position(&mut self) -> Result<TokenPosition, LexError> {
        let Some(next) = self.entry(0)? else {
            return Ok(TokenPosition {
                index: self.produced,
                offset: 0,
            });
        };
        let ordinal = self.ordinals[self.head & MASK];
        let offset = match self.input {
            Input::Tokens(tokens) => next.token.start.saturating_sub(tokens[ordinal].start),
            Input::Source(_) => 0,
        };
        Ok(TokenPosition {
            index: ordinal,
            offset,
        })
    }

    fn classify(&self, token: Token) -> Word {
        classify(self.source, token)
    }

    /// The next significant input token and its ordinal among all input tokens.
    fn next_significant(&mut self) -> Result<Option<(Token, usize)>, LexError> {
        match &mut self.input {
            Input::Source(lexer) => Ok(lexer
                .next_significant(&mut self.produced)?
                .map(|token| (token, self.produced - 1))),
            Input::Tokens(tokens) => loop {
                let Some(token) = tokens.get(self.produced).copied() else {
                    return Ok(None);
                };
                if token.start > token.end
                    || self
                        .source
                        .get(token.start as usize..token.end as usize)
                        .is_none()
                {
                    return Err(error(token.start, "invalid parser token source range"));
                }
                let ordinal = self.produced;
                self.produced += 1;
                if !matches!(
                    token.kind,
                    Kind::Whitespace | Kind::LineComment | Kind::BlockComment
                ) {
                    return Ok(Some((token, ordinal)));
                }
            },
        }
    }

    /// Buffer a token after every ready one, without demanding it.
    fn stage(&mut self, token: Token, word: Word, ordinal: usize) {
        let slot = (self.head + self.ready) & MASK;
        self.ring[slot] = Pending { token, word };
        self.ordinals[slot] = ordinal;
        self.ready += 1;
    }

    /// Construction-time split pieces are demanded as soon as they are staged.
    fn push_back(&mut self, token: Token, word: Word, ordinal: usize) {
        self.stage(token, word, ordinal);
        self.len += 1;
    }

    fn prepend(&mut self, token: Token) {
        self.head = (self.head + MASK) & MASK;
        self.ring[self.head] = Pending {
            token,
            word: Word::Other,
        };
        self.ordinals[self.head] = self.last_ordinal;
        self.len += 1;
        self.ready += 1;
    }

    /// The buffered entry `ahead` significant tokens on; only misses leave line.
    // Forced only without debug assertions: debug builds keep the call so
    // recursive-descent frames stay small enough for the nesting limit.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token lookahead probe; callgrind shows the call costs more than the probe"
        )
    )]
    fn entry(&mut self, ahead: usize) -> Result<Option<Pending>, LexError> {
        if ahead >= self.len {
            if !self.demand(ahead) {
                return Err(self.failure());
            }
            if ahead >= self.len {
                return Ok(None);
            }
        }
        Ok(Some(self.ring[(self.head + ahead) & MASK]))
    }

    /// Demand a token already lexed ahead, or fill. Kept small and out of line.
    #[inline(never)]
    fn demand(&mut self, ahead: usize) -> bool {
        if ahead < self.ready && ahead < BOUND {
            self.len = ahead + 1;
            self.high_water = self.high_water.max(self.len);
            return true;
        }
        self.fill(ahead)
    }

    /// Extend demanded lookahead through `ahead`, stopping early only at end of
    /// input. A failure is recorded for `failure`, keeping this result a flag.
    /// Lexing ahead never changes which tokens count as demanded lookahead, and
    /// a lexical failure is reported only once its position is demanded.
    #[cold]
    #[inline(never)]
    fn fill(&mut self, ahead: usize) -> bool {
        if ahead >= BOUND {
            self.failure = Some(error(
                self.consumed_end,
                "parser lookahead exceeds three tokens",
            ));
            return false;
        }
        while self.ready <= ahead && !self.exhausted {
            self.lex_ahead();
        }
        let demanded = self.ready.min(ahead + 1);
        if demanded > self.len {
            self.len = demanded;
            self.high_water = self.high_water.max(self.len);
        }
        if self.ready <= ahead
            && let Some(failure) = self.deferred.take()
        {
            self.failure = Some(failure);
            return false;
        }
        true
    }

    /// Buffer a batch of source tokens, or one macro token so a fragment parse
    /// never validates tokens beyond its own demand.
    fn lex_ahead(&mut self) {
        let source = self.source;
        let Input::Source(lexer) = &mut self.input else {
            match self.next_significant() {
                Ok(Some((token, ordinal))) => {
                    let word = self.classify(token);
                    self.stage(token, word, ordinal);
                }
                Ok(None) => self.exhausted = true,
                Err(failure) => {
                    self.deferred = Some(failure);
                    self.exhausted = true;
                }
            }
            return;
        };
        while self.ready < BATCH {
            match lexer.next_significant(&mut self.produced) {
                Ok(Some(token)) => {
                    let slot = (self.head + self.ready) & MASK;
                    self.ring[slot] = Pending {
                        token,
                        word: classify(source, token),
                    };
                    self.ordinals[slot] = self.produced - 1;
                    self.ready += 1;
                }
                Ok(None) => {
                    self.exhausted = true;
                    return;
                }
                Err(failure) => {
                    self.deferred = Some(failure);
                    self.exhausted = true;
                    return;
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn failure(&mut self) -> LexError {
        self.failure
            .take()
            .unwrap_or_else(|| error(self.consumed_end, "missing lookahead failure"))
    }

    /// # Errors
    /// Returns lexical errors or a request exceeding the three-token bound.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token lookahead probe; callgrind shows the call costs more than the probe"
        )
    )]
    pub fn peek(&mut self, ahead: usize) -> Result<Option<Token>, LexError> {
        Ok(self.entry(ahead)?.map(|pending| pending.token))
    }

    /// The classified word `ahead` tokens on; `Word::Other` for any
    /// non-identifier, unclassified identifier or end of input.
    ///
    /// # Errors
    /// Returns lexical errors or a request exceeding the three-token bound.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token lookahead probe; callgrind shows the call costs more than the probe"
        )
    )]
    pub fn peek_word(&mut self, ahead: usize) -> Result<Word, LexError> {
        Ok(self
            .entry(ahead)?
            .map_or(Word::Other, |pending| pending.word))
    }

    /// The next token together with its classified word.
    ///
    /// # Errors
    /// Returns lexical failures.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token lookahead probe; callgrind shows the call costs more than the probe"
        )
    )]
    pub fn current(&mut self) -> Result<Option<(Token, Word)>, LexError> {
        Ok(self.entry(0)?.map(|pending| (pending.token, pending.word)))
    }

    /// Consume the buffered front token. Callers have just observed one.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token lookahead probe; callgrind shows the call costs more than the probe"
        )
    )]
    fn advance(&mut self) {
        if self.len == 0 {
            return;
        }
        let slot = self.head & MASK;
        let pending = self.ring[slot];
        self.last_ordinal = self.ordinals[slot];
        self.head = (slot + 1) & MASK;
        self.len -= 1;
        self.ready -= 1;
        self.consumed_end = pending.token.end;
        self.last_kind = Some(pending.token.kind);
        self.last_word = pending.word;
    }

    /// # Errors
    /// Returns lexical failures.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token lookahead probe; callgrind shows the call costs more than the probe"
        )
    )]
    pub fn take(&mut self) -> Result<Option<Token>, LexError> {
        let token = self.peek(0)?;
        if token.is_some() {
            self.advance();
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

    /// The classified word of the most recently consumed token.
    #[must_use]
    pub const fn last_word(&self) -> Word {
        self.last_word
    }

    /// # Errors
    /// Returns lexical failures.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token lookahead probe; callgrind shows the call costs more than the probe"
        )
    )]
    pub fn at(&mut self, punctuation: Punctuation) -> Result<bool, LexError> {
        Ok(self
            .entry(0)?
            .is_some_and(|pending| pending.token.kind == Kind::Punctuation(punctuation)))
    }

    /// Consume the next token when it is `punctuation`.
    ///
    /// # Errors
    /// Returns lexical failures.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token lookahead probe; callgrind shows the call costs more than the probe"
        )
    )]
    pub fn eat(&mut self, punctuation: Punctuation) -> Result<bool, LexError> {
        let found = self.at(punctuation)?;
        if found {
            self.advance();
        }
        Ok(found)
    }

    /// # Errors
    /// Returns lexical failures.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token lookahead probe; callgrind shows the call costs more than the probe"
        )
    )]
    pub fn at_word(&mut self, word: Word) -> Result<bool, LexError> {
        Ok(self.peek_word(0)? == word)
    }

    /// Consume the next token when it is the identifier `word`.
    ///
    /// # Errors
    /// Returns lexical failures.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token lookahead probe; callgrind shows the call costs more than the probe"
        )
    )]
    pub fn eat_word(&mut self, word: Word) -> Result<bool, LexError> {
        let found = self.at_word(word)?;
        if found {
            self.advance();
        }
        Ok(found)
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
        if self.len + additions > BOUND {
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
        self.last_word = Word::Other;
        self.high_water = self.high_water.max(self.len);
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
            || self.len >= BOUND
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
        self.last_word = Word::Other;
        self.high_water = self.high_water.max(self.len);
        Ok(Token {
            kind: Kind::Punctuation(first),
            start: token.start,
            end: token.start + 1,
        })
    }
}

/// The keyword class of an identifier token; every other token is `Other`.
fn classify(source: &str, token: Token) -> Word {
    if token.kind != Kind::Identifier {
        return Word::Other;
    }
    source
        .as_bytes()
        .get(token.start as usize..token.end as usize)
        .map_or(Word::Other, Word::classify)
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
