//! Borrowed opaque-fragment inputs preserve parsed syntax while occurrences get fresh IDs.
use super::{
    ast::{Ast, Kind, NodeId, Syntax},
    attribute::{Attribute, Input},
    expression::fragment::FragmentKind,
    item::{AttributeCapture, Visibility},
    token_tree::{Forest, ForestBuilder, Span, error},
    transfer,
};
use crate::{
    lex::token::{Kind as TokenKind, LexError, Punctuation, Token},
    literal::{
        Value,
        storage::{Bytes, Storage, StoredValue},
        value::Float,
    },
};
use std::borrow::Cow;

pub struct ParsedSyntax {
    pub ast: Syntax<StoredValue>,
    pub literals: Storage,
}
impl ParsedSyntax {
    /// # Errors
    /// Rejects literal storage ranges outside the supplied source image.
    pub fn own(source: &str, ast: Ast<'_>) -> Result<Self, LexError> {
        let mut literals = Storage::default();
        let ast =
            ast.map_literals(|value| literals.store(source, &value).map_err(|e| error(0, e)))?;
        Ok(Self { ast, literals })
    }
}
#[derive(Clone, Copy)]
pub enum Nonterminal<'a> {
    Syntax {
        kind: FragmentKind,
        parsed: &'a ParsedSyntax,
    },
    Metadata {
        forest: &'a Forest,
        attribute: Attribute,
        expression: Option<&'a ParsedSyntax>,
    },
    Visibility {
        value: Visibility,
        path: Option<&'a ParsedSyntax>,
    },
}
#[derive(Clone, Copy)]
pub struct Splice<'a> {
    pub fragment: Nonterminal<'a>,
    pub original_source: &'a str,
    pub original: Span,
    pub destination: Span,
}

