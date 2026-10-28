//! Detached argument expressions retain token grammar and ordinary identifier rules.
use super::{Error, FileId, NodeId};
use crate::{
    lex::token::{Kind as TokenKind, Punctuation},
    resolve::unit::Unit,
    syntax::{ast::Kind, token_tree::error},
};
use std::borrow::Cow;
use unicode_normalization::UnicodeNormalization;
pub(super) struct Inputs<'a> {
    pub values: Vec<NodeId>,
    pub names: Vec<Option<Cow<'a, str>>>,
}
pub(super) fn collect<'a>(
    unit: &'a Unit,
    file: FileId,
    root: NodeId,
    capture: u32,
    pay: &crate::expand::builtin::choice::Charge<'_>,
) -> Result<Inputs<'a>, Error> {
    let Kind::Expansion {
        children,
        tail: None,
    } = unit.configured.ast.node(root).kind
    else {
        return Err(Error::at(
            file,
            0,
            "missing actual formatting argument list",
        ));
    };
    let ast = &unit.configured.ast;
    let arguments = ast.list(children);
    let count = arguments.len().saturating_sub(1);
    pay(
        count.saturating_add(1),
        count.saturating_mul(
            size_of::<NodeId>() + size_of::<Option<Cow<'_, str>>>() + size_of::<Option<&str>>(),
        ),
    )
    .map_err(|e| Error::at(file, e.offset, e.message))?;
    let mut result = Inputs {
        values: Vec::with_capacity(count),
        names: Vec::with_capacity(count),
    };
    let forest = &ast.captures[capture as usize];
    for &argument in arguments.iter().skip(1) {
        let named = if let Kind::Binary {
            op: Punctuation::Equal,
            left,
            right,
        } = ast.node(argument).kind
        {
            if let Kind::Name(span, _) = ast.node(left).kind {
                pay(
                    2 + forest.nodes().len().checked_ilog2().unwrap_or(0) as usize,
                    0,
                )
                .map_err(|e| Error::at(file, e.offset, e.message))?;
                let index = forest
                    .nodes()
                    .partition_point(|tree| tree.span.start < span.start);
                let index = u32::try_from(index)
                    .map_err(|_| Error::at(file, span.start, "formatting token index overflow"))?;
                let token = forest.token(index);
                if token.is_some_and(|token| {
                    token.start == span.start
                        && token.end == span.end
                        && matches!(token.kind, TokenKind::Identifier | TokenKind::RawIdentifier)
                }) && forest.punctuation(index.saturating_add(1), Punctuation::Equal)
                {
                    Some((span, right))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        let (name, value) = if let Some((span, value)) = named {
            let spelling = unit
                .spelling(span)
                .map_err(|e| Error::at(file, e.offset, e.message))?;
            let spelling = spelling.strip_prefix("r#").unwrap_or(spelling);
            pay(spelling.len().saturating_mul(2).saturating_add(1), 0)
                .map_err(|e| Error::at(file, e.offset, e.message))?;
            let name = if unicode_normalization::is_nfc(spelling) {
                Cow::Borrowed(spelling)
            } else {
                let bytes = spelling.nfc().map(char::len_utf8).sum::<usize>();
                pay(spelling.len().saturating_add(bytes), bytes)
                    .map_err(|e| Error::at(file, e.offset, e.message))?;
                let mut normalized = String::with_capacity(bytes);
                normalized.extend(spelling.nfc());
                Cow::Owned(normalized)
            };
            (Some(name), value)
        } else {
            (None, argument)
        };
        result.values.push(value);
        result.names.push(name);
    }
    Ok(result)
}
pub(super) fn direct_literal<'a>(
    unit: &Unit,
    capture: u32,
    registry: &crate::expand::registry::Registry,
    pay: &crate::expand::builtin::choice::Charge<'_>,
    source: impl Fn(FileId) -> Option<&'a Unit>,
) -> Result<bool, crate::lex::token::LexError> {
    use crate::syntax::interpolation::{Inputs as _, Nonterminal};
    pay(1, 0)?;
    let forest = &unit.configured.ast.captures[capture as usize];
    let range = forest
        .children(0)
        .ok_or_else(|| error(0, "formatting invocation lost its delimiters"))?;
    let Some(token) = forest.token(range.start) else {
        return Ok(false);
    };
    if matches!(token.kind, TokenKind::String | TokenKind::RawString) {
        return Ok(true);
    }
    if !matches!(token.kind, TokenKind::Interpolated(_)) {
        return Ok(false);
    }
    pay(4, 0)?;
    let view = registry.view(|id| source(id).map(|unit| unit.source.text()));
    let splice = view
        .get(token)?
        .ok_or_else(|| error(token.start, "missing actual formatting literal capture"))?;
    let Nonterminal::Syntax { parsed, .. } = splice.fragment else {
        return Ok(false);
    };
    let node = parsed.ast.node(parsed.ast.root);
    splice.span(node.span)?;
    Ok(matches!(
        node.kind,
        Kind::Literal(_)
            | Kind::DeferredLiteral(crate::lex::token::Token {
                kind: TokenKind::String | TokenKind::RawString,
                ..
            })
    ))
}
