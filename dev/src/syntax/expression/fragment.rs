//! Macro nonterminals call the ordinary grammar over classified tokens.
use super::{Ast, BlockFlavor, Context, Kind, LexError, NORMAL, NodeId, P, Parser, Word, items};
use crate::{
    lex::token::Token,
    syntax::cursor::{Cursor, TokenPosition},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FragmentKind {
    Block,
    Expression,
    Expression2021,
    Item,
    Literal,
    Pattern,
    PatternParameter,
    Path,
    Statement,
    Type,
}
pub struct Parsed<'source> {
    pub ast: Ast<'source>,
    pub position: TokenPosition,
}
/// Parse one demanded fragment prefix; its remainder stays in the macro matcher.
/// Literals remain deferred until expansion establishes a configuration owner.
///
/// # Errors
/// Returns ordinary syntax errors, edition-fragment restrictions and invalid spans.
pub fn parse<'s>(
    source: &'s str,
    tokens: &'s [Token],
    kind: FragmentKind,
) -> Result<Parsed<'s>, LexError> {
    parse_at(source, tokens, kind, TokenPosition::default())
}

/// # Errors
/// Returns grammar failures and invalid typed-token positions.
pub fn parse_at<'s>(
    source: &'s str,
    tokens: &'s [Token],
    kind: FragmentKind,
    start: TokenPosition,
) -> Result<Parsed<'s>, LexError> {
    parse_with_splices(source, tokens, kind, start, &[])
}
/// # Errors
/// Returns ordinary grammar failures or invalid opaque-fragment provenance.
pub fn parse_with_splices<'s>(
    source: &'s str,
    tokens: &'s [Token],
    kind: FragmentKind,
    start: TokenPosition,
    splices: &[crate::syntax::interpolation::Splice<'_>],
) -> Result<Parsed<'s>, LexError> {
    parse_with_inputs(
        source,
        tokens,
        kind,
        start,
        &crate::syntax::interpolation::Slice(splices),
    )
}
/// # Errors
/// Reports grammar errors or missing persistent nonterminal identities.
pub fn parse_with_inputs<'s>(
    source: &'s str,
    tokens: &'s [Token],
    kind: FragmentKind,
    start: TokenPosition,
    inputs: &dyn crate::syntax::interpolation::Inputs,
) -> Result<Parsed<'s>, LexError> {
    let mut parser = Parser::with_cursor(Cursor::from_position(source, tokens, start)?, true);
    parser.inputs = inputs;
    parser.has_opaque = !inputs.is_empty();
    let root = match kind {
        FragmentKind::Block => {
            let root = parser.block(BlockFlavor::Plain)?;
            if matches!(parser.ast.node(root).kind, Kind::Attributed { .. }) {
                return Err(parser.error("block fragment cannot contain inner attributes")?);
            }
            root
        }
        FragmentKind::Expression | FragmentKind::Expression2021 => {
            if kind == FragmentKind::Expression2021
                && (parser.cursor.at_word(Word::Underscore)?
                    || parser.cursor.at_word(Word::Const)?)
            {
                return Err(
                    parser.error("expr_2021 excludes top-level underscore and const blocks")?
                );
            }
            parser.expr(0, NORMAL)?
        }
        FragmentKind::Item => parser.item_in(items::Scope::Module)?,
        FragmentKind::Literal => parser.literal_pattern()?,
        FragmentKind::Pattern => parser.pattern(true)?,
        FragmentKind::PatternParameter => parser.pattern(false)?,
        FragmentKind::Path => {
            if parser.cursor.at(P::Less)? || parser.cursor.at(P::ShiftLeft)? {
                return Err(parser.error("path fragment cannot begin with a qualified type")?);
            }
            parser.rich_path(true)?
        }
        FragmentKind::Statement => parser.fragment_statement()?,
        FragmentKind::Type => parser.ty()?,
    };
    let position = parser.cursor.token_position()?;
    parser.ast.root = root;
    parser.ast.lookahead_high_water = parser.cursor.high_water;
    Ok(Parsed {
        ast: parser.ast,
        position,
    })
}
pub(crate) use super::statement_modes::statement_expression;
impl Parser<'_, '_> {
    fn fragment_statement(&mut self) -> Result<NodeId, LexError> {
        if let Some(node) = self.interpolated(&[FragmentKind::Statement])? {
            return Ok(node);
        }
        if self.starts_item()? {
            return self.item_in(items::Scope::Module);
        }
        if self.cursor.at(P::Semicolon)? {
            let start = self.required()?.start;
            return self.node(start, Kind::EmptyStatement);
        }
        if self.cursor.at_word(Word::Let)? {
            return self.let_statement(false);
        }
        let opaque = self
            .cursor
            .peek(0)?
            .is_some_and(|token| matches!(token.kind, super::TokenKind::Interpolated(_)));
        let value = self.expr(
            0,
            Context {
                statement: true,
                ..NORMAL
            },
        )?;
        if !opaque {
            self.statement_macro(value);
        }
        self.node(self.ast.node(value).span.start, Kind::Statement(value))
    }
}

