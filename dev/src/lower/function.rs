//! Function-local slots, snapshots and control flow consume the PA10 typed body.
use super::{
    Error,
    arithmetic::Arithmetic,
    casts,
    context::{Context, representation},
};
use crate::{
    body::ir::{self, Action, LocalKind, Place, Rvalue},
    lex::token::Punctuation,
    literal::value::FloatBits,
    lowir::{
        builder::Builder,
        program::{BlockId, Function, Literal, Operand, SlotId, Terminator, Value},
        vocabulary::Type,
    },
    resolve::{model::DefId, unit::NodeKey},
    sema::model::{Cast, Scalar},
    syntax::ast::NodeId,
    types::{Kind, Primitive, TypeId},
};

#[derive(Clone, Copy)]
enum Selected {
    Unit,
    Item(DefId),
    Runtime(Value),
}
impl Selected {
    fn runtime(self) -> Result<Value, &'static str> {
        if let Self::Runtime(value) = self {
            Ok(value)
        } else {
            Err("missing runtime scalar value")
        }
    }
}
struct Lower<'a, 'b> {
    context: &'a mut Context<'b>,
    body: &'b ir::Body,
    builder: Builder,
    locals: Vec<Option<SlotId>>,
    values: Vec<Option<Selected>>,
    blocks: Vec<BlockId>,
    panic: Option<BlockId>,
    source: NodeId,
    arguments: Vec<Value>,
}
pub(super) fn lower<'a>(context: &mut Context<'a>, body: &'a ir::Body) -> Result<Function, Error> {
    let mut lower = Lower {
        context,
        body,
        builder: Builder::default(),
        locals: Vec::new(),
        values: vec![None; body.values.len()],
        blocks: Vec::new(),
        panic: None,
        source: body.source.node,
        arguments: Vec::new(),
    };
    lower.run().map_err(|message| Error {
        node: NodeKey {
            file: body.source.file,
            node: lower.source,
        },
        message,
    })?;
    lower.builder.finish().map_err(|message| Error {
        node: body.source,
        message,
    })
}
impl Lower<'_, '_> {
    fn run(&mut self) -> Result<(), &'static str> {
        for local in &self.body.locals {
            let ty = representation(self.context.types, local.ty)?;
            self.locals.push(if ty == Type::Void {
                None
            } else {
                Some(self.builder.slot(ty)?)
            });
        }
        for _ in &self.body.blocks {
            self.blocks.push(self.builder.block()?);
        }
        let (order, seen) = order(self.body)?;
        for (position, id) in order.into_iter().enumerate() {
            self.builder.enter(self.blocks[id])?;
            if position == 0 {
                self.parameters()?;
            }
            let block = &self.body.blocks[id];
            for statement in &self.body.statements[block.start as usize..block.end as usize] {
                self.source = statement.source;
                match statement.action {
                    Action::Project(_) => {
                        return Err("aggregate places require checked aggregate lowering");
                    }
                    Action::StorageLive(_) | Action::StorageDead(_) => {}
                    Action::Evaluate(value) => {
                        let result = self.evaluate(value)?;
                        self.values[value.0 as usize] = Some(result);
                    }
                    Action::Store { place, value } => {
                        let slot = self.place(place)?;
                        let selected = self.value(value)?;
                        if let Selected::Runtime(value) = selected {
                            let slot = slot.ok_or("runtime store to a zero-sized local")?;
                            self.builder.store(value, Operand::Slot(slot))?;
                        }
                    }
                }
            }
            self.source = block.source;
            let terminator = match block.terminator {
                ir::Terminator::Jump(target) => Terminator::Jump(self.blocks[target.0 as usize]),
                ir::Terminator::Branch { condition, yes, no } => Terminator::Branch {
                    condition: self.value(condition)?.runtime()?.operand,
                    yes: self.blocks[yes.0 as usize],
                    no: self.blocks[no.0 as usize],
                },
                ir::Terminator::Return(value) => Terminator::Return(match self.value(value)? {
                    Selected::Runtime(value) => Some(value),
                    Selected::Unit | Selected::Item(_) => None,
                }),
                ir::Terminator::Unreachable => Terminator::Unreachable,
            };
            self.builder.terminate(terminator)?;
        }
        for (index, visited) in seen.into_iter().enumerate() {
            if !visited {
                self.builder.enter(self.blocks[index])?;
                self.builder.terminate(Terminator::Unreachable)?;
            }
        }
        if let Some(panic) = self.panic {
            self.builder.enter(panic)?;
            let symbol = self.context.abort()?;
            let signature = self.context.output.symbols[symbol.0 as usize].signature;
            self.builder
                .call(signature, Type::Void, Operand::Symbol(symbol), &[])?;
            self.builder.terminate(Terminator::Unreachable)?;
        }
        Ok(())
    }
    fn parameters(&mut self) -> Result<(), &'static str> {
        let mut parameter = 0;
        for (local, slot) in self.body.locals.iter().zip(&self.locals) {
            if matches!(local.kind, LocalKind::Parameter(_))
                && let Some(slot) = slot
            {
                let ty = representation(self.context.types, local.ty)?;
                self.builder.store(
                    Value {
                        ty,
                        operand: Operand::Parameter(parameter),
                    },
                    Operand::Slot(*slot),
                )?;
                parameter += 1;
            }
        }
        Ok(())
    }
    fn place(&self, place: Place) -> Result<Option<SlotId>, &'static str> {
        match place {
            Place::Temporary(_) | Place::Projection(_) => {
                Err("aggregate places require checked aggregate lowering")
            }
            Place::Local(id) => self
                .locals
                .get(id.0 as usize)
                .copied()
                .ok_or("missing body local"),
            Place::Static(_) => {
                Err("static values require constant evaluation and allocation lowering")
            }
        }
    }
    fn value(&self, id: ir::ValueId) -> Result<Selected, &'static str> {
        self.values
            .get(id.0 as usize)
            .copied()
            .flatten()
            .ok_or("body value does not dominate its use")
    }
    fn primitive(&self, ty: TypeId) -> Result<Primitive, &'static str> {
        if let Some(Kind::Primitive(p)) = self.context.types.kind(ty) {
            Ok(p)
        } else {
            Err("missing selected primitive type")
        }
    }
    fn reify(&mut self, id: ir::ValueId) -> Result<Selected, &'static str> {
        let selected = self.value(id)?;
        if let Selected::Item(definition) = selected {
            let symbol = self.context.demand(definition)?;
            self.builder
                .address(Operand::Symbol(symbol))
                .map(Selected::Runtime)
        } else {
            Ok(selected)
        }
    }
    fn evaluate(&mut self, id: ir::ValueId) -> Result<Selected, &'static str> {
        let value = &self.body.values[id.0 as usize];
        let ty = representation(self.context.types, value.ty)?;
        let runtime = match value.kind {
            Rvalue::Unit => return Ok(Selected::Unit),
            Rvalue::Aggregate { .. } | Rvalue::Repeat { .. } => {
                return Err("aggregate values require checked aggregate lowering");
            }
            Rvalue::Function(definition) => return Ok(Selected::Item(definition)),
            Rvalue::Scalar(scalar) => match self.body.scalars[scalar.0 as usize] {
                Scalar::Integer(bits) => self.builder.integer(ty, bits)?,
                Scalar::Boolean(value) => self.builder.integer(ty, u128::from(value))?,
                Scalar::Character(value) => {
                    self.builder.integer(ty, u128::from(u32::from(value)))?
                }
                Scalar::Float(FloatBits::F32(bits)) => {
                    self.builder.literal(ty, Literal::F32(bits))?
                }
                Scalar::Float(FloatBits::F64(bits)) => {
                    self.builder.literal(ty, Literal::F64(bits))?
                }
            },
            Rvalue::Read(place) => {
                if let Some(slot) = self.place(place)? {
                    self.builder.load(ty, Operand::Slot(slot))?
                } else {
                    return Ok(
                        if let Some(Kind::FunctionItem { item, .. }) =
                            self.context.types.kind(value.ty)
                        {
                            Selected::Item(DefId(item.0))
                        } else {
                            Selected::Unit
                        },
                    );
                }
            }
            Rvalue::Constant(_) => return Err("constant values require the constant evaluator"),
            Rvalue::Reify(operand)
            | Rvalue::Cast {
                kind: Cast::FunctionPointer,
                operand,
            } => return self.reify(operand),
            Rvalue::Constrain(_) => return Err("pending region equality requires region checking"),
            Rvalue::Unary {
                operator,
                operand,
                literal,
            } => {
                let primitive = self.primitive(self.body.values[operand.0 as usize].ty)?;
                let value = self.value(operand)?.runtime()?;
                Arithmetic {
                    builder: &mut self.builder,
                    panic: &mut self.panic,
                }
                .unary(operator, primitive, value, literal)?
            }
            Rvalue::Binary {
                operator,
                left,
                right,
            } => {
                let primitive = self.primitive(self.body.values[left.0 as usize].ty)?;
                let lhs = self.value(left)?.runtime()?;
                let rhs = self.value(right)?.runtime()?;
                if primitive.is_float() && operator == Punctuation::Percent {
                    let symbol = self.context.remainder(lhs.ty)?;
                    let signature = self.context.output.symbols[symbol.0 as usize].signature;
                    self.builder
                        .call(signature, ty, Operand::Symbol(symbol), &[lhs, rhs])?
                        .ok_or("missing remainder result")?
                } else {
                    Arithmetic {
                        builder: &mut self.builder,
                        panic: &mut self.panic,
                    }
                    .binary(operator, primitive, lhs, rhs)?
                }
            }
            Rvalue::Cast { kind, operand } => {
                let source = self.primitive(self.body.values[operand.0 as usize].ty)?;
                let target = self.primitive(value.ty)?;
                let value = self.value(operand)?.runtime()?;
                casts::cast(&mut self.builder, kind, source, target, value, ty)?
            }
            Rvalue::Call {
                signature,
                callee,
                arguments,
            } => return self.call(signature, callee, arguments, ty),
        };
        Ok(Selected::Runtime(runtime))
    }
    fn call(
        &mut self,
        signature: TypeId,
        callee: ir::ValueId,
        arguments: ir::ValueList,
        ty: Type,
    ) -> Result<Selected, &'static str> {
        let signature = self.context.signature(signature)?;
        let callee = match self.value(callee)? {
            Selected::Item(definition) => Operand::Symbol(self.context.demand(definition)?),
            Selected::Runtime(value) => value.operand,
            Selected::Unit => return Err("unit used as call target"),
        };
        self.arguments.clear();
        for id in &self.body.arguments[arguments.start as usize..arguments.end as usize] {
            if let Selected::Runtime(value) = self.value(*id)? {
                self.arguments.push(value);
            }
        }
        self.builder
            .call(signature, ty, callee, &self.arguments)
            .map(|value| value.map_or(Selected::Unit, Selected::Runtime))
    }
}

/// Reverse postorder places dominating snapshots before uses without recursion.
fn order(body: &ir::Body) -> Result<(Vec<usize>, Vec<bool>), &'static str> {
    if body.blocks.is_empty() {
        return Err("body has no entry block");
    }
    let mut seen = vec![false; body.blocks.len()];
    let mut stack = vec![(0, false)];
    let mut order = Vec::with_capacity(body.blocks.len());
    while let Some((index, exiting)) = stack.pop() {
        if exiting {
            order.push(index);
            continue;
        }
        if seen[index] {
            continue;
        }
        seen[index] = true;
        stack.push((index, true));
        match body.blocks[index].terminator {
            ir::Terminator::Jump(target) => stack.push((target.0 as usize, false)),
            ir::Terminator::Branch { yes, no, .. } => {
                stack.push((no.0 as usize, false));
                stack.push((yes.0 as usize, false));
            }
            _ => {}
        }
    }
    order.reverse();
    Ok((order, seen))
}
