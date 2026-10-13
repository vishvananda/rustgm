//! Function-local slots, snapshots and control flow consume the PA10 typed body.
mod aggregate;
mod references;
mod selections;
mod slices;
mod text;
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
    sema::{
        model::{Cast, Scalar},
        values::View,
    },
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
    addresses: references::Storage,
    slice_lengths: std::collections::HashMap<ir::ProjectionId, Value>,
    projections: Vec<Projected>,
    selections: std::collections::HashMap<ir::ProjectionId, selections::Recipe>,
    selection_paths: std::collections::HashMap<ir::ProjectionId, std::num::NonZeroU32>,
    selection_work: Vec<(ir::ProjectionId, selections::Step)>,
    selection_steps: Vec<selections::Dynamic>,
    address_work: Vec<usize>,
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
        addresses: references::Storage::default(),
        slice_lengths: std::collections::HashMap::new(),
        projections: vec![Projected::Pending; body.projections.len()],
        selections: std::collections::HashMap::new(),
        selection_paths: std::collections::HashMap::new(),
        selection_work: Vec::new(),
        selection_steps: Vec::new(),
        address_work: Vec::new(),
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
        self.initialize_storage(&order, checked_edges)?;
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
                    Action::BindView { view, place } => self.bind_view(view, place)?,
                    Action::Project(id) => self.project(id)?,
                    Action::Select { projection, anchor } => self.select(projection, anchor)?,
                    Action::GuardStart(_)
                    | Action::GuardUse(_)
                    | Action::StorageLive(_)
                    | Action::StorageDead(_)
                    | Action::StorageEnd { .. } => {}
                    Action::Evaluate(value) => {
                        let result = self.evaluate(value)?;
                        self.materialize(value, result)?;
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
    fn initialize_storage(
        &mut self,
        order: &[usize],
        checked_edges: bool,
    ) -> Result<(), &'static str> {
        self.addresses = references::Storage::collect(self.body)?;
        let live = if checked_edges {
            let mut live = vec![false; self.body.locals.len()];
            for &index in order {
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
                let id = ir::LocalId(u32::try_from(index).map_err(|_| "too many body locals")?);
                self.allocate_local(local.ty, local.kind, self.addresses.needs(Place::Local(id)))?
            } else {
                None
            };
            self.locals.push(address);
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
                    Representation::Scalar(Type::Void) => continue,
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
    fn place(&mut self, place: Place) -> Result<Option<Address>, &'static str> {
        match place {
            Place::Temporary(id) => {
                let selected = self.value(id)?;
                if let Some(value) = self.body.promotions.as_ref().and_then(|p| p.get(id)) {
                    let (ir::Owner::Function { definition, .. }
                    | ir::Owner::Initializer { definition, .. }) = self.body.owner;
                    let ty = self.body.values[id.0 as usize].ty;
                    let global = self.context.constant_data(
                        super::data::Key::Temporary(definition, id),
                        value,
                        ty,
                        true,
                    )?;
                    return Ok(Some(Address {
                        operand: Operand::Global(global),
                        align: self.context.layout(ty)?.align,
                    }));
                }
                if let Some(&address) = self.addresses.temporaries.get(&id) {
                    return Ok(Some(address));
                }
                match selected {
                    Selected::Object(address) => Ok(Some(address)),
                    Selected::Unit | Selected::Item(_) => Ok(None),
                    Selected::Runtime(value) => {
                        let address = self.allocate(self.body.values[id.0 as usize].ty)?;
                        self.write(
                            address,
                            Selected::Runtime(value),
                            self.body.values[id.0 as usize].ty,
                        )?;
                        Ok(address)
                    }
                }
            }
            Place::Projection(id) => match self.projections.get(id.0 as usize) {
                Some(Projected::Ready(address)) => Ok(*address),
                _ => Err("projected address does not dominate its use"),
            },
            Place::Local(id) => {
                let address = self
                    .locals
                    .get(id.0 as usize)
                    .copied()
                    .ok_or("missing body local")?;
                if matches!(self.body.locals[id.0 as usize].kind, LocalKind::Guard(_))
                    && let Some(address) = address
                {
                    let pointer = self.builder.load(Type::Ptr, address.operand)?;
                    let align = self
                        .context
                        .layout(self.body.locals[id.0 as usize].ty)?
                        .align;
                    Ok(Some(Address {
                        operand: pointer.operand,
                        align,
                    }))
                } else {
                    Ok(address)
                }
            }
            Place::Static(id) => {
                let global = self.context.static_data(id)?;
                Ok(Some(Address {
                    operand: Operand::Global(global),
                    align: self.context.output.globals[global.0 as usize].span.align,
                }))
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
    fn scalar(&mut self, scalar: Scalar, ty: Type) -> Result<Value, &'static str> {
        match scalar {
            Scalar::Integer(bits) => self.builder.integer(ty, bits),
            Scalar::Boolean(value) => self.builder.integer(ty, u128::from(value)),
            Scalar::Character(value) => self.builder.integer(ty, u128::from(u32::from(value))),
            Scalar::Float(FloatBits::F32(bits)) => self.builder.literal(ty, Literal::F32(bits)),
            Scalar::Float(FloatBits::F64(bits)) => self.builder.literal(ty, Literal::F64(bits)),
        }
    }
    fn constant(
        &mut self,
        definition: DefId,
        ty: TypeId,
        representation: Representation,
    ) -> Result<Selected, &'static str> {
        let value = self
            .context
            .constants
            .definition(definition)
            .ok_or("named constant was not evaluated")?;
        match self.context.constants.view(value)? {
            View::Scalar(scalar) => self
                .scalar(scalar, representation.result())
                .map(Selected::Runtime),
            View::Function(id) => {
                if representation.result() == Type::Void {
                    return Ok(Selected::Item(id));
                }
                let symbol = self.context.demand(id)?;
                self.builder
                    .address(Operand::Symbol(symbol))
                    .map(Selected::Runtime)
            }
            _ => {
                if matches!(representation, Representation::Scalar(Type::Void)) {
                    return Ok(Selected::Unit);
                }
                let global = self.context.constant_data(
                    super::data::Key::Definition(definition),
                    value,
                    ty,
                    true,
                )?;
                let source = Address {
                    operand: Operand::Global(global),
                    align: self.context.layout(ty)?.align,
                };
                if let Representation::Scalar(ty) = representation {
                    return self.builder.load(ty, source.operand).map(Selected::Runtime);
                }
                let destination = self
                    .allocate(ty)?
                    .ok_or("constant product lacks snapshot storage")?;
                self.write(Some(destination), Selected::Object(source), ty)?;
                Ok(Selected::Object(destination))
            }
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
            Rvalue::Borrow { place, .. } => {
                let address = self
                    .place(place)?
                    .ok_or("borrowed place lacks stable storage")?;
                let pointer = self.pointer(address)?;
                if matches!(representation, Representation::Object(_)) {
                    let length = self.slice_length(place)?;
                    return self.wide_pointer(pointer, length);
                }
                pointer
            }
            Rvalue::Unit | Rvalue::Validate(_) => return Ok(Selected::Unit),
            Rvalue::Aggregate { variant, fields } => {
                return self.aggregate(value.ty, variant, fields);
            }
            Rvalue::Repeat { value: operand } => return self.repeat(value.ty, operand),
            Rvalue::Function(definition) => return Ok(Selected::Item(definition)),
            Rvalue::Scalar(scalar) => self.scalar(self.body.scalars[scalar.0 as usize], ty)?,
            Rvalue::Length(place) => self.slice_length(place)?,
            Rvalue::TextEquals { place, text } => self.text_equals(place, text)?,
            Rvalue::Text(text) => return self.text_literal(text, representation),
            Rvalue::IsVariant { place, variant } => return self.is_variant(place, variant),
            Rvalue::Read(place) => {
                if matches!(representation, Representation::Scalar(Type::Void)) {
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
                if let Place::Temporary(id) = place
                    && !self.addresses.temporaries.contains_key(&id)
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
            Rvalue::Constant(definition) => {
                return self.constant(definition, value.ty, representation);
            }
            Rvalue::Reify(operand)
            | Rvalue::Cast {
                kind: Cast::FunctionPointer,
                operand,
            } => return self.reify(operand),
            Rvalue::Constrain(operand) => return self.constrain(operand, value.ty, representation),
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