pub struct ParsedVisibility<'source> {
    pub value: crate::syntax::item::Visibility,
    pub ast: Option<Ast<'source>>,
    pub position: TokenPosition,
}
/// # Errors
/// Returns malformed visibility or invalid typed-token positions.
pub fn visibility<'s>(
    source: &'s str,
    tokens: &'s [Token],
    start: TokenPosition,
) -> Result<ParsedVisibility<'s>, LexError> {
    visibility_with_inputs(
        source,
        tokens,
        start,
        &crate::syntax::interpolation::EMPTY_INPUTS,
    )
}

/// # Errors
/// Reports malformed visibility and missing path-token edition provenance.
pub fn visibility_with_inputs<'s>(
    source: &'s str,
    tokens: &'s [Token],
    start: TokenPosition,
    inputs: &dyn crate::syntax::interpolation::Inputs,
) -> Result<ParsedVisibility<'s>, LexError> {
    let mut parser = Parser::with_cursor(Cursor::from_position(source, tokens, start)?, true);
    parser.inputs = inputs;
    parser.has_opaque = !inputs.is_empty();
    let value = parser.visibility()?;
    let position = parser.cursor.token_position()?;
    let ast = if let crate::syntax::item::Visibility::Restricted(root) = value {
        parser.ast.root = root;
        parser.ast.lookahead_high_water = parser.cursor.high_water;
        Some(parser.ast)
    } else {
        None
    };
    Ok(ParsedVisibility {
        value,
        ast,
        position,
    })
}

