//! A captured field expression remains a typed root; no rendered-text parsing.
use super::{
    ast::{Kind, NodeId, Syntax},
    token_tree::{Span, error},
};
use crate::lex::token::LexError;

/// Visit components from the outside inward, retaining each actual syntax site.
pub(crate) fn visit<L>(
    ast: &Syntax<L>,
    mut node: NodeId,
    mut field: impl FnMut(NodeId, Span) -> Result<(), LexError>,
) -> Result<(), LexError> {
    // Each edge must descend in the append-only arena, bounding even forged input.
    loop {
        let current = ast
            .nodes
            .get(node.get() as usize)
            .ok_or_else(|| error(0, "offset field root disappeared"))?;
        match current.kind {
            Kind::Field { value, name } if value.get() < node.get() => {
                field(node, name)?;
                node = value;
            }
            Kind::Path {
                absolute: false,
                segments,
            } => {
                let [name] = ast.list(segments) else {
                    return Err(error(
                        current.span.start,
                        "offset field must be unqualified",
                    ));
                };
                if name.get() >= node.get() {
                    return Err(error(current.span.start, "offset field path edge changed"));
                }
                let Kind::Name(span, _) = ast.node(*name).kind else {
                    return Err(error(
                        current.span.start,
                        "offset field path requires a name",
                    ));
                };
                return field(node, span);
            }
            Kind::Name(span, _) => return field(node, span),
            _ => return Err(error(current.span.start, "expected offset field path")),
        }
    }
}

/// Check only the derived numeric path; an unchanged root was already visited.
/// No parsing or semantic permission is performed by this source comparison.
pub(crate) fn validate_source<L>(
    ast: &Syntax<L>,
    expression: NodeId,
    fields: NodeId,
    source: &str,
    mut charge: impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<(), LexError> {
    if fields == expression {
        return Ok(());
    }
    let invalid = || {
        error(
            0,
            "offset normalized fields differ from their source expression",
        )
    };
    let mut original = expression;
    let mut normalized = fields;
    let literal = loop {
        charge(2).map_err(|message| error(0, message))?;
        let old = ast.nodes.get(original.get() as usize).ok_or_else(invalid)?;
        let new = ast
            .nodes
            .get(normalized.get() as usize)
            .ok_or_else(invalid)?;
        match old.kind {
            Kind::Field { value, name } => {
                let Kind::Field {
                    value: next,
                    name: actual,
                } = new.kind
                else {
                    return Err(invalid());
                };
                if name != actual
                    || old.span != new.span
                    || value.get() >= original.get()
                    || next.get() >= normalized.get()
                {
                    return Err(invalid());
                }
                original = value;
                normalized = next;
            }
            Kind::Literal(_) | Kind::DeferredLiteral(_) => break old.span,
            _ => return Err(invalid()),
        }
    };
    literal_path(ast, normalized, literal, source, charge)
}

fn literal_path<L>(
    ast: &Syntax<L>,
    mut normalized: NodeId,
    literal: Span,
    source: &str,
    mut charge: impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<(), LexError> {
    let invalid = || {
        error(
            literal.start,
            "offset normalized fields differ from their source expression",
        )
    };
    let length = literal.end.checked_sub(literal.start).ok_or_else(invalid)?;
    // One delimiter scan plus one digit scan; node visits pay component work.
    charge((length as usize).saturating_mul(2).saturating_add(1))
        .map_err(|message| error(literal.start, message))?;
    let text = source
        .get(literal.start as usize..literal.end as usize)
        .ok_or_else(invalid)?;
    let mut parts = text.rsplit('.').peekable();
    let mut end = literal.end;
    while let Some(part) = parts.next() {
        if part.is_empty()
            || !part.bytes().all(|byte| byte.is_ascii_digit())
            || (part.len() > 1 && part.starts_with('0'))
        {
            return Err(invalid());
        }
        let length = u32::try_from(part.len()).map_err(|_| invalid())?;
        let start = end.checked_sub(length).ok_or_else(invalid)?;
        let component = Span { start, end };
        charge(1).map_err(|message| error(start, message))?;
        let node = ast
            .nodes
            .get(normalized.get() as usize)
            .ok_or_else(invalid)?;
        if node.span
            != (Span {
                start: literal.start,
                end,
            })
        {
            return Err(invalid());
        }
        if parts.peek().is_none() {
            return if matches!(node.kind, Kind::Name(span, None) if span == component) {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        let Kind::Field { value, name } = node.kind else {
            return Err(invalid());
        };
        if name != component || value.get() >= normalized.get() {
            return Err(invalid());
        }
        normalized = value;
        end = start.checked_sub(1).ok_or_else(invalid)?;
    }
    Err(invalid())
}
