//! Statement macro categories preserve opaque expressions and explicit delimiters.
use super::fragment::Parsed;
use super::{BlockFlavor, Context, Kind, LexError, NORMAL, NodeId, Parser};
use crate::{
    lex::token::Token,
    syntax::cursor::{Cursor, TokenPosition},
};

/// Frame one unbraced cfg_select RHS using Rust's statement-expression boundary.
/// This returns the actual expression AST for reuse in expression context.
/// A different requested output category still needs its distinct grammar check.
pub(crate) fn statement_expression<'s>(
    source: &'s str,
    tokens: &'s [Token],
    start: TokenPosition,
    inputs: &dyn crate::syntax::interpolation::Inputs,
) -> Result<(Parsed<'s>, bool), LexError> {
    let mut parser = Parser::with_cursor(Cursor::from_position(source, tokens, start)?, true);
    parser.inputs = inputs;
    parser.has_opaque = !inputs.is_empty();
    let root = parser.expr(
        0,
        super::Context {
            statement: true,
            ..NORMAL
        },
    )?;
    let mut value = root;
    while let Kind::Attributed { value: child, .. } | Kind::Label { value: child, .. } =
        parser.ast.node(value).kind
    {
        value = child;
    }
    // A brace-delimited macro call is not in rustc's expr_is_complete set.
    let complete = matches!(
        parser.ast.node(value).kind,
        Kind::Block {
            flavor: BlockFlavor::Plain | BlockFlavor::Unsafe | BlockFlavor::Const,
            ..
        } | Kind::If { .. }
            | Kind::Match { .. }
            | Kind::While { .. }
            | Kind::Loop { .. }
            | Kind::For { .. }
    );
    let position = parser.cursor.token_position()?;
    parser.ast.root = root;
    parser.ast.lookahead_high_water = parser.cursor.high_water;
    Ok((
        Parsed {
            ast: parser.ast,
            position,
        },
        complete,
    ))
}

impl Parser<'_, '_> {
    pub(super) fn statement_tail_macro(&mut self, mut node: NodeId) {
        while let Kind::Attributed { value, .. } = self.ast.node(node).kind {
            node = value;
        }
        if let Kind::Macro { capture, .. } = self.ast.node(node).kind
            && self.ast.captures[capture as usize]
                .tree(0)
                .is_some_and(|tree| {
                    matches!(
                        tree.kind,
                        crate::syntax::token_tree::TreeKind::Group {
                            delimiter: crate::syntax::token_tree::Delimiter::Brace,
                            ..
                        }
                    )
                })
        {
            self.statement_macro(node);
        }
    }

    pub(super) fn statement_macro(&mut self, mut node: NodeId) {
        while let Kind::Attributed { value, .. } = self.ast.node(node).kind {
            node = value;
        }
        if let Kind::Macro { context, .. } = &mut self.ast.nodes[node.get() as usize].kind {
            *context = crate::syntax::ast::MacroContext::Statements;
        }
    }
}
