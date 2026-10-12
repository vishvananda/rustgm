//! Parser results retained by identity; failed rule alternatives reuse demanded fragments.
use super::{input::Input, model::Fragment};
use crate::{
    lex::{
        keyword,
        token::{Kind as K, LexError, Punctuation as P, Token},
    },
    literal::storage::{Storage, StoredValue},
    syntax::{
        ast::Syntax,
        attribute::Attribute,
        cursor::TokenPosition,
        expression::fragment::{self, FragmentKind as F},
        item::Visibility,
        token_tree::{Forest, error},
    },
};
use std::collections::HashMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CaptureId(pub u32);
pub struct ParsedSyntax {
    pub ast: Syntax<StoredValue>,
    pub literals: Storage,
}
pub enum Payload {
    Transparent,
    Syntax(ParsedSyntax),
    Metadata {
        forest: Forest,
        attribute: Attribute,
        expression: Option<ParsedSyntax>,
    },
    Visibility {
        value: Visibility,
        path: Option<ParsedSyntax>,
    },
}
pub struct Captured {
    pub fragment: Fragment,
    pub start: TokenPosition,
    pub end: TokenPosition,
    pub payload: Payload,
}
#[derive(Default)]
pub struct Captures {
    pub values: Vec<Captured>,
    memo: HashMap<(Fragment, TokenPosition), CaptureId>,
    pub parser_calls: usize,
}
impl Captures {
    pub(super) fn finish(&mut self) {
        self.memo = HashMap::new();
    }
    /// # Errors
    /// Returns fragment grammar failures and capture identity overflow.
    pub fn demand(
        &mut self,
        source: &str,
        input: &Input,
        fragment: Fragment,
        start: TokenPosition,
    ) -> Result<CaptureId, LexError> {
        if let Some(id) = self.memo.get(&(fragment, start)) {
            return Ok(*id);
        }
        let (end, payload) = match fragment {
            Fragment::Identifier | Fragment::Lifetime => {
                (input.advance(source, start)?, Payload::Transparent)
            }
            Fragment::Tree => {
                let token = input
                    .token(source, start)?
                    .ok_or_else(|| error(0, "expected token tree"))?;
                let end = if matches!(
                    token.kind,
                    K::Punctuation(P::OpenParen | P::OpenBracket | P::OpenBrace)
                ) {
                    TokenPosition {
                        index: input.after[start.index],
                        offset: 0,
                    }
                } else {
                    input.advance(source, start)?
                };
                (end, Payload::Transparent)
            }
            Fragment::Syntax(kind) => {
                self.parser_calls += 1;
                let parsed = fragment::parse_at(source, &input.tokens, kind, start)?;
                (parsed.position, Payload::Syntax(own(source, parsed.ast)?))
            }
            Fragment::Metadata => {
                self.parser_calls += 1;
                let parsed = fragment::metadata(source, &input.tokens, start)?;
                (
                    parsed.position,
                    Payload::Metadata {
                        forest: parsed.forest,
                        attribute: parsed.attribute,
                        expression: parsed.expression.map(|ast| own(source, ast)).transpose()?,
                    },
                )
            }
            Fragment::Visibility => {
                self.parser_calls += 1;
                let parsed = fragment::visibility(source, &input.tokens, start)?;
                (
                    parsed.position,
                    Payload::Visibility {
                        value: parsed.value,
                        path: parsed.ast.map(|ast| own(source, ast)).transpose()?,
                    },
                )
            }
        };
        let id = CaptureId(
            u32::try_from(self.values.len())
                .map_err(|_| error(0, "too many captured fragments"))?,
        );
        // A captured syntax payload owns several arenas. Vec's minimum growth
        // would reserve four large records for the common single-fragment call.
        if self.values.is_empty() {
            self.values.reserve_exact(1);
        }
        self.values.push(Captured {
            fragment,
            start,
            end,
            payload,
        });
        self.memo.insert((fragment, start), id);
        Ok(id)
    }
}
fn own(source: &str, ast: crate::syntax::ast::Ast<'_>) -> Result<ParsedSyntax, LexError> {
    let mut literals = Storage::default();
    let ast = ast.map_literals(|value| {
        literals
            .store(source, value)
            .map_err(|message| error(0, message))
    })?;
    Ok(ParsedSyntax { ast, literals })
}
#[must_use]
pub fn may_begin(fragment: Fragment, token: Option<Token>, source: &str) -> bool {
    let Some(token) = token else {
        return false;
    };
    let spelling = source
        .get(token.start as usize..token.end as usize)
        .unwrap_or("");
    match fragment {
        Fragment::Identifier => {
            matches!(token.kind, K::Identifier | K::RawIdentifier) && spelling != "_"
        }
        Fragment::Lifetime => matches!(token.kind, K::Lifetime | K::RawLifetime),
        Fragment::Tree => !matches!(
            token.kind,
            K::Punctuation(P::CloseParen | P::CloseBracket | P::CloseBrace)
        ),
        Fragment::Metadata => matches!(
            token.kind,
            K::Identifier | K::RawIdentifier | K::Punctuation(P::ColonColon)
        ),
        Fragment::Syntax(kind) => syntax_start(kind, token, spelling),
        Fragment::Visibility => {
            matches!(
                token.kind,
                K::Identifier | K::RawIdentifier | K::Punctuation(P::Comma)
            ) || syntax_start(F::Type, token, spelling)
        }
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "one FIRST-token dispatch over the fragment grammar"
)]
fn syntax_start(kind: F, token: Token, spelling: &str) -> bool {
    let literal = matches!(
        token.kind,
        K::Integer
            | K::Float
            | K::Character
            | K::Byte
            | K::String
            | K::ByteString
            | K::CString
            | K::RawString
            | K::RawByteString
            | K::RawCString
    ) || (token.kind == K::Identifier && matches!(spelling, "true" | "false"));
    let path = token.kind == K::RawIdentifier
        || token.kind == K::Punctuation(P::ColonColon)
        || (token.kind == K::Identifier
            && spelling != "_"
            && (!keyword::reserved(spelling)
                || matches!(spelling, "self" | "Self" | "crate" | "super")));
    match kind {
        F::Block => token.kind == K::Punctuation(P::OpenBrace),
        F::Literal => literal || token.kind == K::Punctuation(P::Minus),
        F::Path => matches!(
            token.kind,
            K::Identifier | K::RawIdentifier | K::Punctuation(P::ColonColon)
        ),
        F::Type => {
            path || matches!(
                token.kind,
                K::Lifetime
                    | K::RawLifetime
                    | K::Punctuation(
                        P::OpenParen
                            | P::OpenBracket
                            | P::And
                            | P::AndAnd
                            | P::Star
                            | P::Bang
                            | P::Less
                            | P::ShiftLeft
                    )
            ) || (token.kind == K::Identifier
                && matches!(
                    spelling,
                    "_" | "for" | "unsafe" | "extern" | "fn" | "impl" | "dyn"
                ))
        }
        F::Pattern | F::PatternParameter => {
            path || literal
                || matches!(
                    token.kind,
                    K::Punctuation(
                        P::OpenParen
                            | P::OpenBracket
                            | P::And
                            | P::AndAnd
                            | P::Minus
                            | P::DotDot
                            | P::DotDotEqual
                            | P::Less
                            | P::ShiftLeft
                    )
                )
                || (kind == F::Pattern && token.kind == K::Punctuation(P::Or))
                || (token.kind == K::Identifier && matches!(spelling, "_" | "ref" | "mut"))
        }
        F::Expression | F::Expression2021 => {
            if kind == F::Expression2021
                && token.kind == K::Identifier
                && matches!(spelling, "_" | "const")
            {
                return false;
            }
            path || literal
                || matches!(
                    token.kind,
                    K::Punctuation(
                        P::OpenParen
                            | P::OpenBracket
                            | P::OpenBrace
                            | P::Minus
                            | P::Bang
                            | P::Star
                            | P::And
                            | P::AndAnd
                            | P::Or
                            | P::OrOr
                            | P::DotDot
                            | P::DotDotEqual
                            | P::Less
                            | P::ShiftLeft
                            | P::Pound
                    )
                )
                || (token.kind == K::Identifier
                    && matches!(
                        spelling,
                        "_" | "async"
                            | "break"
                            | "continue"
                            | "const"
                            | "for"
                            | "if"
                            | "loop"
                            | "match"
                            | "move"
                            | "return"
                            | "unsafe"
                            | "while"
                    ))
                || matches!(token.kind, K::Lifetime | K::RawLifetime)
        }
        F::Item | F::Statement => !matches!(
            token.kind,
            K::Punctuation(P::CloseParen | P::CloseBracket | P::CloseBrace)
        ),
    }
}
