//! Library-valued expansion retains a neutral identity, never a constructed source path.
use super::{
    Ast, Config, Expansion, Kind, LexError, List, MacroContext, Node, NodeId, Span, Value,
    crate_syntax, error,
};
use crate::support::library_types::LibraryValue;
/// # Errors
/// Rejects nonexpression syntax contexts and overflowing diagnostic source images.
pub fn option_environment(
    value: Option<&str>,
    grammar: MacroContext,
    config: &Config,
) -> Result<Expansion, LexError> {
    if !matches!(grammar, MacroContext::Expression | MacroContext::Statements) {
        return Err(error(0, "option_env requires an expression context"));
    }
    // The absent-value spelling only reserves a nonempty diagnostic origin range.
    // No parser or resolver derives the library identity from this image.
    let source = value.map_or_else(|| "option_env!".to_owned(), |value| format!("{value:?}"));
    let span = Span {
        start: 0,
        end: u32::try_from(source.len())
            .map_err(|_| error(0, "builtin output exceeds source limit"))?,
    };
    let mut ast = Ast::default();
    let value = value.map(|value| {
        ast.literals
            .push(Value::String(value.as_bytes().to_vec().into()));
        ast.nodes.push(Node {
            span,
            kind: Kind::Literal(0),
        });
        NodeId(0)
    });
    ast.root = NodeId(u32::from(value.is_some()));
    ast.nodes.push(Node {
        span,
        kind: Kind::LibraryValue {
            kind: LibraryValue::EnvironmentOption,
            value,
        },
    });
    if grammar == MacroContext::Statements {
        let tail = ast.root;
        ast.root = NodeId(ast.root.0 + 1);
        ast.nodes.push(Node {
            span,
            kind: Kind::Expansion {
                children: List { start: 0, end: 0 },
                tail: Some(tail),
            },
        });
    }
    let syntax = crate_syntax::configure("", ast, config)?;
    Ok(Expansion { source, syntax })
}
