//! Pratt expression parser over bounded lookahead and append-only typed arenas.
use super::{
    ast::{Ast, BlockFlavor, Kind, List, Node, NodeId, Unary},
    cursor::Cursor,
    token_tree::{Span, error},
};
use crate::{
    lex::{
        keyword,
        token::{Kind as TokenKind, LexError, Punctuation as P, Token},
    },
    literal,
};

mod control;
pub mod fragment;
mod items;
pub use items::parse_crate;
mod attributes;
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

pub struct Parser<'source> {
    cursor: Cursor<'source>,
    ast: Ast<'source>,
    scratch: Vec<NodeId>,
    depth: u16,
    condition_work: Vec<(NodeId, bool)>,
    extended: bool,
    item_scope: items::Scope,
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

impl<'source> Parser<'source> {
    fn new(source: &'source str, extended: bool) -> Result<Self, LexError> {
        Ok(Self::with_cursor(Cursor::new(source)?, extended))
    }
    fn with_cursor(cursor: Cursor<'source>, extended: bool) -> Self {
        Self {
            cursor,
            ast: Ast {
                nodes: Vec::new(),
                edges: Vec::new(),
                literals: Vec::new(),
                captures: Vec::new(),
                root: NodeId(0),
                items: Vec::new(),
                variants: Vec::new(),
                use_trees: Vec::new(),
                generics: Vec::new(),
                function_types: Vec::new(),
                attributes: Vec::new(),
                lookahead_high_water: 0,
            },
            scratch: Vec::new(),
            depth: 0,
            condition_work: Vec::new(),
            extended,
            item_scope: items::Scope::Module,
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
    fn required(&mut self) -> Result<Token, LexError> {
        self.cursor
            .take()?
            .ok_or_else(|| error(self.cursor.end(), "unexpected end of expression"))
    }
    fn eat(&mut self, punctuation: P) -> Result<bool, LexError> {
        if self.cursor.at(punctuation)? {
            self.cursor.take()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn expect(&mut self, punctuation: P) -> Result<Token, LexError> {
        if !self.cursor.at(punctuation)? {
            return Err(self.error("expected punctuation")?);
        }
        self.required()
    }
    fn word(&mut self, word: &str) -> Result<bool, LexError> {
        if self.cursor.word(word)? {
            self.cursor.take()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn node(&mut self, start: u32, kind: Kind) -> Result<NodeId, LexError> {
        let id = u32::try_from(self.ast.nodes.len())
            .map_err(|_| error(start, "too many syntax nodes"))?;
        self.ast.nodes.push(Node {
            span: Span {
                start,
                end: self.cursor.end(),
            },
            kind,
        });
        Ok(NodeId(id))
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
        while let Some(token) = self.cursor.peek(0)? {
            let start = self.ast.node(left).span.start;
            if matches!(self.ast.node(left).kind, Kind::Cast { .. })
                && matches!(
                    token.kind,
                    TokenKind::Punctuation(P::Dot | P::Question | P::OpenParen | P::OpenBracket)
                )
            {
                return Err(error(
                    token.start,
                    "cast followed by a postfix operator requires parentheses",
                ));
            }
            if self.cursor.at(P::Dot)? || self.cursor.at(P::Question)? {
                left = self.postfix(left)?;
                continue;
            }
            if context.statement && self.with_block(left) {
                break;
            }
            if self.cursor.at(P::OpenParen)? || self.cursor.at(P::OpenBracket)? {
                left = self.postfix(left)?;
                continue;
            }
            if self.cursor.word("as")? {
                if minimum > 120 {
                    break;
                }
                self.cursor.take()?;
                let ty = self.ty()?;
                if matches!(
                    self.cursor.last_kind(),
                    Some(TokenKind::Identifier | TokenKind::RawIdentifier)
                ) && (self.cursor.at(P::Less)? || self.cursor.at(P::ShiftLeft)?)
                {
                    return Err(self
                        .error("cast followed by less-than or left shift requires parentheses")?);
                }
                left = self.node(start, Kind::Cast { value: left, ty })?;
                continue;
            }
            let TokenKind::Punctuation(op) = token.kind else {
                break;
            };
            let Some((power, right_power)) = precedence(op) else {
                break;
            };
            if power < minimum {
                break;
            }
            if (power == 50
                && matches!(self.ast.node(left).kind, Kind::Binary { op: old, .. } if precedence(old).is_some_and(|(p, _)| p == 50)))
                || (power == 20 && matches!(self.ast.node(left).kind, Kind::Range { .. }))
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

    fn prefix(&mut self, context: Context) -> Result<NodeId, LexError> {
        let token = self
            .cursor
            .peek(0)?
            .ok_or_else(|| error(self.cursor.end(), "expected expression"))?;
        if self.extended && literal::is_token(self.cursor.source, token) {
            self.required()?;
            return self.node(token.start, Kind::DeferredLiteral(token));
        }
        if let Some(value) = literal::interpret(self.cursor.source, token)? {
            self.cursor.take()?;
            let id = u32::try_from(self.ast.literals.len())
                .map_err(|_| error(token.start, "too many literals"))?;
            self.ast.literals.push(value);
            return self.node(token.start, Kind::Literal(id));
        }
        match token.kind {
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
            TokenKind::Identifier => match self.cursor.spelling(token) {
                "if" => self.if_expression(),
                "loop" | "while" | "for" => self.loop_expression(),
                "match" => self.match_expression(),
                "return" | "break" | "continue" => self.jump(),
                "let" if context.condition => self.let_condition(),
                "move" => {
                    self.cursor.take()?;
                    self.closure(true, false, token.start)
                }
                "async" => self.async_expression(),
                "unsafe" | "const" => {
                    self.cursor.take()?;
                    let flavor = if self.cursor.spelling(token) == "unsafe" {
                        BlockFlavor::Unsafe
                    } else {
                        BlockFlavor::Const
                    };
                    let block = self.block(flavor)?;
                    self.ast.nodes[block.0 as usize].span.start = token.start;
                    Ok(block)
                }
                "_" => {
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
        let raw_borrow = self.cursor.word("raw")?
            && self.cursor.peek(1)?.is_some_and(|next| {
                next.kind == TokenKind::Identifier
                    && matches!(self.cursor.spelling(next), "const" | "mut")
            });
        let op = match token.kind {
            TokenKind::Punctuation(P::Minus) => Unary::Negate,
            TokenKind::Punctuation(P::Bang) => Unary::Not,
            TokenKind::Punctuation(P::Star) => Unary::Dereference,
            _ if self.word("mut")? => Unary::BorrowMut,
            _ if raw_borrow => {
                self.required()?;
                if self.word("const")? {
                    Unary::RawConst
                } else if self.word("mut")? {
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
            let name = self
                .cursor
                .spelling(token)
                .strip_prefix("r#")
                .unwrap_or(self.cursor.spelling(token));
            if !(keyword::identifier(token.kind, name)
                || (token.kind == TokenKind::Identifier
                    && matches!(name, "self" | "Self" | "super" | "crate")))
            {
                return Err(error(token.start, "expected path segment"));
            }
            let segment = self.node(
                token.start,
                Kind::Name(Span {
                    start: token.start,
                    end: token.end,
                }),
            )?;
            self.scratch.push(segment);
            if !self.eat(P::ColonColon)? {
                break;
            }
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
            return self.node(start, Kind::Macro { path, capture });
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

    fn postfix(&mut self, value: NodeId) -> Result<NodeId, LexError> {
        let start = self.ast.node(value).span.start;
        if self.eat(P::Question)? {
            return self.node(
                start,
                Kind::Unary {
                    op: Unary::Try,
                    operand: value,
                },
            );
        }
        if self.eat(P::OpenParen)? {
            let arguments = self.call_arguments()?;
            return self.node(
                start,
                Kind::Call {
                    callee: value,
                    arguments,
                },
            );
        }
        if self.eat(P::OpenBracket)? {
            let index = self.expr(0, NORMAL)?;
            self.expect(P::CloseBracket)?;
            return self.node(start, Kind::Index { value, index });
        }
        self.expect(P::Dot)?;
        if self.word("await")? {
            return self.node(
                start,
                Kind::Unary {
                    op: Unary::Await,
                    operand: value,
                },
            );
        }
        let name = self.cursor.projection()?;
        if !self.field_name(name) {
            return Err(error(
                name.start,
                "expected field name or canonical tuple index",
            ));
        }
        if self.extended && self.eat(P::ColonColon)? {
            if name.kind == TokenKind::Integer {
                return Err(error(name.start, "generic method requires an identifier"));
            }
            let method = self.node(
                name.start,
                Kind::Name(Span {
                    start: name.start,
                    end: name.end,
                }),
            )?;
            let (arguments, output) = self.generic_arguments()?;
            let method = self.node(
                name.start,
                Kind::Applied {
                    base: method,
                    arguments,
                    output,
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
            if !self.field_name(token) {
                return Err(error(token.start, "expected struct field"));
            }
            let name = Span {
                start: token.start,
                end: token.end,
            };
            let value = if self.eat(P::Colon)? {
                self.expr(0, NORMAL)?
            } else if token.kind != TokenKind::Integer {
                self.node(token.start, Kind::Name(name))?
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

    fn field_name(&self, token: Token) -> bool {
        let spelling = self.cursor.spelling(token);
        keyword::identifier(token.kind, spelling.strip_prefix("r#").unwrap_or(spelling))
            || (token.kind == TokenKind::Integer
                && spelling.bytes().all(|b| b.is_ascii_digit())
                && (spelling == "0" || !spelling.starts_with('0')))
    }

    fn can_start(&mut self) -> Result<bool, LexError> {
        let Some(token) = self.cursor.peek(0)? else {
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
            TokenKind::Identifier => !matches!(self.cursor.spelling(token), "else" | "in" | "as"),
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
