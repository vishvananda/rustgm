//! Pratt expression parser over bounded lookahead and append-only typed arenas.
use super::{
    ast::{Ast, BlockFlavor, Kind, List, Node, NodeId, Unary},
    cursor::Cursor,
    token_tree::{Span, error},
};
use crate::{
    lex::{
        keyword::{self, Word},
        token::{Kind as TokenKind, LexError, Punctuation as P, Token},
    },
    literal,
};

mod control;
pub mod fragment;
mod impl_restrictions;
mod items;
mod statement_modes;
pub use items::{parse_crate, parse_crate_in_edition, parse_crate_with};
mod attributes;
mod interpolate;
use fragment::FragmentKind as F;
mod pattern;
pub mod render;
mod types;

#[derive(Clone, Copy)]
struct Context {
    structures: bool,
    condition: bool,
    statement: bool,
}
const NORMAL: Context = Context {
    structures: true,
    condition: false,
    statement: false,
};
const CONDITION: Context = Context {
    structures: false,
    condition: true,
    statement: false,
};

pub struct Parser<'source, 'inputs> {
    cursor: Cursor<'source>,
    ast: Ast<'source>,
    scratch: Vec<NodeId>,
    depth: u16,
    condition_work: Vec<(NodeId, bool)>,
    extended: bool,
    item_scope: items::Scope,
    configured_items: bool,
    inputs: &'inputs dyn super::interpolation::Inputs,
    has_opaque: bool,
    /// The crate's interner, when the caller shares it with resolution.
    names: Option<&'inputs mut crate::support::names::Names>,
}

/// Parse one expression fragment, including a block of statements.
///
/// # Errors
/// Rejects malformed syntax, invalid demanded literals and nesting beyond 256.
pub fn parse(source: &str) -> Result<Ast<'_>, LexError> {
    let mut parser = Parser::new(source, false)?;
    let root = parser.expr(0, NORMAL)?;
    if parser.cursor.peek(0)?.is_some() {
        return Err(parser.error("unexpected token after expression")?);
    }
    parser.ast.root = root;
    parser.ast.lookahead_high_water = parser.cursor.high_water;
    Ok(parser.ast)
}

