//! Statement boundaries and control expressions share the ordinary cursor.
use super::{
    BlockFlavor, CONDITION, Context, Kind, LexError, NORMAL, NodeId, P, Parser, Span, TokenKind,
    error,
};

use super::fragment::FragmentKind as F;

impl Parser<'_, '_> {
    pub(super) fn block(&mut self, flavor: BlockFlavor) -> Result<NodeId, LexError> {
        if let Some(node) = self.interpolated(&[F::Block])? {
            return Ok(node);
        }
        let start = self.expect(P::OpenBrace)?.start;
        let inner = if self.extended {
            self.attributes(true)?
        } else {
            crate::syntax::item::AttributeRange::default()
        };
        let (statements, tail) = self.statement_list(Some(P::CloseBrace))?;
        let block = self.node(
            start,
            Kind::Block {
                flavor,
                statements,
                tail,
            },
        )?;
        self.attach(inner, block)
    }

    fn end_statements(&mut self, close: Option<P>) -> Result<bool, LexError> {
        if let Some(close) = close {
            self.eat(close)
        } else {
            Ok(self.cursor.peek(0)?.is_none())
        }
    }
    pub(super) fn statement_list(
        &mut self,
        close: Option<P>,
    ) -> Result<(super::List, Option<NodeId>), LexError> {
        let mark = self.scratch.len();
        let mut tail = None;
        while !self.end_statements(close)? {
            let attrs = if self.extended {
                self.attributes(false)?
            } else {
                crate::syntax::item::AttributeRange::default()
            };
            let statement = if let Some(statement) = self.interpolated(&[F::Statement])? {
                if !self.eat(P::Semicolon)?
                    && let Kind::Statement(value) = self.ast.node(statement).kind
                    && self.end_statements(close)?
                {
                    tail = Some(self.attach(attrs, value)?);
                    break;
                }
                statement
            } else if self.extended && self.starts_item()? {
                self.item_in(super::items::Scope::Module)?
            } else if self.cursor.at(P::Semicolon)? {
                let token = self.required()?;
                self.node(token.start, Kind::EmptyStatement)?
            } else if self.cursor.word("let")? {
                self.let_statement(true)?
            } else {
                let value = self.expr(
                    0,
                    Context {
                        statement: true,
                        ..NORMAL
                    },
                )?;
                if self.eat(P::Semicolon)? {
                    self.statement_macro(value);
                    self.node(self.ast.node(value).span.start, Kind::Statement(value))?
                } else if self.end_statements(close)? {
                    tail = Some(self.attach(attrs, value)?);
                    break;
                } else if self.with_block(value) {
                    self.node(self.ast.node(value).span.start, Kind::Statement(value))?
                } else {
                    return Err(self.error("expected semicolon or end of block")?);
                }
            };
            let statement = self.attach(attrs, statement)?;
            self.scratch.push(statement);
        }
        let statements = self.list(mark)?;
        Ok((statements, tail))
    }
    pub(super) fn statement_macro(&mut self, mut node: NodeId) {
        while let Kind::Attributed { value, .. } = self.ast.node(node).kind {
            node = value;
        }
        if let Kind::Macro { context, .. } = &mut self.ast.nodes[node.0 as usize].kind {
            *context = crate::syntax::ast::MacroContext::Statements;
        }
    }

    pub(super) fn let_statement(&mut self, semicolon: bool) -> Result<NodeId, LexError> {
        let start = self.required()?.start;
        let pattern = self.pattern(false)?;
        let ty = if self.eat(P::Colon)? {
            Some(self.ty()?)
        } else {
            None
        };
        let value = if self.eat(P::Equal)? {
            Some(self.expr(0, NORMAL)?)
        } else {
            None
        };
        let brace_ending = self.cursor.last_kind() == Some(TokenKind::Punctuation(P::CloseBrace));
        let otherwise = if self.word("else")? {
            let Some(value) = value else {
                return Err(error(start, "let else requires an initializer"));
            };
            if brace_ending {
                return Err(error(
                    start,
                    "let else initializer ending in a brace requires parentheses",
                ));
            }
            if matches!(
                self.ast.node(value).kind,
                Kind::Binary {
                    op: P::AndAnd | P::OrOr,
                    ..
                }
            ) {
                return Err(error(
                    start,
                    "let else lazy boolean initializer requires parentheses",
                ));
            }
            Some(self.block(BlockFlavor::Plain)?)
        } else {
            None
        };
        if semicolon {
            self.expect(P::Semicolon)?;
        }
        self.node(
            start,
            Kind::Let {
                pattern,
                ty,
                value,
                otherwise,
            },
        )
    }