/// Lookup is independent of the parser's source lifetime. A crate registry can
/// retain one nonterminal while each emitted occurrence gets its own spans.
pub trait Inputs {
    /// Lets ordinary source parsers avoid even peeking for opaque tokens.
    fn is_empty(&self) -> bool;
    /// # Errors
    /// Rejects missing handles or invalid provenance for an opaque token.
    fn get(&self, token: Token) -> Result<Option<Splice<'_>>, LexError>;
    /// Defining edition of one token, independent of a macro's fragment grammar.
    /// Existing callers supply the ordinary Rust 2024 source profile. A mixed
    /// stream must provide its immutable provenance for the entire invocation.
    /// The parser only demands this lookup for edition-sensitive syntax.
    ///
    /// # Errors
    /// Rejects missing or invalid token provenance instead of choosing an edition.
    fn edition(&self, _token: Token) -> Result<crate::support::edition::Edition, LexError> {
        Ok(crate::support::edition::Edition::Rust2024)
    }
}
pub struct Slice<'a>(pub &'a [Splice<'a>]);
impl Inputs for Slice<'_> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn get(&self, token: Token) -> Result<Option<Splice<'_>>, LexError> {
        let TokenKind::Interpolated(index) = token.kind else {
            return Ok(None);
        };
        let splice = self
            .0
            .get(index.get() as usize)
            .ok_or_else(|| error(token.start, "missing opaque syntax input"))?;
        if splice.destination.start != token.start || splice.destination.end != token.end {
            return Err(error(
                token.start,
                "opaque token and provenance range disagree",
            ));
        }
        Ok(Some(*splice))
    }
}
pub const EMPTY_INPUTS: Slice<'static> = Slice(&[]);
impl<'capture> Splice<'capture> {
    /// Borrow a captured string value, decoding only a genuinely deferred literal.
    /// # Errors
    /// Rejects other root forms, literal kinds and invalid capture/storage ranges.
    pub(crate) fn string_literal(&self) -> Result<Cow<'capture, str>, LexError> {
        let invalid = || error(self.destination.start, "expected captured string literal");
        let Nonterminal::Syntax { kind, parsed } = self.fragment else {
            return Err(invalid());
        };
        if !matches!(
            kind,
            FragmentKind::Literal | FragmentKind::Expression | FragmentKind::Expression2021
        ) {
            return Err(invalid());
        }
        let node = parsed
            .ast
            .nodes
            .get(parsed.ast.root.get() as usize)
            .ok_or_else(invalid)?;
        self.span(node.span)?;
        let value = match node.kind {
            Kind::Literal(index) => {
                let value = *parsed
                    .ast
                    .literals
                    .get(index as usize)
                    .ok_or_else(invalid)?;
                if let StoredValue::String(Bytes::Source { start, end }) = value
                    && start != end
                {
                    self.span(Span { start, end })?;
                }
                parsed
                    .literals
                    .view(self.original_source, value)
                    .map_err(|message| error(self.destination.start, message))?
            }
            Kind::DeferredLiteral(token) => {
                self.span(Span {
                    start: token.start,
                    end: token.end,
                })?;
                crate::literal::interpret(self.original_source, token)?.ok_or_else(invalid)?
            }
            _ => return Err(invalid()),
        };
        let Value::String(bytes) = value else {
            return Err(invalid());
        };
        match bytes {
            Cow::Borrowed(bytes) => std::str::from_utf8(bytes)
                .map(Cow::Borrowed)
                .map_err(|_| invalid()),
            Cow::Owned(bytes) => String::from_utf8(bytes)
                .map(Cow::Owned)
                .map_err(|_| invalid()),
        }
    }
    /// Materialize only a parsed ordinary string root in an attribute forest.
    /// Its capture and copied source interval remain the provenance authority.
    pub(crate) fn append_attribute_string(
        &self,
        source: &str,
        builder: &mut ForestBuilder,
    ) -> Result<bool, LexError> {
        let Nonterminal::Syntax { kind, parsed } = self.fragment else {
            return Ok(false);
        };
        if !matches!(
            kind,
            FragmentKind::Literal | FragmentKind::Expression | FragmentKind::Expression2021
        ) {
            return Ok(false);
        }
        let node = parsed
            .ast
            .nodes
            .get(parsed.ast.root.get() as usize)
            .ok_or_else(|| error(self.destination.start, "missing captured attribute literal"))?;
        let (kind, span) = match node.kind {
            Kind::DeferredLiteral(token)
                if matches!(token.kind, TokenKind::String | TokenKind::RawString) =>
            {
                (
                    token.kind,
                    Span {
                        start: token.start,
                        end: token.end,
                    },
                )
            }
            Kind::Literal(index)
                if matches!(
                    parsed.ast.literals.get(index as usize),
                    Some(StoredValue::String(_))
                ) =>
            {
                let text = self
                    .original_source
                    .get(node.span.start as usize..node.span.end as usize)
                    .ok_or_else(|| {
                        error(
                            self.destination.start,
                            "captured string source span changed",
                        )
                    })?;
                let Some(kind) = ordinary_string_carrier(text) else {
                    return Ok(false);
                };
                (kind, node.span)
            }
            _ => return Ok(false),
        };
        let text = self
            .original_source
            .get(span.start as usize..span.end as usize)
            .ok_or_else(|| {
                error(
                    self.destination.start,
                    "captured string source span changed",
                )
            })?;
        if ordinary_string_carrier(text) != Some(kind) {
            return Ok(false);
        }
        let span = self.span(span)?;
        builder.push(
            source,
            Token {
                kind,
                start: span.start,
                end: span.end,
            },
        )?;
        Ok(true)
    }

    /// Lower an opaque path to attribute-path tokens from its typed segments.
    /// Punctuation spans cover the existing gap; identifier spans remain exact.
    /// No spelling is lexed and no expression/type grammar is rerun.
    ///
    /// # Errors
    /// Rejects non-path types, generic attribute segments or invalid provenance.
    pub(crate) fn append_attribute_path(
        &self,
        source: &str,
        builder: &mut ForestBuilder,
    ) -> Result<bool, LexError> {
        let Nonterminal::Syntax {
            kind: FragmentKind::Path | FragmentKind::Type,
            parsed,
        } = self.fragment
        else {
            return Ok(false);
        };
        let root = parsed.ast.node(parsed.ast.root);
        let Some((absolute, segments)) = parsed.ast.path_segments(parsed.ast.root) else {
            return Err(error(
                self.destination.start,
                "attribute requires a simple path",
            ));
        };
        let mut previous = root.span.start;
        for (index, node) in segments.iter().enumerate() {
            let Kind::Name(name, _) = parsed.ast.node(*node).kind else {
                return Err(error(
                    self.destination.start,
                    "attribute requires simple path segments",
                ));
            };
            if absolute || index > 0 {
                let gap = self.span(Span {
                    start: previous,
                    end: name.start,
                })?;
                builder.push(
                    source,
                    Token {
                        kind: TokenKind::Punctuation(Punctuation::ColonColon),
                        start: gap.start,
                        end: gap.end,
                    },
                )?;
            }
            let mapped = self.span(name)?;
            let kind = if self
                .original_source
                .get(name.start as usize..name.end as usize)
                .is_some_and(|s| s.starts_with("r#"))
            {
                TokenKind::RawIdentifier
            } else {
                TokenKind::Identifier
            };
            builder.push(
                source,
                Token {
                    kind,
                    start: mapped.start,
                    end: mapped.end,
                },
            )?;
            previous = name.end;
        }
        Ok(true)
    }
    #[must_use]
    pub const fn syntax_kind(&self) -> Option<FragmentKind> {
        if let Nonterminal::Syntax { kind, .. } = self.fragment {
            Some(kind)
        } else {
            None
        }
    }
    /// Classify only the captured syntax grammar; no constant value is proved.
    /// The pinned parser accepts literals, negated literals and actual blocks.
    pub(crate) fn begins_const_argument(&self) -> bool {
        use super::ast::{BlockFlavor, Unary};
        let Nonterminal::Syntax { kind, parsed } = self.fragment else {
            return false;
        };
        if !matches!(
            kind,
            FragmentKind::Literal
                | FragmentKind::Expression
                | FragmentKind::Expression2021
                | FragmentKind::Block
        ) {
            return false;
        }
        let Some(root) = parsed.ast.nodes.get(parsed.ast.root.get() as usize) else {
            return false;
        };
        match root.kind {
            Kind::Literal(_) | Kind::DeferredLiteral(_) => kind != FragmentKind::Block,
            Kind::Unary {
                op: Unary::Negate,
                operand,
            } => {
                kind != FragmentKind::Block
                    && parsed
                        .ast
                        .nodes
                        .get(operand.get() as usize)
                        .is_some_and(|node| {
                            matches!(node.kind, Kind::Literal(_) | Kind::DeferredLiteral(_))
                        })
            }
            Kind::Block {
                flavor: BlockFlavor::Plain,
                ..
            } => kind != FragmentKind::Literal,
            Kind::Block {
                flavor: BlockFlavor::Unsafe,
                ..
            } => matches!(
                kind,
                FragmentKind::Expression | FragmentKind::Expression2021
            ),
            _ => false,
        }
    }
    /// # Errors
    /// Rejects invalid origin spans and inconsistent copied spelling ranges.
    pub fn span(&self, span: Span) -> Result<Span, LexError> {
        if span == (Span { start: 0, end: 0 }) {
            return Ok(Span {
                start: self.destination.start,
                end: self.destination.start,
            });
        }
        if span.start < self.original.start || span.end > self.original.end || span.start > span.end
        {
            return Err(error(
                span.start,
                "opaque syntax span is outside its captured source",
            ));
        }
        let start = self
            .destination
            .start
            .checked_add(span.start - self.original.start);
        let end = self
            .destination
            .start
            .checked_add(span.end - self.original.start);
        match (start, end) {
            (Some(start), Some(end)) if end <= self.destination.end => Ok(Span { start, end }),
            _ => Err(error(
                span.start,
                "opaque syntax destination span is invalid",
            )),
        }
    }
    /// # Errors
    /// Reports malformed provenance, literal storage failures or syntax arena overflow.
    pub fn import<'s>(&self, source: &'s str, target: &mut Ast<'s>) -> Result<NodeId, LexError> {
        let Nonterminal::Syntax { parsed, .. } = self.fragment else {
            return Err(error(
                self.destination.start,
                "expected opaque syntax fragment",
            ));
        };
        self.import_syntax(parsed, source, target)
    }
    fn import_syntax<'s>(
        &self,
        parsed: &ParsedSyntax,
        source: &'s str,
        target: &mut Ast<'s>,
    ) -> Result<NodeId, LexError> {
        transfer::append(
            &parsed.ast,
            target,
            |span| self.span(span),
            |value| self.value(&parsed.literals, source, *value),
        )
    }
    /// # Errors
    /// Rejects a mismatched fragment or malformed restricted-visibility path.
    pub fn visibility<'s>(
        &self,
        source: &'s str,
        target: &mut Ast<'s>,
    ) -> Result<Visibility, LexError> {
        let Nonterminal::Visibility { value, path } = self.fragment else {
            return Err(error(
                self.destination.start,
                "expected opaque visibility fragment",
            ));
        };
        if matches!(value, Visibility::Restricted(_)) {
            let path = path
                .ok_or_else(|| error(self.destination.start, "missing opaque visibility path"))?;
            return self
                .import_syntax(path, source, target)
                .map(Visibility::Restricted);
        }
        Ok(value)
    }
    /// # Errors
    /// Rejects a mismatched fragment or invalid captured attribute spans.
    pub fn metadata(&self) -> Result<AttributeCapture, LexError> {
        let Nonterminal::Metadata {
            forest,
            mut attribute,
            ..
        } = self.fragment
        else {
            return Err(error(
                self.destination.start,
                "expected opaque metadata fragment",
            ));
        };
        let forest = forest.map_spans(&mut |span| self.span(span))?;
        attribute.span = self.span(attribute.span)?;
        if let Input::Doc(body) = &mut attribute.input {
            *body = self.span(*body)?;
        }
        Ok(AttributeCapture { forest, attribute })
    }
    fn bytes<'s>(
        &self,
        storage: &Storage,
        source: &'s str,
        bytes: Bytes,
    ) -> Result<Cow<'s, [u8]>, LexError> {
        if let Bytes::Source { start, end } = bytes {
            let span = self.span(Span { start, end })?;
            let bytes = source
                .as_bytes()
                .get(span.start as usize..span.end as usize)
                .ok_or_else(|| {
                    error(
                        span.start,
                        "opaque literal destination is outside its source",
                    )
                })?;
            return Ok(Cow::Borrowed(bytes));
        }
        let value = storage
            .view(self.original_source, StoredValue::String(bytes))
            .map_err(|e| error(self.original.start, e))?;
        let Value::String(bytes) = value else {
            return Err(error(self.original.start, "invalid stored byte payload"));
        };
        Ok(Cow::Owned(bytes.into_owned()))
    }
    fn value<'s>(
        &self,
        storage: &Storage,
        source: &'s str,
        value: StoredValue,
    ) -> Result<Value<'s>, LexError> {
        Ok(match value {
            StoredValue::Integer(value) => Value::Integer(value),
            StoredValue::Boolean(value) => Value::Boolean(value),
            StoredValue::Character(value) => Value::Character(value),
            StoredValue::Byte(value) => Value::Byte(value),
            StoredValue::String(bytes) => Value::String(self.bytes(storage, source, bytes)?),
            StoredValue::ByteString(bytes) => {
                Value::ByteString(self.bytes(storage, source, bytes)?)
            }
            StoredValue::CString(bytes) => Value::CString(self.bytes(storage, source, bytes)?),
            StoredValue::Float { decimal, suffix } => {
                let decimal = match self.bytes(storage, source, decimal)? {
                    Cow::Borrowed(bytes) => Cow::Borrowed(
                        std::str::from_utf8(bytes)
                            .map_err(|_| error(0, "invalid stored float text"))?,
                    ),
                    Cow::Owned(bytes) => Cow::Owned(
                        String::from_utf8(bytes)
                            .map_err(|_| error(0, "invalid stored float text"))?,
                    ),
                };
                Value::Float(Float { decimal, suffix })
            }
        })
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/interpolated_arguments.rs"]
mod const_arguments;

/// Select the lexical carrier of a typed string's genuine source interval.
/// This checks delimiters only; it neither parses a value nor admits another root kind.
fn ordinary_string_carrier(text: &str) -> Option<TokenKind> {
    let bytes = text.as_bytes();
    if bytes.first() == Some(&b'"') && bytes.last() == Some(&b'"') {
        return Some(TokenKind::String);
    }
    if bytes.first() != Some(&b'r') {
        return None;
    }
    let hashes = bytes
        .iter()
        .skip(1)
        .take_while(|&&byte| byte == b'#')
        .count();
    let prefix = hashes.checked_add(1)?;
    if bytes.get(prefix) != Some(&b'"') || bytes.len() < 2 * hashes + 3 {
        return None;
    }
    let close = bytes.len() - hashes - 1;
    (bytes.get(close) == Some(&b'"') && bytes[close + 1..].iter().all(|&byte| byte == b'#'))
        .then_some(TokenKind::RawString)
}