impl<'source> Parser<'source, '_> {
    /// A whole-source parse reserves its node and edge arenas from the source
    /// length (dense code yields about one node per two bytes), so they do not
    /// grow by repeated doubling. Untouched capacity is never written.
    fn new(source: &'source str, extended: bool) -> Result<Self, LexError> {
        let mut parser = Self::with_cursor(Cursor::new(source)?, extended);
        parser.ast.nodes.reserve(source.len() / 2 + 1);
        parser.ast.edges.reserve(source.len() / 4 + 1);
        Ok(parser)
    }
    fn with_cursor(cursor: Cursor<'source>, extended: bool) -> Self {
        Self {
            cursor,
            ast: Ast::default(),
            scratch: Vec::new(),
            depth: 0,
            condition_work: Vec::new(),
            extended,
            item_scope: items::Scope::Module,
            configured_items: false,
            inputs: &super::interpolation::EMPTY_INPUTS,
            has_opaque: false,
            names: None,
        }
    }
    fn nesting_limit(&self) -> u16 {
        if self.extended { 128 } else { 256 }
    }

    fn error(&mut self, message: &'static str) -> Result<LexError, LexError> {
        Ok(error(
            self.cursor
                .peek(0)?
                .map_or(self.cursor.end(), |token| token.start),
            message,
        ))
    }
    // Forced only without debug assertions: debug builds keep the call so
    // recursive-descent frames stay small enough for the nesting limit.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token grammar helper over an inlined cursor probe; callgrind shows calls dominate"
        )
    )]
    fn required(&mut self) -> Result<Token, LexError> {
        match self.cursor.take()? {
            Some(token) => Ok(token),
            None => Err(error(self.cursor.end(), "unexpected end of expression")),
        }
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token grammar helper over an inlined cursor probe; callgrind shows calls dominate"
        )
    )]
    fn eat(&mut self, punctuation: P) -> Result<bool, LexError> {
        self.cursor.eat(punctuation)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token grammar helper over an inlined cursor probe; callgrind shows calls dominate"
        )
    )]
    fn expect(&mut self, punctuation: P) -> Result<Token, LexError> {
        if !self.cursor.at(punctuation)? {
            return Err(self.error("expected punctuation")?);
        }
        self.required()
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token grammar helper over an inlined cursor probe; callgrind shows calls dominate"
        )
    )]
    fn eat_word(&mut self, word: Word) -> Result<bool, LexError> {
        self.cursor.eat_word(word)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-token grammar helper over an inlined cursor probe; callgrind shows calls dominate"
        )
    )]
    fn node(&mut self, start: u32, kind: Kind) -> Result<NodeId, LexError> {
        let id = u32::try_from(self.ast.nodes.len())
            .ok()
            .and_then(NodeId::checked)
            .ok_or_else(|| error(start, "too many syntax nodes"))?;
        self.ast.nodes.push(Node {
            span: Span {
                start,
                end: self.cursor.end(),
            },
            kind,
        });
        Ok(id)
    }
    /// A literal token whose value configuration interprets only if active.
    fn deferred_literal(&mut self, token: Token) -> Result<NodeId, LexError> {
        let id = self.node(token.start, Kind::DeferredLiteral(token))?;
        self.ast.pending.literals.push(id);
        Ok(id)
    }
    /// A name node for a consumed identifier or lifetime token.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-segment node constructor; callgrind shows calls dominate"
        )
    )]
    fn name_node(&mut self, token: Token) -> Result<NodeId, LexError> {
        let span = Span {
            start: token.start,
            end: token.end,
        };
        // A plain identifier has no raw prefix or lifetime quote to strip.
        let name = match self.names.as_deref_mut() {
            Some(names) if token.kind == TokenKind::Identifier => Some(
                names
                    .intern_plain(self.cursor.spelling(token))
                    .map_err(|message| error(span.start, message))?,
            ),
            _ => self.intern(span)?,
        };
        self.node(span.start, Kind::Name(span, name))
    }
    /// A name node interned once here when the parser shares the crate's
    /// names, with the same raw-prefix and NFC rules as later lookups.
    fn name_span(&mut self, span: Span) -> Result<NodeId, LexError> {
        let name = self.intern(span)?;
        self.node(span.start, Kind::Name(span, name))
    }
    fn intern(&mut self, span: Span) -> Result<Option<crate::support::names::NameId>, LexError> {
        let Some(names) = self.names.as_deref_mut() else {
            return Ok(None);
        };
        let spelling = self
            .cursor
            .source
            .get(span.start as usize..span.end as usize)
            .ok_or_else(|| error(span.start, "invalid name span"))?;
        names
            .intern(spelling)
            .map(Some)
            .map_err(|message| error(span.start, message))
    }
    /// `list` for exactly one child, without staging it in scratch.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-path edge append; callgrind shows calls dominate"
        )
    )]
    fn single(&mut self, child: NodeId) -> Result<List, LexError> {
        let start = u32::try_from(self.ast.edges.len())
            .map_err(|_| error(self.cursor.end(), "too many syntax edges"))?;
        self.ast.edges.push(child);
        let end = u32::try_from(self.ast.edges.len())
            .map_err(|_| error(self.cursor.end(), "too many syntax edges"))?;
        Ok(List { start, end })
    }
    fn list(&mut self, mark: usize) -> Result<List, LexError> {
        let start = u32::try_from(self.ast.edges.len())
            .map_err(|_| error(self.cursor.end(), "too many syntax edges"))?;
        self.ast.edges.extend(self.scratch.drain(mark..));
        let end = u32::try_from(self.ast.edges.len())
            .map_err(|_| error(self.cursor.end(), "too many syntax edges"))?;
        Ok(List { start, end })
    }
    fn expr(&mut self, minimum: u8, context: Context) -> Result<NodeId, LexError> {
        if self.depth >= self.nesting_limit() {
            return Err(self.error(if self.extended {
                "crate syntax nesting exceeds 128"
            } else {
                "expression nesting exceeds 256"
            })?);
        }
        self.depth += 1;
        let attrs = if self.extended {
            self.attributes(false)?
        } else {
            super::item::AttributeRange::default()
        };
        let result = self
            .binding_power(minimum, context)
            .and_then(|value| self.attach(attrs, value));
        self.depth -= 1;
        result
    }
    fn binding_power(&mut self, minimum: u8, context: Context) -> Result<NodeId, LexError> {
        let mut left = self.prefix(Context {
            condition: context.condition && minimum <= 41,
            ..context
        })?;
        while let Some((token, word)) = self.cursor.current()? {
            let node = self.ast.node(left);
            let start = node.span.start;
            let op = match token.kind {
                TokenKind::Punctuation(op) => op,
                TokenKind::Identifier if word == Word::As => {
                    if (context.statement && self.with_block(left)) || minimum > 120 {
                        break;
                    }
                    left = self.cast(left, start)?;
                    continue;
                }
                _ => break,
            };
            if matches!(op, P::Dot | P::Question | P::OpenParen | P::OpenBracket)
                && matches!(node.kind, Kind::Cast { .. })
            {
                return Err(error(
                    token.start,
                    "cast followed by a postfix operator requires parentheses",
                ));
            }
            if matches!(op, P::Dot | P::Question) {
                left = self.postfix(left, op)?;
                continue;
            }
            if context.statement && self.with_block(left) {
                break;
            }
            if matches!(op, P::OpenParen | P::OpenBracket) {
                left = self.postfix(left, op)?;
                continue;
            }
            let Some((power, right_power)) = precedence(op) else {
                break;
            };
            if power < minimum {
                break;
            }
            if (power == 50
                && matches!(node.kind, Kind::Binary { op: old, .. } if precedence(old).is_some_and(|(p, _)| p == 50)))
                || (power == 20 && matches!(node.kind, Kind::Range { .. }))
            {
                return Err(error(
                    token.start,
                    "comparison and range operators require parentheses",
                ));
            }
            self.cursor.take()?;
            if matches!(op, P::DotDot | P::DotDotEqual) {
                let end = if self.can_start()? {
                    Some(self.expr(
                        right_power,
                        Context {
                            statement: false,
                            ..context
                        },
                    )?)
                } else {
                    None
                };
                if op == P::DotDotEqual && end.is_none() {
                    return Err(error(token.start, "inclusive range requires an end"));
                }
                left = self.node(
                    start,
                    Kind::Range {
                        inclusive: op == P::DotDotEqual,
                        start: Some(left),
                        end,
                    },
                )?;
            } else {
                let right = self.expr(
                    right_power,
                    Context {
                        statement: false,
                        ..context
                    },
                )?;
                left = self.node(start, Kind::Binary { op, left, right })?;
            }
        }
        Ok(left)
    }

    /// Consume the observed `as` and its target type.
    fn cast(&mut self, value: NodeId, start: u32) -> Result<NodeId, LexError> {
        self.required()?;
        let ty = self.ty()?;
        if matches!(
            self.cursor.last_kind(),
            Some(TokenKind::Identifier | TokenKind::RawIdentifier)
        ) && (self.cursor.at(P::Less)? || self.cursor.at(P::ShiftLeft)?)
        {
            return Err(
                self.error("cast followed by less-than or left shift requires parentheses")?
            );
        }
        self.node(start, Kind::Cast { value, ty })
    }

    fn prefix(&mut self, context: Context) -> Result<NodeId, LexError> {
        if let Some(node) = self.interpolated(&[
            F::Expression,
            F::Expression2021,
            F::Literal,
            F::Block,
            F::Path,
        ])? {
            return Ok(node);
        }
        let Some((token, word)) = self.cursor.current()? else {
            return Err(error(self.cursor.end(), "expected expression"));
        };
        if literal_candidate(token.kind, word) {
            if self.extended && literal::is_token(self.cursor.source, token) {
                self.required()?;
                return self.deferred_literal(token);
            }
            if let Some(value) = literal::interpret(self.cursor.source, token)? {
                self.cursor.take()?;
                let id = u32::try_from(self.ast.literals.len())
                    .map_err(|_| error(token.start, "too many literals"))?;
                self.ast.literals.push(value);
                return self.node(token.start, Kind::Literal(id));
            }
        }
        match token.kind {
            TokenKind::CrateRoot(_) => self.path_expression(context),
            TokenKind::Punctuation(P::Minus | P::Bang | P::Star | P::And | P::AndAnd) => {
                self.unary(context)
            }
            TokenKind::Punctuation(P::OpenParen) => self.parentheses(),
            TokenKind::Punctuation(P::OpenBracket) => self.array(),
            TokenKind::Punctuation(P::OpenBrace) => self.block(BlockFlavor::Plain),
            TokenKind::Punctuation(P::DotDot | P::DotDotEqual) => {
                self.cursor.take()?;
                let inclusive = token.kind == TokenKind::Punctuation(P::DotDotEqual);
                let end = if self.can_start()? {
                    Some(self.expr(21, context)?)
                } else {
                    None
                };
                if inclusive && end.is_none() {
                    return Err(error(token.start, "inclusive range requires an end"));
                }
                self.node(
                    token.start,
                    Kind::Range {
                        inclusive,
                        start: None,
                        end,
                    },
                )
            }
            TokenKind::Punctuation(P::Or | P::OrOr) => self.closure(false, false, token.start),
            TokenKind::Lifetime | TokenKind::RawLifetime => self.labeled(),
            TokenKind::Identifier => match word {
                Word::If => self.if_expression(),
                Word::Loop | Word::While | Word::For => self.loop_expression(),
                Word::Match => self.match_expression(),
                Word::Return | Word::Break | Word::Continue => self.jump(),
                Word::Let if context.condition => self.let_condition(context),
                Word::Move => {
                    self.cursor.take()?;
                    self.closure(true, false, token.start)
                }
                Word::Async => self.async_expression(),
                Word::Unsafe | Word::Const => {
                    self.cursor.take()?;
                    let flavor = if word == Word::Unsafe {
                        BlockFlavor::Unsafe
                    } else {
                        BlockFlavor::Const
                    };
                    let block = self.block(flavor)?;
                    self.ast.nodes[block.get() as usize].span.start = token.start;
                    Ok(block)
                }
                Word::Underscore => {
                    self.cursor.take()?;
                    self.node(token.start, Kind::Wildcard)
                }
                _ => self.path_expression(context),
            },
            TokenKind::Punctuation(P::Less | P::ShiftLeft) if self.extended => {
                self.path_expression(context)
            }
            TokenKind::RawIdentifier | TokenKind::Punctuation(P::ColonColon) => {
                self.path_expression(context)
            }
            _ => Err(error(token.start, "expected expression")),
        }
    }

    fn unary(&mut self, context: Context) -> Result<NodeId, LexError> {
        let token = if self.cursor.at(P::AndAnd)? {
            self.cursor.split(P::And, P::And)?
        } else {
            self.required()?
        };
        let raw_borrow = self.cursor.at_word(Word::Raw)?
            && matches!(self.cursor.peek_word(1)?, Word::Const | Word::Mut);
        let op = match token.kind {
            TokenKind::Punctuation(P::Minus) => Unary::Negate,
            TokenKind::Punctuation(P::Bang) => Unary::Not,
            TokenKind::Punctuation(P::Star) => Unary::Dereference,
            _ if self.eat_word(Word::Mut)? => Unary::BorrowMut,
            _ if raw_borrow => {
                self.required()?;
                if self.eat_word(Word::Const)? {
                    Unary::RawConst
                } else if self.eat_word(Word::Mut)? {
                    Unary::RawMut
                } else {
                    return Err(self.error("raw borrow requires const or mut")?);
                }
            }
            _ => Unary::Borrow,
        };
        let operand = self.expr(
            130,
            Context {
                statement: false,
                ..context
            },
        )?;
        self.node(token.start, Kind::Unary { op, operand })
    }

    fn path(&mut self) -> Result<NodeId, LexError> {
        if self.extended {
            return self.rich_path(false);
        }
        self.simple_path()
    }
    fn simple_path(&mut self) -> Result<NodeId, LexError> {
        let start = self
            .cursor
            .peek(0)?
            .map_or(self.cursor.end(), |token| token.start);
        let mark = self.scratch.len();
        let absolute = self.eat(P::ColonColon)?;
        loop {
            let token = self.required()?;
            if !self.path_segment(token, self.cursor.last_word())? {
                return Err(error(token.start, "expected path segment"));
            }
            let segment = self.name_node(token)?;
            if !self.eat(P::ColonColon)? {
                if !absolute && self.scratch.len() == mark {
                    // A relative path of one plain name is that name alone.
                    return Ok(segment);
                }
                self.scratch.push(segment);
                break;
            }
            self.scratch.push(segment);
        }
        let segments = self.list(mark)?;
        self.node(start, Kind::Path { absolute, segments })
    }
    fn path_expression(&mut self, context: Context) -> Result<NodeId, LexError> {
        let path = self.path()?;
        let start = self.ast.node(path).span.start;
        if self.eat(P::Bang)? {
            let opener = self.required()?;
            if !matches!(
                opener.kind,
                TokenKind::Punctuation(P::OpenParen | P::OpenBracket | P::OpenBrace)
            ) {
                return Err(error(opener.start, "expected macro delimiter"));
            }
            let forest = self.cursor.capture(opener)?;
            let capture = u32::try_from(self.ast.captures.len())
                .map_err(|_| error(start, "too many captured fragments"))?;
            self.ast.captures.push(forest);
            return self.node(
                start,
                Kind::Macro {
                    path,
                    capture,
                    context: if context.statement
                        && opener.kind == TokenKind::Punctuation(P::OpenBrace)
                    {
                        crate::syntax::ast::MacroContext::Statements
                    } else {
                        crate::syntax::ast::MacroContext::Expression
                    },
                },
            );
        }
        if context.structures && self.cursor.at(P::OpenBrace)? {
            return self.structure(path);
        }
        Ok(path)
    }

    fn parentheses(&mut self) -> Result<NodeId, LexError> {
        let start = self.required()?.start;
        let mark = self.scratch.len();
        if self.eat(P::CloseParen)? {
            let values = self.list(mark)?;
            return self.node(start, Kind::Tuple(values));
        }
        let first = self.expr(0, NORMAL)?;
        if self.eat(P::CloseParen)? {
            return self.node(start, Kind::Group(first));
        }
        self.expect(P::Comma)?;
        self.scratch.push(first);
        self.comma_expressions(P::CloseParen)?;
        let values = self.list(mark)?;
        self.node(start, Kind::Tuple(values))
    }
    fn comma_expressions(&mut self, close: P) -> Result<(), LexError> {
        while !self.eat(close)? {
            let expression = self.expr(0, NORMAL)?;
            self.scratch.push(expression);
            if self.eat(close)? {
                break;
            }
            self.expect(P::Comma)?;
        }
        Ok(())
    }
    fn array(&mut self) -> Result<NodeId, LexError> {
        let start = self.required()?.start;
        let mark = self.scratch.len();
        if self.eat(P::CloseBracket)? {
            let values = self.list(mark)?;
            return self.node(start, Kind::Array(values));
        }
        let first = self.expr(0, NORMAL)?;
        if self.eat(P::Semicolon)? {
            let count = self.expr(0, NORMAL)?;
            self.expect(P::CloseBracket)?;
            return self.node(
                start,
                Kind::Repeat {
                    value: first,
                    count,
                },
            );
        }
        self.scratch.push(first);
        if !self.eat(P::CloseBracket)? {
            self.expect(P::Comma)?;
            self.comma_expressions(P::CloseBracket)?;
        }
        let values = self.list(mark)?;
        self.node(start, Kind::Array(values))
    }
    fn call_arguments(&mut self) -> Result<List, LexError> {
        let mark = self.scratch.len();
        self.comma_expressions(P::CloseParen)?;
        self.list(mark)
    }

    /// `op` is the already observed postfix punctuation at the cursor.
    fn postfix(&mut self, value: NodeId, op: P) -> Result<NodeId, LexError> {
        let start = self.ast.node(value).span.start;
        self.required()?;
        match op {
            P::Question => {
                return self.node(
                    start,
                    Kind::Unary {
                        op: Unary::Try,
                        operand: value,
                    },
                );
            }
            P::OpenParen => {
                let arguments = self.call_arguments()?;
                return self.node(
                    start,
                    Kind::Call {
                        callee: value,
                        arguments,
                    },
                );
            }
            P::OpenBracket => {
                let index = self.expr(0, NORMAL)?;
                self.expect(P::CloseBracket)?;
                return self.node(start, Kind::Index { value, index });
            }
            _ => {}
        }
        self.member(value, start)
    }

    /// The member access after a consumed `.`: await, field, tuple index or method call.
    fn member(&mut self, value: NodeId, start: u32) -> Result<NodeId, LexError> {
        if self.eat_word(Word::Await)? {
            return self.node(
                start,
                Kind::Unary {
                    op: Unary::Await,
                    operand: value,
                },
            );
        }
        let name = self.cursor.projection()?;
        if !self.field_name(name, self.cursor.last_word())? {
            return Err(error(
                name.start,
                "expected field name or canonical tuple index",
            ));
        }
        if self.extended && self.eat(P::ColonColon)? {
            if name.kind == TokenKind::Integer {
                return Err(error(name.start, "generic method requires an identifier"));
            }
            let method = self.name_node(name)?;
            let (arguments, output, parenthesized) = self.generic_arguments()?;
            let method = self.node(
                name.start,
                Kind::Applied {
                    base: method,
                    arguments,
                    output,
                    parenthesized,
                },
            )?;
            self.expect(P::OpenParen)?;
            let arguments = self.call_arguments()?;
            return self.node(
                start,
                Kind::GenericMethodCall {
                    receiver: value,
                    method,
                    arguments,
                },
            );
        }
        if name.kind != TokenKind::Integer && self.eat(P::OpenParen)? {
            let arguments = self.call_arguments()?;
            return self.node(
                start,
                Kind::MethodCall {
                    receiver: value,
                    name: Span {
                        start: name.start,
                        end: name.end,
                    },
                    arguments,
                },
            );
        }
        self.node(
            start,
            Kind::Field {
                value,
                name: Span {
                    start: name.start,
                    end: name.end,
                },
            },
        )
    }
    fn structure(&mut self, path: NodeId) -> Result<NodeId, LexError> {
        let start = self.ast.node(path).span.start;
        self.expect(P::OpenBrace)?;
        let mark = self.scratch.len();
        let mut base = None;
        while !self.eat(P::CloseBrace)? {
            let attrs = if self.extended {
                self.attributes(false)?
            } else {
                super::item::AttributeRange::default()
            };
            if self.eat(P::DotDot)? {
                base = Some(self.expr(0, NORMAL)?);
                self.eat(P::Comma)?;
                self.expect(P::CloseBrace)?;
                break;
            }
            let token = self.required()?;
            if !self.field_name(token, self.cursor.last_word())? {
                return Err(error(token.start, "expected struct field"));
            }
            let name = Span {
                start: token.start,
                end: token.end,
            };
            let value = if self.eat(P::Colon)? {
                self.expr(0, NORMAL)?
            } else if token.kind != TokenKind::Integer {
                self.name_span(name)?
            } else {
                return Err(error(token.start, "numeric struct field requires a value"));
            };
            let field = self.node(token.start, Kind::FieldInit { name, value })?;
            let field = self.attach(attrs, field)?;
            self.scratch.push(field);
            if self.eat(P::CloseBrace)? {
                break;
            }
            self.expect(P::Comma)?;
        }
        let fields = self.list(mark)?;
        self.node(start, Kind::Struct { path, fields, base })
    }

    /// `word` is the cursor classification of `token` when it was consumed.
    fn field_name(&self, token: Token, word: Word) -> Result<bool, LexError> {
        if self.identifier(token, word)? {
            return Ok(true);
        }
        let spelling = self.cursor.spelling(token);
        Ok(token.kind == TokenKind::Integer
            && spelling.bytes().all(|b| b.is_ascii_digit())
            && (spelling == "0" || !spelling.starts_with('0')))
    }

    /// `keyword::identifier` over a consumed token, using its classified word.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-segment keyword check; callgrind shows calls dominate"
        )
    )]
    fn identifier(&self, token: Token, word: Word) -> Result<bool, LexError> {
        Ok(match token.kind {
            TokenKind::Identifier if word == Word::Gen => {
                !word.reserved_in(self.inputs.edition(token)?)
            }
            TokenKind::Identifier => !word.reserved(),
            TokenKind::RawIdentifier => {
                let spelling = self.cursor.spelling(token);
                keyword::identifier(token.kind, spelling.strip_prefix("r#").unwrap_or(spelling))
            }
            _ => false,
        })
    }

    /// An identifier or one of the path keywords `self`, `Self`, `super` and `crate`.
    fn path_segment(&self, token: Token, word: Word) -> Result<bool, LexError> {
        Ok(self.identifier(token, word)?
            || (token.kind == TokenKind::Identifier && word.path_keyword()))
    }

    fn can_start(&mut self) -> Result<bool, LexError> {
        let Some((token, word)) = self.cursor.current()? else {
            return Ok(false);
        };
        Ok(match token.kind {
            TokenKind::Punctuation(punctuation) => matches!(
                punctuation,
                P::Minus
                    | P::Bang
                    | P::Star
                    | P::And
                    | P::AndAnd
                    | P::Or
                    | P::OrOr
                    | P::OpenParen
                    | P::OpenBracket
                    | P::OpenBrace
                    | P::DotDot
                    | P::DotDotEqual
                    | P::ColonColon
            ),
            TokenKind::Identifier => !matches!(word, Word::Else | Word::In | Word::As),
            TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment => false,
            _ => true,
        })
    }
    fn with_block(&self, mut expression: NodeId) -> bool {
        while let Kind::Attributed { value, .. } = self.ast.node(expression).kind {
            expression = value;
        }
        match self.ast.node(expression).kind {
            Kind::Block {
                flavor: BlockFlavor::Plain | BlockFlavor::Unsafe | BlockFlavor::Const,
                ..
            }
            | Kind::If { .. }
            | Kind::Loop { .. }
            | Kind::While { .. }
            | Kind::For { .. }
            | Kind::Match { .. }
            | Kind::Label { .. } => true,
            Kind::Macro { capture, .. } => {
                self.ast.captures[capture as usize]
                    .tree(0)
                    .is_some_and(|tree| {
                        matches!(
                            tree.kind,
                            super::token_tree::TreeKind::Group {
                                delimiter: super::token_tree::Delimiter::Brace,
                                ..
                            }
                        )
                    })
            }
            _ => false,
        }
    }
}

