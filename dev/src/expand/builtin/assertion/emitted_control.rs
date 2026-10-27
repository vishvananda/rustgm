//! Assemble a real conditional and panic path around one transferred expression.
use super::{Condition, error};
use crate::{
    expand::{
        builtin::{
            choice::Charge,
            delegation::{Delegated, Site},
        },
        config::Config,
        crate_syntax,
        macros::input::Input,
        registry::Registry,
        stream::Stream,
    },
    lex::token::{Kind as TokenKind, LexError, Punctuation, Token},
    literal::Value,
    support::{edition::Edition, retained::Growth},
    syntax::{
        ast::{Ast, BlockFlavor, Kind, List, MacroContext, Node, NodeId, Unary},
        interpolation::Inputs,
        token_tree::{ForestBuilder, Span},
    },
};
pub(crate) struct Request<'a> {
    pub site: Site<'a>,
    pub edition: Edition,
    pub input: &'a Input,
    pub registry: &'a Registry,
    pub inputs: &'a dyn Inputs,
    pub config: &'a Config,
    pub charge: &'a Charge<'a>,
}
struct Output<'a> {
    site: &'a Site<'a>,
    edition: Edition,
    charge: &'a Charge<'a>,
    ast: Ast<'static>,
    stream: Stream,
}
pub(crate) fn build(
    request: &Request<'_>,
    mut provenance: impl FnMut(Token) -> Result<(crate::expand::marks::SyntaxContextId, bool), LexError>,
) -> Result<(Delegated, usize), LexError> {
    let Request {
        site,
        input,
        inputs,
        config,
        charge,
        edition,
        registry,
    } = request;
    if !matches!(
        site.grammar,
        MacroContext::Expression | MacroContext::Statements
    ) {
        return Err(error(
            site.original.span.start,
            "assert requires an expression context",
        ));
    }
    charge(1, size_of::<Request<'_>>())?;
    let prepared = super::prepare(site.source, input, *inputs, charge)?;
    let message = if prepared.message.is_empty() {
        Some(prepared.default_message(*inputs, charge)?)
    } else {
        None
    };
    let Condition {
        ast,
        stream,
        message: suffix,
        parser_calls,
    } = prepared.consume("", &mut provenance, charge)?;
    let mut output = Output {
        site,
        edition: *edition,
        charge,
        ast,
        stream,
    };
    let condition = output.ast.root;
    let negate = output.text("!")?;
    let condition = output.node(
        negate,
        Kind::Unary {
            op: Unary::Not,
            operand: condition,
        },
    )?;
    // Finish vocabulary before forwarding registers the immutable token handles.
    let block = output.text("{}")?;
    let introducer = output.text("if")?;
    let failure = if let Some(message) = message {
        output.default_call(message)?
    } else {
        output.message_call(input, suffix, registry, &mut provenance)?
    };
    let then = output.node(
        block,
        Kind::Block {
            flavor: BlockFlavor::Plain,
            statements: List { start: 0, end: 0 },
            tail: Some(failure),
        },
    )?;
    output.ast.root = output.node(
        introducer,
        Kind::If {
            condition,
            then,
            otherwise: None,
        },
    )?;
    if site.grammar == MacroContext::Statements {
        let tail = output.ast.root;
        output.ast.root = output.node(
            introducer,
            Kind::Expansion {
                children: List { start: 0, end: 0 },
                tail: Some(tail),
            },
        )?;
    }
    let syntax = crate_syntax::configure(&output.stream.source, output.ast, config)?;
    Ok((
        Delegated {
            stream: output.stream,
            syntax,
        },
        parser_calls,
    ))
}
impl Output<'_> {
    fn text(&mut self, text: &str) -> Result<Span, LexError> {
        let bytes = Growth::vector::<u8>(
            self.stream.source.len(),
            self.stream.source.capacity(),
            text.len().saturating_add(1),
        );
        let origins = Growth::vector::<crate::expand::stream::Origin>(
            self.stream.origins.len(),
            self.stream.origins.capacity(),
            1,
        );
        (self.charge)(
            text.len()
                .saturating_add(bytes.work)
                .saturating_add(origins.work),
            bytes.bytes.saturating_add(origins.bytes),
        )?;
        self.stream.synthetic_image(
            text,
            self.site.original,
            self.site.path_context,
            self.site.introduction,
            self.edition,
        )
    }
    fn node(&mut self, span: Span, kind: Kind) -> Result<NodeId, LexError> {
        let growth = Growth::vector::<Node>(self.ast.nodes.len(), self.ast.nodes.capacity(), 1);
        (self.charge)(growth.work.saturating_add(1), growth.bytes)?;
        let index = u32::try_from(self.ast.nodes.len())
            .ok()
            .and_then(NodeId::checked)
            .ok_or_else(|| error(span.start, "assert syntax node overflow"))?;
        self.ast
            .nodes
            .try_reserve(1)
            .map_err(|_| error(span.start, "assert node allocation failed"))?;
        self.ast.nodes.push(Node { span, kind });
        Ok(index)
    }
    fn list(&mut self, nodes: &[NodeId]) -> Result<List, LexError> {
        let growth =
            Growth::vector::<NodeId>(self.ast.edges.len(), self.ast.edges.capacity(), nodes.len());
        (self.charge)(growth.work.saturating_add(nodes.len()), growth.bytes)?;
        let start =
            u32::try_from(self.ast.edges.len()).map_err(|_| error(0, "assert edge overflow"))?;
        let end = start
            .checked_add(u32::try_from(nodes.len()).map_err(|_| error(0, "assert edge overflow"))?)
            .ok_or_else(|| error(0, "assert edge overflow"))?;
        self.ast
            .edges
            .try_reserve(nodes.len())
            .map_err(|_| error(0, "assert edge allocation failed"))?;
        self.ast.edges.extend_from_slice(nodes);
        Ok(List { start, end })
    }
    fn path(&mut self, middle: &str, last: &str) -> Result<NodeId, LexError> {
        let root = self.text("$crate")?;
        let root = self.node(root, Kind::CrateRoot(0))?;
        let span = self.text(middle)?;
        let middle = self.node(span, Kind::Name(span, None))?;
        let span = self.text(last)?;
        let last = self.node(span, Kind::Name(span, None))?;
        let segments = self.list(&[root, middle, last])?;
        self.node(
            span,
            Kind::Path {
                absolute: false,
                segments,
            },
        )
    }
    fn default_call(&mut self, message: String) -> Result<NodeId, LexError> {
        (self.charge)(
            message.len().saturating_add(1),
            message.len().saturating_mul(8).saturating_add(8),
        )?;
        let mut quoted = String::new();
        quoted
            .try_reserve(message.len().saturating_mul(4).saturating_add(2))
            .map_err(|_| error(0, "assert literal allocation failed"))?;
        quoted.push('"');
        for c in message.chars() {
            quoted.extend(c.escape_default());
        }
        quoted.push('"');
        let span = self.text(&quoted)?;
        let slot = u32::try_from(self.ast.literals.len())
            .map_err(|_| error(span.start, "assert literal overflow"))?;
        let growth =
            Growth::vector::<Value<'_>>(self.ast.literals.len(), self.ast.literals.capacity(), 1);
        (self.charge)(growth.work.saturating_add(1), growth.bytes)?;
        self.ast
            .literals
            .try_reserve(1)
            .map_err(|_| error(span.start, "assert literal allocation failed"))?;
        self.ast
            .literals
            .push(Value::String(message.into_bytes().into()));
        let literal = self.node(span, Kind::Literal(slot))?;
        let arguments = self.list(&[literal])?;
        let callee = self.path("panicking", "panic")?;
        self.node(span, Kind::Call { callee, arguments })
    }
    fn message_call(
        &mut self,
        input: &Input,
        suffix: std::ops::Range<usize>,
        registry: &Registry,
        provenance: impl FnMut(Token) -> Result<(crate::expand::marks::SyntaxContextId, bool), LexError>,
    ) -> Result<NodeId, LexError> {
        let path = self.path("panic", "panic_2021")?;
        let first = self.stream.tokens.len();
        let opening = self.text("(")?;
        self.stream.tokens.push(Token {
            kind: TokenKind::Punctuation(Punctuation::OpenParen),
            start: opening.start,
            end: opening.end,
        });
        let tokens = input
            .tokens
            .get(suffix)
            .ok_or_else(|| error(0, "assert message range changed"))?;
        (self.charge)(
            tokens.len(),
            tokens.len().saturating_mul(4 * size_of::<Token>()),
        )?;
        self.stream
            .forward_tokens(tokens, self.site.source, provenance)?;
        let closing = self.text(")")?;
        self.stream.tokens.push(Token {
            kind: TokenKind::Punctuation(Punctuation::CloseParen),
            start: closing.start,
            end: closing.end,
        });
        registry.forward(&mut self.stream)?;
        (self.charge)(
            self.stream.tokens.len() - first,
            (self.stream.tokens.len() - first)
                .saturating_mul(4 * size_of::<crate::syntax::token_tree::Tree>()),
        )?;
        let mut forest = ForestBuilder::default();
        for &token in &self.stream.tokens[first..] {
            forest.push(&self.stream.source, token)?;
        }
        let capture = u32::try_from(self.ast.captures.len())
            .map_err(|_| error(0, "assert capture overflow"))?;
        self.ast.captures.push(forest.finish(closing.end)?);
        self.node(
            closing,
            Kind::Macro {
                path,
                capture,
                context: MacroContext::Expression,
            },
        )
    }
}
