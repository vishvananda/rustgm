//! Operator precedence and ordinary expression forms share one iterative writer.
use super::{Action, Kind, LexError, Node, View, Writer, error};
use crate::{
    lex::token::Punctuation as P,
    syntax::ast::{BlockFlavor, Unary},
};
pub(super) fn power(kind: Kind) -> u8 {
    match kind {
        Kind::Binary { op, .. } => super::super::precedence(op).map_or(0, |(left, _)| left),
        Kind::Range { .. } => 20,
        Kind::Cast { .. } => 120,
        Kind::Unary {
            op: Unary::Try | Unary::Await,
            ..
        } => 150,
        Kind::Unary { .. } => 130,
        Kind::Return(_)
        | Kind::Break { .. }
        | Kind::Closure { .. }
        | Kind::ConstClosure { .. }
        | Kind::LetCondition { .. } => 0,
        _ => 160,
    }
}
impl<'a> Writer<'a> {
    #[expect(
        clippy::too_many_lines,
        reason = "exhaustive ordinary expression spelling dispatch"
    )]
    pub(super) fn operand(&mut self, view: View<'a>, node: Node) -> Result<(), LexError> {
        use Action::Word as W;
        match node.kind {
            Kind::Name(name, _) => self.enqueue(&[view.span(name)]),
            Kind::Literal(_) | Kind::DeferredLiteral(_) => self.enqueue(&[view.span(node.span)]),
            Kind::CrateRoot(_) => self.text("$crate"),
            Kind::Group(value) => self.enqueue(&[W("("), view.expr().n(value), W(")")]),
            Kind::TryBlock { body, .. } => self.enqueue(&[W("try "), view.expr().n(body)]),
            Kind::OffsetOf {
                container, fields, ..
            } => self.enqueue(&[
                W("builtin # offset_of("),
                view.ty().n(container),
                W(", "),
                view.expr().n(fields),
                W(")"),
            ]),
            Kind::ConstClosure { closure, .. } => {
                self.enqueue(&[W("const "), view.expr().n(closure)])
            }
            Kind::Tuple(list) => self.enqueue(&[
                W("("),
                view.expr().sequence(list, ", "),
                W(if view.list(list)?.len() == 1 {
                    ",)"
                } else {
                    ")"
                }),
            ]),
            Kind::Array(list) => self.enqueue(&[W("["), view.expr().sequence(list, ", "), W("]")]),
            Kind::Repeat { value, count } => {
                self.enqueue(&[W("["), view.n(value), W("; "), view.expr().n(count), W("]")])
            }
            Kind::Path { absolute, segments } => self.enqueue(&[
                W(if absolute { "::" } else { "" }),
                view.sequence(segments, "::"),
            ]),
            Kind::Unary { op, operand } => self.unary(view, op, operand),
            Kind::Binary { op, left, right } => {
                let (left_power, right_power) = super::super::precedence(op).ok_or_else(|| {
                    error(
                        node.span.start,
                        "expression writer received an invalid binary operator",
                    )
                })?;
                let comparable = matches!(
                    op,
                    P::EqualEqual
                        | P::BangEqual
                        | P::Less
                        | P::Greater
                        | P::LessEqual
                        | P::GreaterEqual
                );
                let cast = matches!(op, P::Less | P::ShiftLeft)
                    && matches!(view.node(left)?.kind, Kind::Cast { .. });
                let fix_left = comparable || left_power == right_power;
                let minimum_left = if cast {
                    161
                } else {
                    left_power.saturating_add(u8::from(fix_left))
                };
                let left_view = if matches!(
                    view.mode,
                    super::Mode::Statement | super::Mode::LeftmostOperand
                ) {
                    view.leftmost()
                } else {
                    view.expr()
                };
                self.enqueue(&[
                    left_view.child(left, minimum_left),
                    Action::Space { offset: 0 },
                    W(super::super::render::operator(op)),
                    Action::Space { offset: 0 },
                    view.expr().child(right, right_power),
                ])
            }
            Kind::Cast { value, ty } => self.enqueue(&[
                if matches!(
                    view.mode,
                    super::Mode::Statement | super::Mode::LeftmostOperand
                ) {
                    view.leftmost().child(value, 120)
                } else {
                    view.expr().child(value, 120)
                },
                Action::Separator(" as "),
                view.ty().n(ty),
            ]),
            Kind::Call { callee, arguments } => {
                let parentheses = match view.node(callee)?.kind {
                    Kind::Field { name, .. } => {
                        let spelling = view
                            .source
                            .get(name.start as usize..name.end as usize)
                            .ok_or_else(|| {
                                error(name.start, "callee field writer source changed")
                            })?;
                        self.pay(spelling.len().saturating_add(1), 0)?;
                        !spelling.bytes().all(|byte| byte.is_ascii_digit())
                    }
                    Kind::Block { .. }
                    | Kind::If { .. }
                    | Kind::Match { .. }
                    | Kind::Loop { .. }
                    | Kind::While { .. }
                    | Kind::For { .. } => true,
                    _ => false,
                };
                self.enqueue(&[
                    W(if parentheses { "(" } else { "" }),
                    view.expr().child(callee, 150),
                    W(if parentheses { ")(" } else { "(" }),
                    view.expr().sequence(arguments, ", "),
                    W(")"),
                ])
            }
            Kind::MethodCall {
                receiver,
                name,
                arguments,
            } => self.enqueue(&[
                view.expr().child(receiver, 150),
                W("."),
                view.span(name),
                W("("),
                view.expr().sequence(arguments, ", "),
                W(")"),
            ]),
            Kind::GenericMethodCall {
                receiver,
                method,
                arguments,
            } => self.enqueue(&[
                view.expr().child(receiver, 150),
                W("."),
                view.n(method),
                W("("),
                view.expr().sequence(arguments, ", "),
                W(")"),
            ]),
            Kind::Index { value, index } => self.enqueue(&[
                view.expr().child(value, 150),
                W("["),
                view.expr().n(index),
                W("]"),
            ]),
            Kind::Field { value, name } => {
                self.enqueue(&[view.expr().child(value, 150), W("."), view.span(name)])
            }
            Kind::Block {
                flavor,
                statements,
                tail,
            } => {
                let prefix = match flavor {
                    BlockFlavor::Plain => "",
                    BlockFlavor::Unsafe => "unsafe ",
                    BlockFlavor::Const => "const ",
                    BlockFlavor::Async => "async ",
                    BlockFlavor::AsyncMove => "async move ",
                };
                let nonempty = statements.start != statements.end || tail.is_some();
                if nonempty {
                    self.enqueue(&[Action::EndBox])?;
                    self.enqueue(&[Action::Space { offset: -4 }, W("}")])?;
                    if let Some(value) = tail {
                        self.enqueue(&[view.statement().n(value)])?;
                    }
                    if tail.is_some() && statements.start != statements.end {
                        self.enqueue(&[Action::Space { offset: 0 }])?;
                    }
                    self.enqueue(&[
                        Action::BeginBox {
                            indent: 0,
                            consistent: true,
                        },
                        W(prefix),
                        W("{"),
                        Action::Space { offset: 0 },
                        view.expr().sequence(statements, " "),
                    ])
                } else {
                    self.enqueue(&[W(prefix), W("{}")])
                }
            }
            Kind::EmptyStatement => self.text(";"),
            Kind::Statement(value) => self.enqueue(&[view.statement().n(value), W(";")]),
            // Additional admitted grammar forms are supplied by their owning modules
            // before this staged writer can be selected by source Assert.
            _ => self.flow(view, node),
        }
    }
    fn unary(
        &mut self,
        view: View<'a>,
        op: Unary,
        operand: crate::syntax::ast::NodeId,
    ) -> Result<(), LexError> {
        use Action::Word as W;
        let prefix = match op {
            Unary::Negate => "-",
            Unary::Not => "!",
            Unary::Dereference => "*",
            Unary::Borrow => "&",
            Unary::BorrowMut => "&mut ",
            Unary::RawConst => "&raw const ",
            Unary::RawMut => "&raw mut ",
            Unary::Try | Unary::Await => "",
        };
        if matches!(op, Unary::Try | Unary::Await) {
            self.enqueue(&[
                view.child(operand, 150),
                W(if op == Unary::Try { "?" } else { ".await" }),
            ])
        } else {
            self.enqueue(&[W(prefix), view.child(operand, 130)])
        }
    }
}