pub struct ParsedMetadata<'source> {
    pub forest: crate::syntax::token_tree::Forest,
    pub attribute: crate::syntax::attribute::Attribute,
    pub expression: Option<Ast<'source>>,
    pub position: TokenPosition,
}
/// # Errors
/// Returns malformed attribute syntax, value expressions and token positions.
pub fn metadata<'s>(
    source: &'s str,
    tokens: &'s [Token],
    start: TokenPosition,
) -> Result<ParsedMetadata<'s>, LexError> {
    metadata_with_inputs(
        source,
        tokens,
        start,
        &crate::syntax::interpolation::EMPTY_INPUTS,
    )
}
/// # Errors
/// Reports malformed metadata and invalid opaque path or expression inputs.
pub fn metadata_with_inputs<'s>(
    source: &'s str,
    tokens: &'s [Token],
    start: TokenPosition,
    inputs: &dyn crate::syntax::interpolation::Inputs,
) -> Result<ParsedMetadata<'s>, LexError> {
    let mut parser = Parser::with_cursor(Cursor::from_position(source, tokens, start)?, true);
    parser.inputs = inputs;
    parser.has_opaque = !inputs.is_empty();
    let first = parser
        .cursor
        .peek(0)?
        .ok_or_else(|| super::error(parser.cursor.end(), "expected metadata fragment"))?;
    if parser.splice()?.is_some() {
        parser.required()?;
    } else {
        parser.eat(P::ColonColon)?;
        loop {
            let name = parser.required()?;
            if !matches!(
                name.kind,
                super::TokenKind::Identifier | super::TokenKind::RawIdentifier
            ) {
                return Err(super::error(name.start, "expected metadata path"));
            }
            if !parser.eat(P::ColonColon)? {
                break;
            }
        }
    }
    let expression = if parser.eat(P::Equal)? {
        Some(parser.expr(0, NORMAL)?)
    } else {
        if parser.cursor.peek(0)?.is_some_and(|t| {
            matches!(
                t.kind,
                super::TokenKind::Punctuation(P::OpenParen | P::OpenBrace | P::OpenBracket)
            )
        }) {
            let token = parser.required()?;
            parser.cursor.skip_group(token)?;
        }
        None
    };
    let position = parser.cursor.token_position()?;
    let end = parser.cursor.end();
    let mut cursor = Cursor::from_position(source, tokens, start)?;
    let mut builder = crate::syntax::token_tree::ForestBuilder::default();
    let mut context = crate::syntax::attribute::Context::new(inputs);
    while cursor.token_position()? != position {
        let token = cursor
            .take()?
            .ok_or_else(|| super::error(end, "invalid metadata token range"))?;
        if matches!(token.kind, super::TokenKind::Interpolated(_))
            && let Some(splice) = inputs.get(token)?
        {
            if let crate::syntax::interpolation::Nonterminal::Metadata { forest, .. } =
                splice.fragment
            {
                builder.append(forest, |span| splice.span(span))?;
                context.parsed_path(splice.destination)?;
                continue;
            }
            if splice.append_attribute_path(source, &mut builder)? {
                context.parsed_path(splice.destination)?;
                continue;
            }
        }
        builder.push(source, token)?;
    }
    let forest = builder.finish(end)?;
    let attribute = crate::syntax::attribute::parse_with_context(
        &forest,
        source,
        forest.roots(),
        crate::syntax::attribute::Style::Outer,
        crate::syntax::token_tree::Span {
            start: first.start,
            end,
        },
        &context,
    )?;
    let expression = expression.map(|root| {
        parser.ast.root = root;
        parser.ast.lookahead_high_water = parser.cursor.high_water;
        parser.ast
    });
    Ok(ParsedMetadata {
        forest,
        attribute,
        expression,
        position,
    })
}

/// Parse a complete generated fragment in its original syntactic context.
///
/// # Errors
/// Rejects malformed generated syntax, invalid splices and trailing unused tokens.
pub fn parse_expansion<'s>(
    source: &'s str,
    tokens: &'s [Token],
    context: crate::syntax::ast::MacroContext,
    splices: &[crate::syntax::interpolation::Splice<'_>],
) -> Result<Ast<'s>, LexError> {
    parse_expansion_with_inputs(
        source,
        tokens,
        context,
        &crate::syntax::interpolation::Slice(splices),
    )
}
/// # Errors
/// Reports malformed expansions or missing persistent nonterminal identities.
pub fn parse_expansion_with_inputs<'s>(
    source: &'s str,
    tokens: &'s [Token],
    context: crate::syntax::ast::MacroContext,
    inputs: &dyn crate::syntax::interpolation::Inputs,
) -> Result<Ast<'s>, LexError> {
    use crate::syntax::ast::MacroContext;
    let kind = match context {
        MacroContext::Expression => Some(FragmentKind::Expression),
        MacroContext::Type => Some(FragmentKind::Type),
        MacroContext::Pattern => Some(FragmentKind::Pattern),
        MacroContext::Statements | MacroContext::Items(_) => None,
    };
    if let Some(kind) = kind {
        let parsed = parse_with_inputs(source, tokens, kind, TokenPosition::default(), inputs)?;
        if parsed.position.index != tokens.len() || parsed.position.offset != 0 {
            return Err(super::error(
                tokens
                    .get(parsed.position.index)
                    .map_or(0, |t| t.start + parsed.position.offset),
                "unexpected token after macro expansion",
            ));
        }
        return Ok(parsed.ast);
    }
    let mut parser = Parser::with_cursor(Cursor::from_tokens(source, tokens), true);
    parser.inputs = inputs;
    parser.has_opaque = !inputs.is_empty();
    let (children, tail) = match context {
        MacroContext::Items(scope) => (parser.item_list(false, scope)?, None),
        MacroContext::Statements => parser.statement_list(None)?,
        _ => return Err(super::error(0, "invalid expansion parser context")),
    };
    let start = tokens.first().map_or(0, |t| t.start);
    parser.ast.root = parser.node(start, Kind::Expansion { children, tail })?;
    parser.ast.lookahead_high_water = parser.cursor.high_water;
    Ok(parser.ast)
}

