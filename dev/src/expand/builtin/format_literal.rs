//! Literal formatting owns decoded data; its language owner is selected in semantics.
use super::{
    Ast, Config, Expansion, Kind, LexError, List, MacroContext, Node, NodeId, Span, Value,
    crate_syntax, error,
};
use crate::{
    support::{library_types::LibraryValue, names::NameId},
    syntax::ast::Syntax,
};
/// Parse only the zero-argument template profile. Placeholder formatting remains
/// explicit unsupported until actual source argument construction is available.
pub(crate) fn expand<'a, L>(
    ast: &Syntax<L>,
    root: NodeId,
    mut literal: impl FnMut(u32) -> Result<Value<'a>, LexError>,
    newline: bool,
    constructor: NameId,
    grammar: MacroContext,
    config: &Config,
) -> Result<Expansion, LexError> {
    if !matches!(grammar, MacroContext::Expression | MacroContext::Statements) {
        return Err(error(0, "formatting requires an expression context"));
    }
    let (span, text) = super::string_argument(ast, root, &mut literal)?;
    let text = literal_bytes(&text, newline).map_err(|message| error(span.start, message))?;
    // Diagnostic image only: syntax is built from owned decoded bytes and is
    // never reparsed. A nonempty range also authenticates an empty literal use.
    let source = format!("{text:?}");
    let span = Span {
        start: 0,
        end: u32::try_from(source.len())
            .map_err(|_| error(0, "formatting literal exceeds source limit"))?,
    };
    let mut output = Ast::default();
    output
        .literals
        .push(Value::String(text.into_bytes().into()));
    output.nodes.push(Node {
        span,
        kind: Kind::Literal(0),
    });
    output.root = NodeId::new(1);
    output.nodes.push(Node {
        span,
        kind: Kind::LibraryValue {
            kind: LibraryValue::FormatLiteral { constructor },
            value: Some(NodeId::new(0)),
        },
    });
    if grammar == MacroContext::Statements {
        let tail = output.root;
        output.root = NodeId::new(2);
        output.nodes.push(Node {
            span,
            kind: Kind::Expansion {
                children: List { start: 0, end: 0 },
                tail: Some(tail),
            },
        });
    }
    // Like the existing owned library-value builtin, all decoded literals are
    // owned and no deferred source/attribute forest borrows this diagnostic image.
    let syntax = crate_syntax::configure("", output, config)?;
    Ok(Expansion { source, syntax })
}
fn literal_bytes(text: &str, newline: bool) -> Result<String, &'static str> {
    let mut result = String::with_capacity(text.len().saturating_add(usize::from(newline)));
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        match character {
            '{' => {
                if characters.next() == Some('{') {
                    result.push('{');
                } else {
                    return Err(
                        "formatting placeholders require actual source argument construction",
                    );
                }
            }
            '}' => {
                if characters.next() == Some('}') {
                    result.push('}');
                } else {
                    return Err("unmatched closing brace in formatting literal");
                }
            }
            character => result.push(character),
        }
    }
    if newline {
        result.push('\n');
    }
    Ok(result)
}
