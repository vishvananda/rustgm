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
    File,
    ModulePath,
    Stringify,
    Concat,
    CompileError,
    Env,
    IncludeStr,
    IncludeBytes,
    Include,
}
impl Builtin {
    pub const ALL: [Self; 12] = [
        Self::Cfg,
        Self::Line,
        Self::Column,
        Self::File,
        Self::ModulePath,
        Self::Stringify,
        Self::Concat,
        Self::CompileError,
        Self::Env,
        Self::IncludeStr,
        Self::IncludeBytes,
        Self::Include,
    ];
    #[must_use]
    pub const fn eager(self) -> bool {
        matches!(
            self,
            Self::Concat
                | Self::CompileError
                | Self::Env
                | Self::IncludeStr
                | Self::IncludeBytes
                | Self::Include
        )
    }
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
            Self::File => "file",
            Self::ModulePath => "module_path",
            Self::Stringify => "stringify",
            Self::Concat => "concat",
            Self::CompileError => "compile_error",
            Self::Env => "env",
            Self::IncludeStr => "include_str",
            Self::IncludeBytes => "include_bytes",
            Self::Include => "include",
        }
    }
}

pub struct Environment<'a> {
    pub config: &'a Config,
    pub grammar: MacroContext,
    pub position: Option<Position>,
    pub text: Option<&'a str>,
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
        Builtin::Concat
        | Builtin::CompileError
        | Builtin::Env
        | Builtin::IncludeStr
        | Builtin::IncludeBytes
        | Builtin::Include => Err(error(0, "builtin requires expanded argument facts")),
        Builtin::Cfg => cfg(
            source,
            input,
            inputs,
            environment.config,
            environment.grammar,
        ),
        Builtin::File | Builtin::ModulePath | Builtin::Stringify => {
            if let Some(token) = input.tokens.first()
                && builtin != Builtin::Stringify
            {
                return Err(error(token.start, "location builtin takes no arguments"));
            }
            let text = environment
                .text
                .ok_or_else(|| error(0, "missing builtin source name"))?;
            literal(
                Value::String(text.as_bytes().to_vec().into()),
                format!("{text:?}"),
                environment.grammar,
                environment.config,
            )
        }
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

/// Build concat's string directly from expanded literal facts.
///
/// # Errors
/// Rejects nonliteral operands, byte/C strings and invalid output contexts.
pub fn concat<'a, L>(
    ast: &crate::syntax::ast::Syntax<L>,
    root: NodeId,
    mut value: impl FnMut(u32) -> Result<Value<'a>, LexError>,
    grammar: MacroContext,
    config: &Config,
) -> Result<Expansion, LexError> {
    use std::fmt::Write;
    let Kind::Expansion {
        children,
        tail: None,
    } = ast.node(root).kind
    else {
        return Err(error(0, "missing builtin argument container"));
    };
    let mut text = String::new();
    for &argument in ast.list(children) {
        let node = ast.node(argument);
        let (kind, negative) = if let Kind::Unary {
            op: crate::syntax::ast::Unary::Negate,
            operand,
        } = node.kind
        {
            (ast.node(operand).kind, true)
        } else {
            (node.kind, false)
        };
        let Kind::Literal(index) = kind else {
            return Err(error(node.span.start, "concat requires literal arguments"));
        };
        let literal = value(index)?;
        if negative && !matches!(literal, Value::Integer(_) | Value::Float(_)) {
            return Err(error(
                node.span.start,
                "concat requires numeric literals after minus",
            ));
        }
        if negative {
            text.push('-');
        }
        match literal {
            Value::String(bytes) => text.push_str(
                std::str::from_utf8(&bytes)
                    .map_err(|_| error(node.span.start, "invalid string literal UTF-8"))?,
            ),
            Value::Character(ch) => text.push(ch),
            Value::Integer(integer) => write!(&mut text, "{}", integer.magnitude)
                .map_err(|_| error(node.span.start, "cannot format integer literal"))?,
            Value::Float(float) => text.push_str(&float.decimal),
            Value::Boolean(boolean) => text.push_str(if boolean { "true" } else { "false" }),
            Value::Byte(_) | Value::ByteString(_) | Value::CString(_) => {
                return Err(error(
                    node.span.start,
                    "concat does not accept byte or C string literals",
                ));
            }
        }
    }
    string(text, grammar, config)
}

/// Read one eagerly expanded string argument for diagnostic and source builtins.
///
/// # Errors
/// Rejects arity, nonliteral arguments and invalid string storage.
pub fn string_argument<'a, L>(
    ast: &crate::syntax::ast::Syntax<L>,
    root: NodeId,
    mut value: impl FnMut(u32) -> Result<Value<'a>, LexError>,
) -> Result<(Span, String), LexError> {
    let container = ast.node(root);
    let failure = || {
        error(
            container.span.start,
            "builtin requires one string literal argument",
        )
    };
    let Kind::Expansion {
        children,
        tail: None,
    } = container.kind
    else {
        return Err(failure());
    };
    let [argument] = ast.list(children) else {
        return Err(failure());
    };
    let node = ast.node(*argument);
    let Kind::Literal(index) = node.kind else {
        return Err(failure());
    };
    let Value::String(bytes) = value(index)? else {
        return Err(failure());
    };
    let text = String::from_utf8(bytes.into_owned()).map_err(|_| failure())?;
    Ok((node.span, text))
}

/// # Errors
/// Rejects output contexts which cannot contain a string expression/pattern.
pub fn string(
    value: String,
    grammar: MacroContext,
    config: &Config,
) -> Result<Expansion, LexError> {
    let spelling = format!("{value:?}");
    literal(
        Value::String(value.into_bytes().into()),
        spelling,
        grammar,
        config,
    )
}

/// # Errors
/// Rejects invalid output contexts and diagnostic spelling failures.
pub fn byte_string(
    value: Vec<u8>,
    grammar: MacroContext,
    config: &Config,
) -> Result<Expansion, LexError> {
    let mut spelling = vec![b'b'];
    crate::support::quoted::bytes(&mut spelling, &value, false)
        .map_err(|_| error(0, "cannot quote builtin byte string"))?;
    let spelling = String::from_utf8(spelling)
        .map_err(|_| error(0, "invalid builtin byte string spelling"))?;
    literal(Value::ByteString(value.into()), spelling, grammar, config)
}

/// # Errors
/// Rejects env's arity or non-string arguments before an external lookup occurs.
pub fn environment_arguments<'a, L>(
    ast: &crate::syntax::ast::Syntax<L>,
    root: NodeId,
    mut value: impl FnMut(u32) -> Result<Value<'a>, LexError>,
) -> Result<(String, Option<String>), LexError> {
    let container = ast.node(root);
    let failure = || {
        error(
            container.span.start,
            "env requires one or two string literal arguments",
        )
    };
    let Kind::Expansion {
        children,
        tail: None,
    } = container.kind
    else {
        return Err(failure());
    };
    let arguments = ast.list(children);
    if !(1..=2).contains(&arguments.len()) {
        return Err(failure());
    }
    let mut strings = arguments.iter().map(|argument| {
        let Kind::Literal(index) = ast.node(*argument).kind else {
            return Err(failure());
        };
        let Value::String(bytes) = value(index)? else {
            return Err(failure());
        };
        String::from_utf8(bytes.into_owned()).map_err(|_| failure())
    });
    let name = strings.next().ok_or_else(failure)??;
    let message = strings.next().transpose()?;
    Ok((name, message))
}