#[derive(Clone, Copy)]
pub(crate) enum ArgumentsGrammar {
    Expressions,
    Assembly,
    Formatting,
}
/// Parse a builtin's comma-separated expression arguments once over classified tokens.
/// The expansion-shaped root is an argument container, not a source-level expression.
///
/// # Errors
/// Rejects malformed expressions, missing separators and invalid opaque inputs.
pub fn parse_arguments<'s>(
    source: &'s str,
    tokens: &'s [Token],
    inputs: &dyn crate::syntax::interpolation::Inputs,
) -> Result<Ast<'s>, LexError> {
    arguments(source, tokens, inputs, false)
}
pub(crate) fn parse_format_arguments<'s>(
    source: &'s str,
    tokens: &'s [Token],
    inputs: &dyn crate::syntax::interpolation::Inputs,
) -> Result<Ast<'s>, LexError> {
    arguments(source, tokens, inputs, true)
}
fn arguments<'s>(
    source: &'s str,
    tokens: &'s [Token],
    inputs: &dyn crate::syntax::interpolation::Inputs,
    formatting: bool,
) -> Result<Ast<'s>, LexError> {
    let mut parser = Parser::with_cursor(Cursor::from_tokens(source, tokens), true);
    parser.inputs = inputs;
    parser.has_opaque = !inputs.is_empty();
    while parser.cursor.peek(0)?.is_some() {
        let named = formatting
            && !parser.scratch.is_empty()
            && parser.cursor.peek(0)?.is_some_and(|token| {
                matches!(
                    token.kind,
                    super::TokenKind::Identifier | super::TokenKind::RawIdentifier
                )
            })
            && parser
                .cursor
                .peek(1)?
                .is_some_and(|token| token.kind == super::TokenKind::Punctuation(P::Equal));
        let argument = if named {
            let token = parser.required()?;
            let left = parser.name_node(token)?;
            parser.expect(P::Equal)?;
            let right = parser.expr(0, NORMAL)?;
            parser.node(
                token.start,
                Kind::Binary {
                    op: P::Equal,
                    left,
                    right,
                },
            )?
        } else {
            parser.expr(0, NORMAL)?
        };
        parser.scratch.push(argument);
        if !parser.eat(P::Comma)? && parser.cursor.peek(0)?.is_some() {
            return Err(parser.error("expected comma between builtin arguments")?);
        }
    }
    let children = parser.list(0)?;
    let start = tokens.first().map_or(0, |token| token.start);
    parser.ast.root = parser.node(
        start,
        Kind::Expansion {
            children,
            tail: None,
        },
    )?;
    parser.ast.lookahead_high_water = parser.cursor.high_water;
    Ok(parser.ast)
}

