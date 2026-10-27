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
