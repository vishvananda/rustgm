//! Control-flow and binding spelling preserve ordinary expression/statement structure.
use super::{Action, Kind, LexError, Node, NodeId, View, Writer, error};
impl<'a> Writer<'a> {
    #[expect(
        clippy::too_many_lines,
        reason = "exhaustive control-flow and statement spelling dispatch"
    )]
    pub(super) fn flow(&mut self, view: View<'a>, node: Node) -> Result<(), LexError> {
        use Action::Word as W;
        let view = view.expr();
        match node.kind {
            Kind::If {
                condition,
                then,
                otherwise,
            } => {
                if let Some(value) = otherwise {
                    self.enqueue(&[W(" else "), view.n(value)])?;
                }
                let minimum = if self.condition_parens(view, condition)? {
                    161
                } else {
                    0
                };
                self.enqueue(&[
                    W("if "),
                    view.child(condition, minimum),
                    W(" "),
                    view.n(then),
                ])
            }
            Kind::Loop { body } => self.enqueue(&[W("loop "), view.n(body)]),
            Kind::While { condition, body } => {
                let minimum = if self.condition_parens(view, condition)? {
                    161
                } else {
                    0
                };
                self.enqueue(&[
                    W("while "),
                    view.child(condition, minimum),
                    W(" "),
                    view.n(body),
                ])
            }
            Kind::For {
                pattern,
                iterator,
                body,
            } => {
                let minimum = if self.condition_parens(view, iterator)? {
                    161
                } else {
                    0
                };
                self.enqueue(&[
                    W("for "),
                    view.pattern().n(pattern),
                    W(" in "),
                    view.child(iterator, minimum),
                    W(" "),
                    view.n(body),
                ])
            }
            Kind::LetCondition { pattern, value } => self.enqueue(&[
                W("let "),
                view.pattern().n(pattern),
                W(" = "),
                view.n(value),
            ]),
            Kind::Let {
                pattern,
                ty,
                value,
                otherwise,
            } => {
                self.enqueue(&[W(";")])?;
                if let Some(otherwise) = otherwise {
                    self.enqueue(&[W(" else "), view.n(otherwise)])?;
                }
                if let Some(value) = value {
                    self.enqueue(&[W(" = "), view.n(value)])?;
                }
                if let Some(ty) = ty {
                    self.enqueue(&[W(": "), view.ty().n(ty)])?;
                }
                self.enqueue(&[W("let "), view.pattern().n(pattern)])
            }
            Kind::Return(value) => {
                if let Some(value) = value {
                    self.enqueue(&[W("return "), view.n(value)])
                } else {
                    self.text("return")
                }
            }
            Kind::Break { label, value } => {
                if let Some(value) = value {
                    self.enqueue(&[W(" "), view.n(value)])?;
                }
                if let Some(label) = label {
                    self.enqueue(&[W(" "), view.span(label)])?;
                }
                self.enqueue(&[W("break")])
            }
            Kind::Continue(label) => {
                if let Some(label) = label {
                    self.enqueue(&[W("continue "), view.span(label)])
                } else {
                    self.text("continue")
                }
            }
            Kind::Label { name, value } => self.enqueue(&[view.span(name), W(": "), view.n(value)]),
            Kind::Match { value, arms } => {
                if arms.start == arms.end {
                    return self.enqueue(&[W("match "), view.n(value), W(" {}")]);
                }
                let minimum = if self.condition_parens(view, value)? {
                    161
                } else {
                    0
                };
                self.enqueue(&[
                    W("match "),
                    view.child(value, minimum),
                    W(" { "),
                    view.sequence(arms, " "),
                    W(" }"),
                ])
            }
            Kind::Arm {
                pattern,
                guard,
                value,
            } => {
                let comma = !matches!(
                    view.node(value)?.kind,
                    Kind::Block {
                        flavor: crate::syntax::ast::BlockFlavor::Plain,
                        ..
                    }
                );
                self.enqueue(&[W(" => "), view.n(value), W(if comma { "," } else { "" })])?;
                if let Some(guard) = guard {
                    self.enqueue(&[W(" if "), view.n(guard)])?;
                }
                self.enqueue(&[view.pattern().n(pattern)])
            }
            Kind::Closure {
                parameters,
                result,
                body,
                moving,
                asynchronous,
            } => {
                self.enqueue(&[W(" "), view.n(body)])?;
                if let Some(result) = result {
                    self.enqueue(&[W(" -> "), view.ty().n(result)])?;
                }
                self.enqueue(&[
                    W(if asynchronous { "async " } else { "" }),
                    W(if moving { "move " } else { "" }),
                    W("|"),
                    view.sequence(parameters, ", "),
                    W("|"),
                ])
            }
            Kind::Parameter { pattern, ty } => {
                if let Some(ty) = ty {
                    self.enqueue(&[W(": "), view.ty().n(ty)])?;
                }
                self.enqueue(&[view.pattern().n(pattern)])
            }
            _ => self.pattern_type(view, node),
        }
    }
    fn condition_parens(&mut self, view: View<'_>, root: NodeId) -> Result<bool, LexError> {
        let mut pending = Vec::new();
        self.pay(1, 4 * size_of::<NodeId>())?;
        pending
            .try_reserve(1)
            .map_err(|_| error(0, "condition spelling allocation failed"))?;
        pending.push(root);
        while let Some(id) = pending.pop() {
            self.pay(1, 4 * 2 * size_of::<NodeId>())?;
            if pending.len().saturating_add(2) > super::MAX_FRAMES {
                return Err(error(0, "condition spelling frame budget exceeded"));
            }
            pending
                .try_reserve(2)
                .map_err(|_| error(0, "condition spelling allocation failed"))?;
            match view.node(id)?.kind {
                Kind::Struct { .. } => return Ok(true),
                Kind::Break { .. } | Kind::Return(_) | Kind::Closure { .. } if id == root => {
                    return Ok(true);
                }
                Kind::Binary { left, right, .. } => {
                    pending.push(right);
                    pending.push(left);
                }
                Kind::Unary { operand, .. }
                | Kind::Cast { value: operand, .. }
                | Kind::Field { value: operand, .. }
                | Kind::Index { value: operand, .. }
                | Kind::MethodCall {
                    receiver: operand, ..
                }
                | Kind::GenericMethodCall {
                    receiver: operand, ..
                }
                | Kind::Call {
                    callee: operand, ..
                } => pending.push(operand),
                Kind::Range { start, end, .. } => {
                    pending.extend(end);
                    pending.extend(start);
                }
                _ => {}
            }
        }
        Ok(false)
    }
}