/// Parse included source once using the ordinary bounded source cursor.
/// Includes supply module items or one expression; block statements use that expression.
///
/// # Errors
/// Rejects unsupported contexts, inner file attributes and unused trailing tokens.
pub fn parse_include<'s>(
    source: &'s str,
    context: crate::syntax::ast::MacroContext,
    names: Option<&mut crate::support::names::Names>,
) -> Result<Ast<'s>, LexError> {
    parse_include_in_edition(
        source,
        context,
        names,
        crate::support::edition::Edition::Rust2024,
    )
}

/// Parse a physical include with its compilation unit's lexical edition.
/// # Errors
/// Rejects malformed included syntax and unsupported include contexts.
pub fn parse_include_in_edition<'s>(
    source: &'s str,
    context: crate::syntax::ast::MacroContext,
    names: Option<&mut crate::support::names::Names>,
    edition: crate::support::edition::Edition,
) -> Result<Ast<'s>, LexError> {
    use crate::syntax::{
        ast::{List, MacroContext},
        item::ItemContext,
    };
    let items = match context {
        MacroContext::Items(ItemContext::Module) => true,
        MacroContext::Expression | MacroContext::Statements => false,
        _ => {
            return Err(super::error(
                0,
                "include supports module items or an expression",
            ));
        }
    };
    let mut parser = Parser::with_cursor(Cursor::source(source, items)?, true);
    parser.names = names;
    parser.inputs = &edition;
    let root = if items {
        let children = parser.item_list(false, ItemContext::Module)?;
        parser.node(
            0,
            Kind::Expansion {
                children,
                tail: None,
            },
        )?
    } else {
        let expression = parser.expr(0, NORMAL)?;
        if parser.cursor.peek(0)?.is_some() {
            return Err(parser.error("include expected a single expression")?);
        }
        if context == MacroContext::Statements {
            parser.node(
                0,
                Kind::Expansion {
                    children: List { start: 0, end: 0 },
                    tail: Some(expression),
                },
            )?
        } else {
            expression
        }
    };
    parser.ast.root = root;
    parser.ast.lookahead_high_water = parser.cursor.high_water;
    Ok(parser.ast)
}

/// Parse the actual compiler `pattern_type` input once, including opaque bases.
/// # Errors
/// Rejects malformed syntax and currently unsupported range/or constraints.
pub(crate) fn type_pattern_with_inputs<'s>(
    source: &'s str,
    tokens: &'s [Token],
    inputs: &dyn crate::syntax::interpolation::Inputs,
) -> Result<Ast<'s>, LexError> {
    let mut parser = Parser::with_cursor(
        Cursor::from_position(source, tokens, TokenPosition::default())?,
        true,
    );
    parser.inputs = inputs;
    parser.has_opaque = !inputs.is_empty();
    let base = parser.ty()?;
    let start = parser.ast.node(base).span.start;
    let separator = parser.required()?;
    if source.get(separator.start as usize..separator.end as usize) != Some("is") {
        return Err(crate::syntax::token_tree::error(
            separator.start,
            "expected is in pattern type",
        ));
    }
    let constraint = if parser.eat(P::Bang)? {
        let null = parser.required()?;
        if source.get(null.start as usize..null.end as usize) != Some("null") {
            return Err(crate::syntax::token_tree::error(
                null.start,
                "expected null",
            ));
        }
        crate::syntax::ast::TypeConstraint::NonNull
    } else {
        let pattern = parser.pattern(true)?;
        crate::syntax::ast::TypeConstraint::Pattern(pattern)
    };
    if parser.cursor.peek(0)?.is_some() {
        return Err(parser.error("unexpected token after pattern type")?);
    }
    let root = parser.node(start, Kind::TypePattern { base, constraint })?;
    if let crate::syntax::ast::TypeConstraint::Pattern(pattern) = constraint {
        parser.lower_pattern_endpoints(root, pattern)?;
    }
    parser.ast.root = root;
    parser.ast.lookahead_high_water = parser.cursor.high_water;
    Ok(parser.ast)
}
