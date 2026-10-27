//! A template expression joins typed operands into one canonical identifier.
use super::{Binding, CaptureId, Fragment, Kind, LexError, List, Payload, Piece, Writer, error};
use crate::{
    lex::token::{Kind as TokenKind, Token},
    literal::storage::StoredValue,
    syntax::{
        ast::Kind as Syntax,
        expression::fragment::FragmentKind,
        interpolation::{Nonterminal, ParsedSyntax},
        token_tree::Span,
    },
};
use std::borrow::Cow;
use unicode_normalization::UnicodeNormalization;

impl Writer<'_> {
    pub(super) fn concat(&mut self, operands: List, span: Span) -> Result<(), LexError> {
        let mut spelling = String::new();
        for id in self.definition.list(operands) {
            let node = self.definition.node(*id);
            self.tick(node.span.start)?;
            let symbol = match node.kind {
                Kind::Atom(atom) => self.names.get(atom.name.ok_or_else(|| {
                    error(node.span.start, "concat atom lost its canonical symbol")
                })?),
                Kind::Substitute { name, .. } => {
                    let variable = self.variables.get(&name).copied().ok_or_else(|| {
                        error(node.span.start, "concat references an unknown metavariable")
                    })?;
                    let Binding::Capture(capture) = self.variable(variable)? else {
                        return Err(error(
                            node.span.start,
                            "concat metavariable is still repeating at this depth",
                        ));
                    };
                    self.concat_capture(capture)?
                }
                _ => return Err(error(node.span.start, "invalid typed concat operand")),
            };
            let bytes = symbol.len();
            if bytes > 1_000_000_usize.saturating_sub(self.output.work) {
                return Err(error(
                    span.start,
                    "macro transcription work budget exceeded",
                ));
            }
            spelling.push_str(symbol);
            self.output.work += bytes;
        }
        let canonical = if spelling.is_ascii() || unicode_normalization::is_nfc(&spelling) {
            Cow::Borrowed(spelling.as_str())
        } else {
            Cow::Owned(spelling.nfc().collect::<String>())
        };
        let mut characters = canonical.chars();
        if !characters
            .next()
            .is_some_and(|ch| ch == '_' || unicode_ident::is_xid_start(ch))
            || !characters.all(unicode_ident::is_xid_continue)
        {
            return Err(error(span.start, "concat result is not a valid identifier"));
        }
        self.charge(span.start, canonical.len())?;
        let name = self
            .names
            .intern_plain(&canonical)
            .map_err(|e| error(span.start, e))?;
        self.output.pieces.push(Piece::Identifier { name, span });
        Ok(())
    }

    fn concat_capture(&self, id: CaptureId) -> Result<&str, LexError> {
        let capture = self
            .matched
            .captures
            .values
            .get(id.0 as usize)
            .ok_or_else(|| error(0, "concat capture is outside its actual match"))?;
        if !matches!(
            capture.fragment,
            Fragment::Identifier | Fragment::Tree | Fragment::Syntax(FragmentKind::Literal)
        ) {
            return Err(error(0, "concat requires ident, literal or tt fragments"));
        }
        if let (Fragment::Syntax(FragmentKind::Literal), Payload::Syntax(parsed)) =
            (capture.fragment, &capture.payload)
        {
            return literal_symbol(parsed, self.source);
        }
        let token = self
            .input
            .token(self.source, capture.start)?
            .ok_or_else(|| error(0, "concat capture is empty"))?;
        if self.input.advance(self.source, capture.start)? != capture.end {
            return Err(error(
                token.start,
                "concat requires a single captured operand",
            ));
        }
        if matches!(token.kind, TokenKind::Interpolated(_)) {
            if capture.fragment != Fragment::Syntax(FragmentKind::Literal) {
                return Err(error(
                    token.start,
                    "concat cannot inspect an opaque token-tree operand",
                ));
            }
            let splice = self
                .inputs
                .get(token)?
                .ok_or_else(|| error(token.start, "concat lost its forwarded literal"))?;
            let Nonterminal::Syntax {
                kind: FragmentKind::Literal,
                parsed,
            } = splice.fragment
            else {
                return Err(error(
                    token.start,
                    "concat requires a forwarded literal fragment",
                ));
            };
            return literal_symbol(parsed, splice.original_source);
        }
        token_symbol(self.source, token)
    }
}

fn literal_symbol<'a>(parsed: &ParsedSyntax, source: &'a str) -> Result<&'a str, LexError> {
    let node = parsed.ast.node(parsed.ast.root);
    if let Syntax::DeferredLiteral(token) = node.kind {
        return match token.kind {
            TokenKind::Integer => crate::literal::integer_symbol(source, token),
            TokenKind::String => token_symbol(source, token),
            _ => Err(error(
                token.start,
                "concat literal must be an unsuffixed string or integer",
            )),
        };
    }
    let Syntax::Literal(index) = node.kind else {
        return Err(error(
            node.span.start,
            "concat requires a string or unsigned-spelling integer literal",
        ));
    };
    let kind = match parsed.ast.literals.get(index as usize) {
        Some(StoredValue::String(_)) => TokenKind::String,
        Some(StoredValue::Integer(value)) if value.suffix.is_none() => TokenKind::Integer,
        _ => {
            return Err(error(
                node.span.start,
                "concat literal must be an unsuffixed string or integer",
            ));
        }
    };
    let token = Token {
        kind,
        start: node.span.start,
        end: node.span.end,
    };
    if kind == TokenKind::Integer {
        return source
            .get(token.start as usize..token.end as usize)
            .ok_or_else(|| error(token.start, "concat literal is outside its source"));
    }
    token_symbol(source, token)
}

fn token_symbol(source: &str, token: Token) -> Result<&str, LexError> {
    let spelling = source
        .get(token.start as usize..token.end as usize)
        .ok_or_else(|| error(token.start, "concat token is outside its source"))?;
    match token.kind {
        TokenKind::Identifier => Ok(spelling),
        TokenKind::String => spelling
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .ok_or_else(|| error(token.start, "concat requires an unsuffixed cooked string")),
        _ => Err(error(
            token.start,
            "concat does not support this captured token",
        )),
    }
}
