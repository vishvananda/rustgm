//! Typed local panic delegation preserves the actual argument token forest.
use crate::{
    expand::{
        config::Config,
        crate_syntax::{self, ConfiguredCrate},
        macros::input::Input,
        marks::{ExpansionId, SyntaxContextId},
        registry::Registry,
        stream::{Location, Source, Stream},
    },
    lex::token::{Kind as TokenKind, LexError, Punctuation, Token},
    syntax::{
        ast::{Ast, Kind, List, MacroContext, Node, NodeId},
        token_tree::{ForestBuilder, Span, error},
    },
};

#[derive(Clone, Copy)]
pub(crate) enum Handler {
    Panic2021,
    Unreachable2021,
}
impl Handler {
    fn name(self) -> &'static str {
        match self {
            Self::Panic2021 => "panic_2021",
            Self::Unreachable2021 => "unreachable_2021",
        }
    }
}
pub(crate) struct Site<'a> {
    pub source: Source<'a>,
    pub original: Location,
    pub introduction: ExpansionId,
    pub call_context: SyntaxContextId,
    pub path_context: SyntaxContextId,
    pub grammar: MacroContext,
}
pub(crate) struct Delegated {
    pub stream: Stream,
    pub syntax: ConfiguredCrate<'static>,
}
impl Delegated {
    pub(crate) fn payload_bytes(&self) -> usize {
        let ast = &self.syntax.ast;
        self.stream
            .source
            .capacity()
            .saturating_add(
                self.stream
                    .tokens
                    .capacity()
                    .saturating_mul(size_of::<Token>()),
            )
            .saturating_add(
                self.stream
                    .origins
                    .capacity()
                    .saturating_mul(size_of::<crate::expand::stream::Origin>()),
            )
            .saturating_add(
                self.stream
                    .opaque
                    .capacity()
                    .saturating_mul(size_of::<crate::expand::stream::Opaque>()),
            )
            .saturating_add(ast.nodes.capacity().saturating_mul(size_of::<Node>()))
            .saturating_add(ast.edges.capacity().saturating_mul(size_of::<NodeId>()))
            .saturating_add(
                ast.captures
                    .capacity()
                    .saturating_mul(size_of::<crate::syntax::token_tree::Forest>()),
            )
            .saturating_add(
                ast.captures
                    .iter()
                    .map(crate::syntax::token_tree::Forest::payload_bytes)
                    .sum::<usize>(),
            )
            .saturating_add(self.syntax.active.capacity())
            .saturating_add(
                self.syntax
                    .owners
                    .capacity()
                    .saturating_mul(size_of::<crate_syntax::Owner>()),
            )
            .saturating_add(
                self.syntax
                    .effects
                    .capacity()
                    .saturating_mul(size_of::<crate_syntax::AttributeEffect>()),
            )
    }
}
/// Only the resolver's validated local defining-source contract calls this.
/// Foreign defining roots remain unsupported rather than fabricated wire IDs.
pub(crate) fn build(
    site: &Site<'_>,
    handler: Handler,
    input: &Input,
    registry: &Registry,
    config: &Config,
    provenance: impl FnMut(Token) -> Result<(SyntaxContextId, bool), LexError>,
) -> Result<Delegated, LexError> {
    if !matches!(
        site.grammar,
        MacroContext::Expression | MacroContext::Statements
    ) {
        return Err(error(
            site.original.span.start,
            "source panic delegation requires an expression context",
        ));
    }
    let mut stream = Stream::default();
    let call = stream.image(
        "panic!",
        site.original,
        Some(site.call_context),
        site.introduction,
    )?;
    let mut ast = Ast::default();
    for (index, spelling) in ["$crate", "panic", handler.name()].into_iter().enumerate() {
        let span = stream.image(
            spelling,
            site.original,
            Some(site.path_context),
            site.introduction,
        )?;
        let kind = if index == 0 {
            Kind::CrateRoot(0)
        } else {
            Kind::Name(span, None)
        };
        ast.nodes.push(Node { span, kind });
        ast.edges.push(NodeId::new(
            u32::try_from(index).map_err(|_| error(call.start, "panic path overflow"))?,
        ));
    }
    let path = NodeId::new(3);
    ast.nodes.push(Node {
        span: call,
        kind: Kind::Path {
            absolute: false,
            segments: List { start: 0, end: 3 },
        },
    });
    let opening = stream.image(
        "(",
        site.original,
        Some(site.call_context),
        site.introduction,
    )?;
    stream.tokens.push(Token {
        kind: TokenKind::Punctuation(Punctuation::OpenParen),
        start: opening.start,
        end: opening.end,
    });
    stream.forward_input(input, site.source, provenance)?;
    let closing = stream.image(
        ")",
        site.original,
        Some(site.call_context),
        site.introduction,
    )?;
    stream.tokens.push(Token {
        kind: TokenKind::Punctuation(Punctuation::CloseParen),
        start: closing.start,
        end: closing.end,
    });
    registry.forward(&mut stream)?;
    let mut forest = ForestBuilder::default();
    for &token in &stream.tokens {
        forest.push(&stream.source, token)?;
    }
    ast.captures.push(forest.finish(closing.end)?);
    invocation(&mut ast, path, call, site.grammar);
    // Typed path nodes and owned Forest captures have no deferred literal or
    // attribute owners; configuration never borrows the diagnostic spelling.
    let syntax = crate_syntax::configure("", ast, config)?;
    Ok(Delegated { stream, syntax })
}
fn invocation(ast: &mut Ast, path: NodeId, call: Span, grammar: MacroContext) {
    ast.root = NodeId::new(4);
    ast.nodes.push(Node {
        span: call,
        kind: Kind::Macro {
            path,
            capture: 0,
            context: MacroContext::Expression,
        },
    });
    if grammar == MacroContext::Statements {
        let tail = ast.root;
        ast.root = NodeId::new(5);
        ast.nodes.push(Node {
            span: call,
            kind: Kind::Expansion {
                children: List { start: 3, end: 3 },
                tail: Some(tail),
            },
        });
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/delegation_inputs.rs"]
mod tests;
