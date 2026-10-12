//! Macro nonterminals call the ordinary grammar over classified tokens.
use super::{Ast, BlockFlavor, Context, Kind, LexError, NORMAL, NodeId, P, Parser, items};
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
    let mut parser = Parser::with_cursor(Cursor::from_position(source, tokens, start)?, true);
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
                && (parser.cursor.word("_")? || parser.cursor.word("const")?)
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
impl Parser<'_> {
    fn fragment_statement(&mut self) -> Result<NodeId, LexError> {
        if self.starts_item()? {
            return self.item_in(items::Scope::Module);
        }
        if self.cursor.at(P::Semicolon)? {
            let start = self.required()?.start;
            return self.node(start, Kind::EmptyStatement);
        }
        if self.cursor.word("let")? {
            return self.let_statement(false);
        }
        let value = self.expr(
            0,
            Context {
                statement: true,
                ..NORMAL
            },
        )?;
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
    let mut parser = Parser::with_cursor(Cursor::from_position(source, tokens, start)?, true);
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
    let mut parser = Parser::with_cursor(Cursor::from_position(source, tokens, start)?, true);
    let first = parser
        .cursor
        .peek(0)?
        .ok_or_else(|| super::error(parser.cursor.end(), "expected metadata fragment"))?;
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
    while cursor.token_position()? != position {
        let token = cursor
            .take()?
            .ok_or_else(|| super::error(end, "invalid metadata token range"))?;
        builder.push(source, token)?;
    }
    let forest = builder.finish(end)?;
    let attribute = crate::syntax::attribute::parse(
        &forest,
        source,
        forest.roots(),
        crate::syntax::attribute::Style::Outer,
        crate::syntax::token_tree::Span {
            start: first.start,
            end,
        },
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
