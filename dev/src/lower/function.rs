//! Function-local slots, snapshots and control flow consume the PA10 typed body.
mod aggregate;
use super::{
    Error,
    arithmetic::Arithmetic,
    casts,
    context::{Context, Representation},
};
use crate::{
    body::ir::{self, Action, LocalKind, Place, Rvalue},
    lex::token::Punctuation,
    literal::value::FloatBits,
    lowir::{
        builder::Builder,
        program::{BlockId, Function, Literal, Operand, Terminator, Value},
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
    Object(Address),
}
#[derive(Clone, Copy)]
struct Address {
    operand: Operand,
    align: u32,
}
#[derive(Clone, Copy)]
enum Projected {
    Pending,
    Ready(Option<Address>),
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
    locals: Vec<Option<Address>>,
    projections: Vec<Projected>,
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
        projections: vec![Projected::Pending; body.projections.len()],
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
        let (order, seen, checked_edges) = order(self.body)?;
        let live = if checked_edges {
            let mut live = vec![false; self.body.locals.len()];
            for &index in &order {
                for statement in self.body.statements(&self.body.blocks[index]) {
                    if let Action::StorageLive(id) = statement.action {
                        live[id.0 as usize] = true;
                    }
                }
            }
            Some(live)
        } else {
            None
        };
        for (index, local) in self.body.locals.iter().enumerate() {
            let address = if live.as_ref().is_none_or(|live| live[index]) {
                self.allocate(local.ty)?
            } else {
                None
            };
            self.locals.push(address);
        }
        for _ in &self.body.blocks {
            self.blocks.push(self.builder.block()?);
        }
        for (position, id) in order.into_iter().enumerate() {
            self.builder.enter(self.blocks[id])?;
            if position == 0 {
                self.parameters()?;
            }
            let block = &self.body.blocks[id];
            for statement in &self.body.statements[block.start as usize..block.end as usize] {
                self.source = statement.source;
                match statement.action {
                    Action::Project(id) => self.project(id)?,
                    Action::StorageLive(_) | Action::StorageDead(_) | Action::StorageEnd { .. } => {
                    }
                    Action::Evaluate(value) => {
                        let result = self.evaluate(value)?;
                        self.values[value.0 as usize] = Some(result);
                    }
                    Action::Store { place, value } => {
                        let address = self.place(place)?;
                        let selected = self.value(value)?;
                        self.write(address, selected, self.body.values[value.0 as usize].ty)?;
                    }
                }
            }
            self.source = block.source;
            let terminator = match block.terminator {
                ir::Terminator::Jump(target) | ir::Terminator::FalseEdge { real: target, .. } => {
                    Terminator::Jump(self.blocks[target.0 as usize])
                }
                ir::Terminator::Branch { condition, yes, no } => Terminator::Branch {
                    condition: self.value(condition)?.runtime()?.operand,
                    yes: self.blocks[yes.0 as usize],
                    no: self.blocks[no.0 as usize],
                },
                ir::Terminator::Return(value) => Terminator::Return(match self.value(value)? {
                    Selected::Runtime(value) => Some(value),
                    Selected::Object(address) => {
                        let align = self.context.layout(self.body.result)?.align;
                        self.write(
                            Some(Address {
                                operand: Operand::Parameter(0),
                                align,
                            }),
                            Selected::Object(address),
                            self.body.result,
                        )?;
                        None
                    }
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
        let mut parameter = u32::from(matches!(
            self.context.representation(self.body.result)?,
            Representation::Object(_)
        ));
        for (index, local) in self.body.locals.iter().enumerate() {
            if matches!(local.kind, LocalKind::Parameter(_))
                && let Some(address) = self.locals[index]
            {
                let value = match self.context.representation(local.ty)? {
                    Representation::Scalar(ty) => Selected::Runtime(Value {
                        ty,
                        operand: Operand::Parameter(parameter),
                    }),
                    Representation::Object(span) => Selected::Object(Address {
                        operand: Operand::Parameter(parameter),
                        align: span.align,
                    }),
                };
                self.write(Some(address), value, local.ty)?;
                parameter += 1;
            }
        }
        Ok(())
    }
    fn place(&self, place: Place) -> Result<Option<Address>, &'static str> {
        match place {
            Place::Temporary(id) => match self.value(id)? {
                Selected::Object(address) => Ok(Some(address)),
                Selected::Unit | Selected::Item(_) => Ok(None),
                Selected::Runtime(_) => Err("scalar temporary has no aggregate place"),
            },
            Place::Projection(id) => match self.projections.get(id.0 as usize) {
                Some(Projected::Ready(address)) => Ok(*address),
                _ => Err("projected address does not dominate its use"),
            },
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
    #[expect(
        clippy::too_many_lines,
        reason = "One typed-rvalue dispatch consumes the selected body operations"
    )]
    fn evaluate(&mut self, id: ir::ValueId) -> Result<Selected, &'static str> {
        let value = &self.body.values[id.0 as usize];
        let representation = self.context.representation(value.ty)?;
        let ty = representation.result();
        let runtime = match value.kind {
            Rvalue::Unit | Rvalue::Validate(_) => return Ok(Selected::Unit),
            Rvalue::Aggregate { variant, fields } => {
                return self.aggregate(value.ty, variant, fields);
            }
            Rvalue::Repeat { value: operand } => return self.repeat(value.ty, operand),
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
            Rvalue::IsVariant { place, variant } => return self.is_variant(place, variant),
            Rvalue::Read(place) => {
                if let Place::Temporary(id) = place
                    && let Selected::Runtime(snapshot) = self.value(id)?
                {
                    // Scalar temporaries are immutable evaluated snapshots; their
                    // owned-value proof still precedes this SSA reuse.
                    return Ok(Selected::Runtime(snapshot));
                }
                if let Some(address) = self.place(place)? {
                    if matches!(representation, Representation::Object(_)) {
                        let destination = self
                            .allocate(value.ty)?
                            .ok_or("missing aggregate snapshot")?;
                        self.write(Some(destination), Selected::Object(address), value.ty)?;
                        return Ok(Selected::Object(destination));
                    }
                    self.builder.load(ty, address.operand)?
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
            Rvalue::Cast {
                kind: Cast::EnumToInteger,
                operand,
            } => {
                return self.enum_cast(operand, ty);
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
            } => return self.call(signature, callee, arguments, value.ty),
        };
        Ok(Selected::Runtime(runtime))
    }
    fn call(
        &mut self,
        signature: TypeId,
        callee: ir::ValueId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let signature = self.context.signature(signature)?;
        let representation = self.context.representation(result)?;
        let callee = match self.value(callee)? {
            Selected::Item(definition) => Operand::Symbol(self.context.demand(definition)?),
            Selected::Runtime(value) => value.operand,
            Selected::Unit | Selected::Object(_) => return Err("non-function used as call target"),
        };
        self.arguments.clear();
        let destination = if matches!(representation, Representation::Object(_)) {
            let address = self.allocate(result)?.ok_or("missing result allocation")?;
            let pointer = self.pointer(address)?;
            self.arguments.push(pointer);
            Some(address)
        } else {
            None
        };
        for id in &self.body.arguments[arguments.start as usize..arguments.end as usize] {
            match self.value(*id)? {
                Selected::Runtime(value) => self.arguments.push(value),
                Selected::Object(address) => {
                    let pointer = self.pointer(address)?;
                    self.arguments.push(pointer);
                }
                Selected::Unit | Selected::Item(_) => {}
            }
        }
        let value =
            self.builder
                .call(signature, representation.result(), callee, &self.arguments)?;
        Ok(destination.map_or_else(
            || value.map_or(Selected::Unit, Selected::Runtime),
            Selected::Object,
        ))
    }
}

/// Reverse postorder places dominating snapshots before uses without recursion.
fn order(body: &ir::Body) -> Result<(Vec<usize>, Vec<bool>, bool), &'static str> {
    if body.blocks.is_empty() {
        return Err("body has no entry block");
    }
    let mut seen = vec![false; body.blocks.len()];
    let mut stack = vec![(0, false)];
    let mut order = Vec::with_capacity(body.blocks.len());
    let mut checked_edges = false;
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
            ir::Terminator::FalseEdge { real, .. } => {
                checked_edges = true;
                stack.push((real.0 as usize, false));
            }
            ir::Terminator::Branch { yes, no, .. } => {
                stack.push((no.0 as usize, false));
                stack.push((yes.0 as usize, false));
            }
            _ => {}
        }
    }
    order.reverse();
    Ok((order, seen, checked_edges))
}

#[cfg(test)]
mod tests {
    #[test]
    fn aggregate_addresses_keep_ordinary_snapshots_compact() {
        assert!(std::mem::size_of::<Option<super::Selected>>() <= 24);
    }
}
