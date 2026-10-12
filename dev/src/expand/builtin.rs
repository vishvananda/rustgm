//! Builtin expansion consumes classified inputs and constructs typed syntax directly.
use super::{
    config::{self, Config},
    crate_syntax::{self, ConfiguredCrate},
    macros::input::Input,
};
use crate::{
    lex::token::{Kind as TokenKind, LexError, Punctuation},
    literal::{
        Value,
        value::{Integer, IntegerType},
    },
    support::source::Position,
    syntax::{
        ast::{Ast, Kind, List, MacroContext, Node, NodeId},
        interpolation::{Inputs, Nonterminal},
        token_tree::{ForestBuilder, Span, error},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Builtin {
    Cfg,
    Line,
    Column,
}
impl Builtin {
    pub const ALL: [Self; 3] = [Self::Cfg, Self::Line, Self::Column];
    #[must_use]
    pub const fn needs_position(self) -> bool {
        matches!(self, Self::Line | Self::Column)
    }
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Cfg => "cfg",
            Self::Line => "line",
            Self::Column => "column",
        }
    }
}

pub struct Environment<'a> {
    pub config: &'a Config,
    pub grammar: MacroContext,
    pub position: Option<Position>,
}

pub struct Expansion {
    pub source: String,
    pub syntax: ConfiguredCrate<'static>,
}

/// # Errors
/// Rejects invalid builtin arguments, missing typed captures and invalid output contexts.
pub fn expand(
    builtin: Builtin,
    source: &str,
    input: &Input,
    inputs: &dyn Inputs,
    environment: &Environment<'_>,
) -> Result<Expansion, LexError> {
    match builtin {
        Builtin::Cfg => cfg(
            source,
            input,
            inputs,
            environment.config,
            environment.grammar,
        ),
        Builtin::Line | Builtin::Column => {
            if let Some(token) = input.tokens.first() {
                return Err(error(token.start, "location builtin takes no arguments"));
            }
            let position = environment
                .position
                .ok_or_else(|| error(0, "missing physical builtin position"))?;
            let value = if builtin == Builtin::Line {
                position.line
            } else {
                position.column
            };
            literal(
                Value::Integer(Integer {
                    magnitude: u128::from(value),
                    suffix: Some(IntegerType::U32),
                }),
                format!("{value}u32"),
                environment.grammar,
                environment.config,
            )
        }
    }
}

fn cfg(
    source: &str,
    input: &Input,
    inputs: &dyn Inputs,
    config: &Config,
    context: MacroContext,
) -> Result<Expansion, LexError> {
    let mut builder = ForestBuilder::default();
    for token in &input.tokens {
        if matches!(token.kind, TokenKind::Interpolated(_)) {
            let splice = inputs
                .get(*token)?
                .ok_or_else(|| error(token.start, "missing builtin input fragment"))?;
            if let Nonterminal::Metadata { forest, .. } = splice.fragment {
                builder.append(forest, |span| splice.span(span))?;
                continue;
            }
            if splice.append_attribute_path(source, &mut builder)? {
                continue;
            }
        }
        builder.push(source, *token)?;
    }
    let forest = builder.finish(input.tokens.last().map_or(0, |t| t.end))?;
    let mut range = forest.roots();
    if range.start < range.end && forest.punctuation(range.end - 1, Punctuation::Comma) {
        range.end -= 1;
    }
    let value = config::predicate(&forest, source, range, config)?;
    literal(
        Value::Boolean(value),
        if value { "true" } else { "false" }.into(),
        context,
        config,
    )
}
fn literal(
    value: Value<'static>,
    source: String,
    context: MacroContext,
    config: &Config,
) -> Result<Expansion, LexError> {
    if matches!(context, MacroContext::Type | MacroContext::Items(_)) {
        return Err(error(
            0,
            "builtin expression is invalid in this syntax context",
        ));
    }
    let span = Span {
        start: 0,
        end: u32::try_from(source.len())
            .map_err(|_| error(0, "builtin output exceeds source limit"))?,
    };
    let mut ast = Ast::default();
    ast.literals.push(value);
    ast.nodes.push(Node {
        span,
        kind: Kind::Literal(0),
    });
    if context == MacroContext::Statements {
        ast.root = NodeId(1);
        ast.nodes.push(Node {
            span,
            kind: Kind::Expansion {
                children: List { start: 0, end: 0 },
                tail: Some(NodeId(0)),
            },
        });
    }
    // There are no deferred tokens in this typed arena, so configuration retains
    // the owned value without deriving any semantic fact from its spelling.
    let syntax = crate_syntax::configure("", ast, config)?;
    Ok(Expansion { source, syntax })
}
