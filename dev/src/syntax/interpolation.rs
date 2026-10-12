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
impl Splice<'_> {
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
        let mut root = parsed.ast.node(parsed.ast.root);
        if let Kind::Type {
            flavor: super::ast::TypeFlavor::Path,
            elements,
        } = root.kind
        {
            let [path] = parsed.ast.list(elements) else {
                return Err(error(self.destination.start, "invalid captured path type"));
            };
            root = parsed.ast.node(*path);
        }
        let Kind::Path { absolute, segments } = root.kind else {
            return Err(error(
                self.destination.start,
                "attribute requires a simple path",
            ));
        };
        let mut previous = root.span.start;
        for (index, node) in parsed.ast.list(segments).iter().enumerate() {
            let Kind::Name(name) = parsed.ast.node(*node).kind else {
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