/// Only literal categories and `true`/`false` can start a literal expression.
fn literal_candidate(kind: TokenKind, word: Word) -> bool {
    match kind {
        TokenKind::Identifier => matches!(word, Word::True | Word::False),
        TokenKind::Integer
        | TokenKind::Float
        | TokenKind::Character
        | TokenKind::Byte
        | TokenKind::String
        | TokenKind::ByteString
        | TokenKind::CString
        | TokenKind::RawString
        | TokenKind::RawByteString
        | TokenKind::RawCString => true,
        _ => false,
    }
}

#[cfg_attr(not(debug_assertions), inline(always))]
#[cfg_attr(
    not(debug_assertions),
    expect(
        clippy::inline_always,
        reason = "per-operator table lookup; callgrind shows calls dominate"
    )
)]
fn precedence(op: P) -> Option<(u8, u8)> {
    let power = match op {
        P::Equal
        | P::PlusEqual
        | P::MinusEqual
        | P::StarEqual
        | P::SlashEqual
        | P::PercentEqual
        | P::CaretEqual
        | P::AndEqual
        | P::OrEqual
        | P::ShiftLeftEqual
        | P::ShiftRightEqual => return Some((10, 10)),
        P::DotDot | P::DotDotEqual => 20,
        P::OrOr => 30,
        P::AndAnd => 40,
        P::EqualEqual | P::BangEqual | P::Less | P::Greater | P::LessEqual | P::GreaterEqual => 50,
        P::Or => 60,
        P::Caret => 70,
        P::And => 80,
        P::ShiftLeft | P::ShiftRight => 90,
        P::Plus | P::Minus => 100,
        P::Star | P::Slash | P::Percent => 110,
        _ => return None,
    };
    Some((power, power + 1))
}
