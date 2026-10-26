//! Parser results retained by identity; failed rule alternatives reuse demanded fragments.
use super::{input::Input, model::Fragment};
use crate::support::hash::HashMap;
use crate::{
    lex::{
        keyword,
        token::{Kind as K, LexError, Punctuation as P, Token},
    },
    syntax::{
        attribute::Attribute,
        cursor::TokenPosition,
        expression::fragment::{self, FragmentKind as F},
        interpolation::{EMPTY_INPUTS, Inputs, Nonterminal},
        item::Visibility,
        token_tree::{Forest, error},
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CaptureId(pub u32);
pub use crate::syntax::interpolation::ParsedSyntax;
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
impl Payload {
    /// # Errors
    /// Rejects transparent tokens or inconsistent fragment/payload classes.
    pub fn nonterminal(&self, fragment: Fragment) -> Result<Nonterminal<'_>, LexError> {
        Ok(match (fragment, self) {
            (Fragment::Syntax(kind), Self::Syntax(parsed)) => Nonterminal::Syntax { kind, parsed },
            (
                Fragment::Metadata,
                Self::Metadata {
                    forest,
                    attribute,
                    expression,
                },
            ) => Nonterminal::Metadata {
                forest,
                attribute: *attribute,
                expression: expression.as_ref(),
            },
            (Fragment::Visibility, Self::Visibility { value, path }) => Nonterminal::Visibility {
                value: *value,
                path: path.as_ref(),
            },
            _ => return Err(error(0, "inconsistent opaque input payload")),
        })
    }
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
        self.memo = HashMap::default();
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
        self.demand_with_inputs(source, input, fragment, start, &EMPTY_INPUTS)
    }

    /// # Errors
    /// Reports fragment failures and invalid forwarded nonterminal identities.
    pub fn demand_with_inputs(
        &mut self,
        source: &str,
        input: &Input,
        fragment: Fragment,
        start: TokenPosition,
        inputs: &dyn Inputs,
    ) -> Result<CaptureId, LexError> {
        if let Some(id) = self.memo.get(&(fragment, start)) {
            return Ok(*id);
        }
        let (end, payload) = if let Some(end) =
            forwarded_end(source, input, fragment, start, inputs)?
        {
            (end, Payload::Transparent)
        } else {
            match fragment {
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
                    let parsed =
                        fragment::parse_with_inputs(source, &input.tokens, kind, start, inputs)?;
                    (parsed.position, Payload::Syntax(own(source, parsed.ast)?))
                }
                Fragment::Metadata => {
                    self.parser_calls += 1;
                    let parsed =
                        fragment::metadata_with_inputs(source, &input.tokens, start, inputs)?;
                    (
                        parsed.position,
                        Payload::Metadata {
                            forest: parsed.forest,
                            attribute: parsed.attribute,
                            expression: parsed
                                .expression
                                .map(|ast| own(source, ast))
                                .transpose()?,
                        },
                    )
                }
                Fragment::Visibility => {
                    self.parser_calls += 1;
                    let parsed =
                        fragment::visibility_with_inputs(source, &input.tokens, start, inputs)?;
                    (
                        parsed.position,
                        Payload::Visibility {
                            value: parsed.value,
                            path: parsed.ast.map(|ast| own(source, ast)).transpose()?,
                        },
                    )
                }
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

/// Conservative FIRST sets for already parsed nonterminals. Compatibility is
/// determined by their typed class, never the spelling inside an opaque token.
///
/// # Errors
/// Rejects missing opaque inputs before the matcher selects a parser branch.
pub fn may_begin_with_inputs(
    fragment: Fragment,
    token: Option<Token>,
    source: &str,
    inputs: &dyn Inputs,
) -> Result<bool, LexError> {
    let Some(token) = token else {
        return Ok(false);
    };
    if !matches!(token.kind, K::Interpolated(_)) {
        let edition = if matches!(fragment, Fragment::Syntax(_))
            && token.kind == K::Identifier
            && source.get(token.start as usize..token.end as usize) == Some("gen")
        {
            inputs.edition(token)?
        } else {
            crate::support::edition::Edition::Rust2024
        };
        return Ok(may_begin_in(fragment, Some(token), source, edition));
    }
    let splice = inputs
        .get(token)?
        .ok_or_else(|| error(token.start, "missing forwarded macro fragment"))?;
    let same = match (fragment, splice.fragment) {
        (Fragment::Tree, _)
        | (Fragment::Visibility, Nonterminal::Visibility { .. })
        | (
            Fragment::Metadata,
            Nonterminal::Metadata { .. }
            | Nonterminal::Syntax {
                kind: F::Path | F::Type,
                ..
            },
        ) => true,
        (Fragment::Syntax(expected), Nonterminal::Syntax { kind, .. }) => match expected {
            F::Block => kind == F::Block,
            F::Literal => kind == F::Literal,
            F::Item => kind == F::Item,
            F::Statement => matches!(
                kind,
                F::Statement
                    | F::Item
                    | F::Block
                    | F::Expression
                    | F::Expression2021
                    | F::Literal
                    | F::Path
            ),
            F::Expression | F::Expression2021 => matches!(
                kind,
                F::Expression | F::Expression2021 | F::Literal | F::Block | F::Path
            ),
            F::Pattern | F::PatternParameter => matches!(
                kind,
                F::Pattern
                    | F::PatternParameter
                    | F::Literal
                    | F::Expression
                    | F::Expression2021
                    | F::Path
            ),
            F::Path | F::Type => matches!(kind, F::Path | F::Type),
        },
        _ => false,
    };
    Ok(same)
}
fn forwarded_end(
    source: &str,
    input: &Input,
    fragment: Fragment,
    start: TokenPosition,
    inputs: &dyn Inputs,
) -> Result<Option<TokenPosition>, LexError> {
    if !input
        .tokens
        .get(start.index)
        .is_some_and(|t| matches!(t.kind, K::Interpolated(_)))
    {
        return Ok(None);
    }
    let token = input.token(source, start)?;
    let end = input.advance(source, start)?;
    let complete = input.tokens.get(end.index).is_none_or(|token| {
        matches!(
            token.kind,
            K::Punctuation(
                P::Comma
                    | P::Semicolon
                    | P::FatArrow
                    | P::CloseParen
                    | P::CloseBracket
                    | P::CloseBrace
            )
        )
    });
    if !complete
        && fragment == Fragment::Metadata
        && let Some(token) = token
        && inputs
            .get(token)?
            .is_some_and(|splice| matches!(splice.fragment, Nonterminal::Syntax { .. }))
    {
        return Ok(None);
    }
    if !complete
        && !matches!(
            fragment,
            Fragment::Metadata | Fragment::Visibility | Fragment::Tree
        )
    {
        return Ok(None);
    }
    if !may_begin_with_inputs(fragment, token, source, inputs)? {
        return Err(error(
            token.map_or(0, |t| t.start),
            "incompatible forwarded macro fragment",
        ));
    }
    Ok(Some(end))
}
fn own(source: &str, ast: crate::syntax::ast::Ast<'_>) -> Result<ParsedSyntax, LexError> {
    ParsedSyntax::own(source, ast)
}
#[must_use]
pub fn may_begin(fragment: Fragment, token: Option<Token>, source: &str) -> bool {
    may_begin_in(
        fragment,
        token,
        source,
        crate::support::edition::Edition::Rust2024,
    )
}
fn may_begin_in(
    fragment: Fragment,
    token: Option<Token>,
    source: &str,
    edition: crate::support::edition::Edition,
) -> bool {
    let Some(token) = token else {
        return false;
    };
    let spelling = source
        .get(token.start as usize..token.end as usize)
        .unwrap_or("");
    match fragment {
        Fragment::Identifier => {
            matches!(
                token.kind,
                K::Identifier | K::RawIdentifier | K::CrateRoot(_)
            ) && spelling != "_"
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
        Fragment::Syntax(kind) => syntax_start(kind, token, spelling, edition),
        Fragment::Visibility => {
            matches!(
                token.kind,
                K::Identifier | K::RawIdentifier | K::Punctuation(P::Comma)
            ) || syntax_start(F::Type, token, spelling, edition)
        }
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "one FIRST-token dispatch over the fragment grammar"
)]
fn syntax_start(
    kind: F,
    token: Token,
    spelling: &str,
    edition: crate::support::edition::Edition,
) -> bool {
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
    let path = matches!(token.kind, K::RawIdentifier | K::CrateRoot(_))
        || token.kind == K::Punctuation(P::ColonColon)
        || (token.kind == K::Identifier
            && spelling != "_"
            && (!keyword::Word::classify(spelling.as_bytes()).reserved_in(edition)
                || matches!(spelling, "self" | "Self" | "crate" | "super")));
    match kind {
        F::Block => token.kind == K::Punctuation(P::OpenBrace),
        F::Literal => literal || token.kind == K::Punctuation(P::Minus),
        F::Path => matches!(
            token.kind,
            K::Identifier | K::RawIdentifier | K::CrateRoot(_) | K::Punctuation(P::ColonColon)
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
                            | "try"
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