    pub(super) fn let_condition(&mut self, context: Context) -> Result<NodeId, LexError> {
        let start = self.required()?.start;
        let pattern = self.pattern(true)?;
        self.expect(P::Equal)?;
        let value = self.expr(
            41,
            Context {
                condition: false,
                ..context
            },
        )?;
        self.node(start, Kind::LetCondition { pattern, value })
    }

    pub(super) fn if_expression(&mut self) -> Result<NodeId, LexError> {
        let start = self.required()?.start;
        let condition = self.expr(0, CONDITION)?;
        self.validate_condition(condition)?;
        let then = self.block(BlockFlavor::Plain)?;
        let otherwise = if self.word("else")? {
            Some(if self.cursor.word("if")? {
                self.expr(
                    0,
                    Context {
                        statement: true,
                        ..NORMAL
                    },
                )?
            } else {
                self.block(BlockFlavor::Plain)?
            })
        } else {
            None
        };
        self.node(
            start,
            Kind::If {
                condition,
                then,
                otherwise,
            },
        )
    }

    fn validate_condition(&mut self, root: NodeId) -> Result<(), LexError> {
        // A condition's top-level conjunction may contain let expressions. A let
        // under ||, parentheses or another operator is not a let-chain operand.
        self.condition_work.clear();
        self.condition_work.push((root, true));
        let mut has_let = false;
        let mut excluded = None;
        while let Some((id, allowed)) = self.condition_work.pop() {
            let node = self.ast.node(id);
            match node.kind {
                Kind::LetCondition { .. } => {
                    has_let = true;
                    if !allowed {
                        return Err(error(
                            node.span.start,
                            "let is only permitted in a condition conjunction",
                        ));
                    }
                }
                Kind::Binary { op, left, right } => {
                    let child_allowed = allowed && op == P::AndAnd;
                    if allowed
                        && (op == P::OrOr
                            || super::precedence(op).is_some_and(|(power, _)| power == 10))
                    {
                        excluded = Some(node.span.start);
                    }
                    self.condition_work.push((left, child_allowed));
                    self.condition_work.push((right, child_allowed));
                }
                Kind::Range { start, end, .. } => {
                    if allowed {
                        excluded = Some(node.span.start);
                    }
                    for child in [start, end].into_iter().flatten() {
                        self.condition_work.push((child, false));
                    }
                }
                _ => {}
            }
        }
        if has_let && let Some(offset) = excluded {
            return Err(error(offset, "let-chain operand requires parentheses"));
        }
        Ok(())
    }

    pub(super) fn loop_expression(&mut self) -> Result<NodeId, LexError> {
        let token = self.required()?;
        let kind = match self.cursor.spelling(token) {
            "loop" => Kind::Loop {
                body: self.block(BlockFlavor::Plain)?,
            },
            "while" => {
                let condition = self.expr(0, CONDITION)?;
                self.validate_condition(condition)?;
                Kind::While {
                    condition,
                    body: self.block(BlockFlavor::Plain)?,
                }
            }
            _ => {
                let pattern = self.pattern(true)?;
                if !self.word("in")? {
                    return Err(self.error("for requires in")?);
                }
                let iterator = self.expr(
                    0,
                    Context {
                        condition: false,
                        ..CONDITION
                    },
                )?;
                Kind::For {
                    pattern,
                    iterator,
                    body: self.block(BlockFlavor::Plain)?,
                }
            }
        };
        self.node(token.start, kind)
    }

    pub(super) fn labeled(&mut self) -> Result<NodeId, LexError> {
        let token = self.required()?;
        self.expect(P::Colon)?;
        let value = if self.cursor.at(P::OpenBrace)? {
            self.block(BlockFlavor::Plain)?
        } else if self.cursor.word("loop")?
            || self.cursor.word("while")?
            || self.cursor.word("for")?
        {
            self.loop_expression()?
        } else {
            return Err(self.error("label requires a loop or block")?);
        };
        self.node(
            token.start,
            Kind::Label {
                name: Span {
                    start: token.start,
                    end: token.end,
                },
                value,
            },
        )
    }

