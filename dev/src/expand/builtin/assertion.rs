//! Assert condition parsing and transfer; source authority is owned by the resolver.
use super::choice::{Charge, Counted, syntax_payload};
use crate::{
    expand::{
        macros::input::Input,
        marks::SyntaxContextId,
        stream::{Source, Stream},
    },
    lex::token::{Kind as TokenKind, LexError, Punctuation, Token},
    syntax::{
        ast::{Ast, Node},
        cursor::TokenPosition,
        expression::fragment::{self, FragmentKind},
        interpolation::Inputs,
        token_tree::{Span, Tree, error},
        transfer,
    },
};
use std::ops::Range;

/// Non-clonable temporary ownership, never retained permission or a published AST.
pub(crate) struct Prepared<'source> {
    source: Source<'source>,
    tokens: &'source [Token],
    ast: Ast<'source>,
    condition: Span,
    message: Range<usize>,
}
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "private Assert result awaits its checked source-runtime consumer"
    )
)]
pub(crate) struct Condition {
    pub ast: Ast<'static>,
    pub stream: Stream,
    pub message: Range<usize>,
    pub parser_calls: usize,
}

/// Parse the actual condition once; only the later panic macro interprets a message.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "private Assert parser awaits its checked source-runtime consumer"
    )
)]
pub(crate) fn prepare<'source>(
    source: Source<'source>,
    input: &'source Input,
    inputs: &dyn Inputs,
    charge: &Charge<'_>,
) -> Result<Prepared<'source>, LexError> {
    charge(input.tokens.len().saturating_add(1), 0)?;
    let first = input
        .tokens
        .first()
        .ok_or_else(|| error(0, "assert requires a condition"))?;
    let bytes = input.tokens.iter().try_fold(0_usize, |sum, token| {
        token
            .end
            .checked_sub(token.start)
            .map(|width| sum.saturating_add(width as usize))
            .ok_or_else(|| error(token.start, "invalid assert input token span"))
    })?;
    charge(
        0,
        bytes
            .saturating_add(
                input
                    .tokens
                    .len()
                    .saturating_mul(size_of::<Node>() + size_of::<Tree>() + 64),
            )
            .saturating_mul(4),
    )?;
    let counted = Counted { inputs, charge };
    let parsed = fragment::parse_with_inputs(
        source.text,
        &input.tokens,
        FragmentKind::Expression,
        TokenPosition::default(),
        &counted,
    )?;
    let position = parsed.position;
    if position.offset != 0 || position.index == 0 || position.index > input.tokens.len() {
        return Err(error(first.start, "assert condition ended inside a token"));
    }
    let message = match input.tokens.get(position.index) {
        None => position.index..position.index,
        Some(token) if token.kind == TokenKind::Punctuation(Punctuation::Comma) => {
            position.index + 1..input.tokens.len()
        }
        Some(token) => {
            return Err(error(
                token.start,
                "assert condition requires a comma before its message",
            ));
        }
    };
    let condition = Span {
        start: first.start,
        end: input.tokens[position.index - 1].end,
    };
    Ok(Prepared {
        source,
        tokens: &input.tokens[..position.index],
        ast: parsed.ast,
        condition,
        message,
    })
}
impl Prepared<'_> {
    /// Consume once into a caller-owned draft; errors publish no package state.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private Assert transfer awaits its checked source-runtime consumer"
        )
    )]
    pub(crate) fn consume(
        self,
        prefix: &str,
        provenance: impl FnMut(Token) -> Result<(SyntaxContextId, bool), LexError>,
        charge: &Charge<'_>,
    ) -> Result<Condition, LexError> {
        let nodes = self.ast.nodes.len().saturating_add(self.ast.edges.len());
        let bytes = (self.condition.end - self.condition.start) as usize;
        charge(
            nodes
                .saturating_add(self.tokens.len())
                .saturating_add(self.ast.items.len())
                .saturating_add(self.ast.attributes.len())
                .saturating_add(self.ast.captures.len())
                .saturating_add(self.ast.generics.len())
                .saturating_add(self.ast.variants.len())
                .saturating_add(self.ast.use_trees.len())
                .saturating_add(1),
            0,
        )?;
        for forest in self
            .ast
            .captures
            .iter()
            .chain(self.ast.attributes.iter().map(|a| &a.forest))
        {
            charge(forest.nodes().len().saturating_add(1), 0)?;
        }
        charge(
            0,
            bytes
                .saturating_add(prefix.len())
                .saturating_mul(4)
                .saturating_add(self.tokens.len().saturating_mul(
                    4 * (size_of::<Token>()
                        + size_of::<crate::expand::stream::Origin>()
                        + size_of::<crate::expand::stream::Opaque>()),
                ))
                .saturating_add(syntax_payload(&self.ast).saturating_mul(4)),
        )?;
        transfer::validate_spans(&self.ast, |span| {
            if span.start < self.condition.start
                || span.end > self.condition.end
                || self
                    .source
                    .text
                    .get(span.start as usize..span.end as usize)
                    .is_none()
            {
                return Err(error(
                    span.start,
                    "assert typed span is outside its authenticated condition image",
                ));
            }
            Ok(span)
        })?;
        let mut stream = Stream::default();
        stream.source.push_str(prefix);
        let projection =
            stream.forward_interval(self.source, self.tokens, self.condition, provenance)?;
        let mut ast = Ast::default();
        let root = transfer::append(
            &self.ast,
            &mut ast,
            |span| projection.span(span),
            |value| {
                charge(1, literal_bytes(value).saturating_mul(4))?;
                Ok(own_literal(value))
            },
        )?;
        ast.root = root;
        Ok(Condition {
            ast,
            stream,
            message: self.message,
            parser_calls: 1,
        })
    }
}
fn own_literal(value: &crate::literal::Value<'_>) -> crate::literal::Value<'static> {
    use crate::literal::{Value, value::Float};
    match value {
        Value::Integer(value) => Value::Integer(*value),
        Value::Character(value) => Value::Character(*value),
        Value::Byte(value) => Value::Byte(*value),
        Value::Boolean(value) => Value::Boolean(*value),
        Value::Float(value) => Value::Float(Float {
            decimal: value.decimal.to_string().into(),
            suffix: value.suffix,
        }),
        Value::String(bytes) => Value::String(bytes.to_vec().into()),
        Value::ByteString(bytes) => Value::ByteString(bytes.to_vec().into()),
        Value::CString(bytes) => Value::CString(bytes.to_vec().into()),
    }
}
fn literal_bytes(value: &crate::literal::Value<'_>) -> usize {
    use crate::literal::Value;
    match value {
        Value::Float(value) => value.decimal.len(),
        Value::String(bytes) | Value::ByteString(bytes) | Value::CString(bytes) => bytes.len(),
        _ => 0,
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/assert_inputs.rs"]
mod tests;