    fn optional_label(&mut self) -> Result<Option<Span>, LexError> {
        if self
            .cursor
            .peek(0)?
            .is_some_and(|t| matches!(t.kind, TokenKind::Lifetime | TokenKind::RawLifetime))
        {
            let token = self.required()?;
            Ok(Some(Span {
                start: token.start,
                end: token.end,
            }))
        } else {
            Ok(None)
        }
    }

    pub(super) fn jump(&mut self) -> Result<NodeId, LexError> {
        let token = self.required()?;
        let kind = match self.cursor.spelling(token) {
            "continue" => Kind::Continue(self.optional_label()?),
            "break" => {
                let label = self.optional_label()?;
                let value = if self.can_start()? {
                    Some(self.expr(0, NORMAL)?)
                } else {
                    None
                };
                Kind::Break { label, value }
            }
            _ => Kind::Return(if self.can_start()? {
                Some(self.expr(0, NORMAL)?)
            } else {
                None
            }),
        };
        self.node(token.start, kind)
    }

    pub(super) fn match_expression(&mut self) -> Result<NodeId, LexError> {
        let start = self.required()?.start;
        let value = self.expr(
            0,
            Context {
                condition: false,
                ..CONDITION
            },
        )?;
        self.expect(P::OpenBrace)?;
        let mark = self.scratch.len();
        while !self.eat(P::CloseBrace)? {
            let attributes = if self.extended {
                self.attributes(false)?
            } else {
                crate::syntax::item::AttributeRange::default()
            };
            let pattern = self.pattern(true)?;
            let guard = if self.word("if")? {
                // The fat arrow, unlike an if/while body, leaves struct literals unambiguous.
                let guard = self.expr(
                    0,
                    Context {
                        condition: true,
                        ..NORMAL
                    },
                )?;
                self.validate_condition(guard)?;
                Some(guard)
            } else {
                None
            };
            self.expect(P::FatArrow)?;
            let value = self.expr(
                0,
                Context {
                    statement: true,
                    ..NORMAL
                },
            )?;
            let arm = self.node(
                self.ast.node(pattern).span.start,
                Kind::Arm {
                    pattern,
                    guard,
                    value,
                },
            )?;
            let arm = self.attach(attributes, arm)?;
            self.scratch.push(arm);
            if self.eat(P::CloseBrace)? {
                break;
            }
            if !self.eat(P::Comma)? && !self.with_block(value) {
                return Err(self.error("expected comma after match arm")?);
            }
        }
        let arms = self.list(mark)?;
        self.node(start, Kind::Match { value, arms })
    }

    pub(super) fn closure(
        &mut self,
        moving: bool,
        asynchronous: bool,
        start: u32,
    ) -> Result<NodeId, LexError> {
        let mark = self.scratch.len();
        if !self.eat(P::OrOr)? {
            self.expect(P::Or)?;
            while !self.eat(P::Or)? {
                let attributes = if self.extended {
                    self.attributes(false)?
                } else {
                    crate::syntax::item::AttributeRange::default()
                };
                let pattern = self.pattern(false)?;
                let ty = if self.eat(P::Colon)? {
                    Some(self.ty()?)
                } else {
                    None
                };
                let parameter = self.node(
                    self.ast.node(pattern).span.start,
                    Kind::Parameter { pattern, ty },
                )?;
                let parameter = self.attach(attributes, parameter)?;
                self.scratch.push(parameter);
                if self.eat(P::Or)? {
                    break;
                }
                self.expect(P::Comma)?;
            }
        }
        let parameters = self.list(mark)?;
        let result = if self.eat(P::Arrow)? {
            Some(self.ty()?)
        } else {
            None
        };
        let body = if result.is_some() {
            self.block(BlockFlavor::Plain)?
        } else {
            self.expr(0, NORMAL)?
        };
        self.node(
            start,
            Kind::Closure {
                parameters,
                result,
                body,
                moving,
                asynchronous,
            },
        )
    }

    pub(super) fn async_expression(&mut self) -> Result<NodeId, LexError> {
        let start = self.required()?.start;
        let moving = self.word("move")?;
        if self.cursor.at(P::OpenBrace)? {
            let block = self.block(if moving {
                BlockFlavor::AsyncMove
            } else {
                BlockFlavor::Async
            })?;
            self.ast.nodes[block.0 as usize].span.start = start;
            Ok(block)
        } else {
            self.closure(moving, true, start)
        }
    }
}
