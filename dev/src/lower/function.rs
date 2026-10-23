//! Function-local slots, snapshots and control flow consume the PA10 typed body.
mod aggregate;
mod c_boundary;
mod cleanup;
mod intrinsics;
mod invocations;
mod pointers;
mod references;
mod referents;
mod selections;
mod slices;
mod text;
use super::{
    Error,
    address::Address,
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
    resolve::model::DefId,
    sema::{
        model::{Cast, Scalar},
        values::View,
    },
    types::{Kind, Primitive, TypeId, instance::Instance},
};

#[derive(Clone, Copy)]
enum Selected {
    Unit,
    Item(Instance),
    Runtime(Value),
    Object(Address),
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
/// Dense per-projection facts of one body.
pub(super) struct ProjectionMap<V> {
    slots: Vec<Option<V>>,
}
impl<V> Default for ProjectionMap<V> {
    fn default() -> Self {
        Self { slots: Vec::new() }
    }
}
impl<V: Copy> ProjectionMap<V> {
    fn reset(&mut self, projections: usize) {
        self.slots.clear();
        self.slots.resize(projections, None);
    }
    fn get(&self, id: ir::ProjectionId) -> Option<&V> {
        self.slots.get(id.0 as usize)?.as_ref()
    }
    fn insert(&mut self, id: ir::ProjectionId, value: V) {
        let index = id.0 as usize;
        if index >= self.slots.len() {
            self.slots.resize(index + 1, None);
        }
        self.slots[index] = Some(value);
    }
}
/// Per-body lowering vectors whose capacity is reused by the next body.
#[derive(Default)]
pub(super) struct Scratch {
    object_arguments: Vec<crate::lowir::boundary::Value>,
    locals: Vec<Option<Address>>,
    place_metadata: ProjectionMap<Value>,
    projections: Vec<Projected>,
    selections: ProjectionMap<selections::Recipe>,
    selection_paths: ProjectionMap<std::num::NonZeroU32>,
    selection_work: Vec<(ir::ProjectionId, selections::Step)>,
    selection_steps: Vec<selections::Dynamic>,
    address_work: Vec<usize>,
    values: Vec<Option<Selected>>,
    blocks: Vec<BlockId>,
    arguments: Vec<Value>,
    order: BlockOrder,
}
struct Lower<'a, 'b, 'c> {
    body_index: usize,
    native: crate::lowir::program::SignatureId,
    object_arguments: Vec<crate::lowir::boundary::Value>,
    context: &'a mut Context<'c>,
    body: &'b ir::Body,
    cleanup: Option<Box<cleanup::Pending<'b>>>,
    builder: Builder,
    locals: Vec<Option<Address>>,
    addresses: references::Storage,
    place_metadata: ProjectionMap<Value>,
    projections: Vec<Projected>,
    selections: ProjectionMap<selections::Recipe>,
    selection_paths: ProjectionMap<std::num::NonZeroU32>,
    selection_work: Vec<(ir::ProjectionId, selections::Step)>,
    selection_steps: Vec<selections::Dynamic>,
    address_work: Vec<usize>,
    values: Vec<Option<Selected>>,
    blocks: Vec<BlockId>,
    panic: Option<BlockId>,
    source: ir::BodySite,
    arguments: Vec<Value>,
    order: BlockOrder,
}
pub(super) fn lower<'b>(
    context: &mut Context<'_>,
    body_index: usize,
    body: &'b ir::Body,
    plan: Option<&'b crate::borrow::destruction::Plan>,
    native: crate::lowir::program::SignatureId,
) -> Result<Function, Error> {
    let cleanup = plan
        .map(|p| cleanup::Pending::new(body, p).map(Box::new))
        .transpose()
        .map_err(|message| Error {
            node: Some(body.source.into()),
            message,
        })?;
    let builder = context.output.body_builder();
    let mut scratch = std::mem::take(&mut context.lowering);
    scratch.object_arguments.clear();
    scratch.locals.clear();
    scratch.place_metadata.reset(body.projections.len());
    scratch.projections.clear();
    scratch
        .projections
        .resize(body.projections.len(), Projected::Pending);
    scratch.selections.reset(body.projections.len());
    scratch.selection_paths.reset(body.projections.len());
    scratch.selection_work.clear();
    scratch.selection_steps.clear();
    scratch.address_work.clear();
    scratch.values.clear();
    scratch.values.resize(body.values.len(), None);
    scratch.blocks.clear();
    scratch.arguments.clear();
    let mut lower = Lower {
        body_index,
        native,
        object_arguments: scratch.object_arguments,
        context,
        body,
        cleanup,
        builder,
        locals: scratch.locals,
        addresses: references::Storage::default(),
        place_metadata: scratch.place_metadata,
        projections: scratch.projections,
        selections: scratch.selections,
        selection_paths: scratch.selection_paths,
        selection_work: scratch.selection_work,
        selection_steps: scratch.selection_steps,
        address_work: scratch.address_work,
        values: scratch.values,
        blocks: scratch.blocks,
        panic: None,
        source: body.root_site(),
        arguments: scratch.arguments,
        order: scratch.order,
    };
    lower.run().map_err(|message| Error {
        node: Some(body.diagnostic_site(lower.source)),
        message,
    })?;
    lower.context.lowering = Scratch {
        object_arguments: lower.object_arguments,
        locals: lower.locals,
        place_metadata: lower.place_metadata,
        projections: lower.projections,
        selections: lower.selections,
        selection_paths: lower.selection_paths,
        selection_work: lower.selection_work,
        selection_steps: lower.selection_steps,
        address_work: lower.address_work,
        values: lower.values,
        blocks: lower.blocks,
        arguments: lower.arguments,
        order: lower.order,
    };
    lower.builder.finish().map_err(|message| Error {
        node: Some(body.source.into()),
        message,
    })
}
impl Lower<'_, '_, '_> {
    fn run(&mut self) -> Result<(), &'static str> {
        let mut order = std::mem::take(&mut self.order);
        let checked_edges = order.compute(self.body)?;
        self.initialize_storage(&order.blocks, checked_edges)?;
        for _ in &self.body.blocks {
            self.blocks.push(self.builder.block()?);
        }
        if let Some(cleanup) = &mut self.cleanup {
            cleanup.allocate(&mut self.builder, self.body, &order.seen)?;
        }
        for (position, &id) in order.blocks.iter().enumerate() {
            self.builder.enter(self.blocks[id])?;
            if position == 0 {
                self.parameters()?;
                if let Some(cleanup) = &mut self.cleanup {
                    cleanup.state.initialize(&mut self.builder)?;
                }
            }
            let block = &self.body.blocks[id];
            if let Some(cleanup) = &self.cleanup {
                let range = cleanup.timeline.blocks[id].clone();
                self.cleanup_events(range)?;
            } else {
                for statement in &self.body.statements[block.start as usize..block.end as usize] {
                    self.source = statement.source;
                    self.statement(statement.action)?;
                }
            }
            self.source = block.source;
            let terminator = match block.terminator {
                ir::Terminator::Jump(target) | ir::Terminator::FalseEdge { real: target, .. } => {
                    Terminator::Jump(self.target(id, target)?)
                }
                ir::Terminator::Branch { condition, yes, no } => Terminator::Branch {
                    condition: self.value(condition)?.runtime()?.operand,
                    yes: self.target(id, yes)?,
                    no: self.target(id, no)?,
                },
                ir::Terminator::Return(value) => self.returned(value)?,
                ir::Terminator::Unreachable => Terminator::Unreachable,
                ir::Terminator::Panic => {
                    let panic = if let Some(block) = self.panic {
                        block
                    } else {
                        let block = self.builder.block()?;
                        self.panic = Some(block);
                        block
                    };
                    Terminator::Jump(panic)
                }
            };
            self.builder.terminate(terminator)?;
        }
        for (index, &visited) in order.seen.iter().enumerate() {
            if !visited {
                self.builder.enter(self.blocks[index])?;
                self.builder.terminate(Terminator::Unreachable)?;
            }
        }
        self.order = order;
        self.cleanup_edges()?;
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
    fn statement(&mut self, action: Action) -> Result<(), &'static str> {
        match action {
            Action::BindView { view, place } => self.bind_view(view, place)?,
            Action::Project(id) => self.project(id)?,
            Action::Select { projection, anchor } => self.select(projection, anchor)?,
            Action::CheckInitialized(_)
            | Action::Mention(_)
            | Action::GuardStart(_)
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
        Ok(())
    }
    fn initialize_storage(
        &mut self,
        order: &[usize],
        checked_edges: bool,
    ) -> Result<(), &'static str> {
        self.addresses = references::Storage::collect(self.body)?;
        if let Some(cleanup) = &self.cleanup {
            for entry in &cleanup.plan.cleanups {
                if entry.node.is_some() {
                    self.addresses.mark(self.body, entry.candidate.place)?;
                }
            }
        }
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
        if matches!(
            self.context.output.signatures.get(self.native.0 as usize),
            Some(crate::lowir::program::Signature::Object(_))
        ) {
            return self.object_parameters();
        }
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
                    Representation::PointerObject(_) => Selected::Runtime(Value {
                        ty: Type::Ptr,
                        operand: Operand::Parameter(parameter),
                    }),
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
                    let ty = self.body.values[id.0 as usize].ty;
                    let global = self.context.constant_data(
                        super::data::Key::Temporary(self.body_index, id),
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
        let selected = self
            .values
            .get(id.0 as usize)
            .copied()
            .flatten()
            .ok_or("body value does not dominate its use")?;
        if let Selected::Unit = selected {
            let value = self
                .body
                .values
                .get(id.0 as usize)
                .ok_or("missing evaluated body value")?;
            if matches!(
                self.context.types.kind(value.ty),
                Some(Kind::FunctionItem { .. })
            ) {
                // A call returning an item executes normally but carries no
                // runtime bits. Its checked complete type retains the target.
                return Ok(Selected::Item(
                    self.body
                        .callable_target(Instance::new(self.context.types, value.ty)?),
                ));
            }
        }
        Ok(selected)
    }
    fn primitive(&self, ty: TypeId) -> Result<Primitive, &'static str> {
        if let Some(Kind::Primitive(p)) = self.context.types.kind(ty) {
            Ok(p)
        } else {
            Err("missing selected primitive type")
        }
    }
    fn reify(&mut self, id: ir::ValueId) -> Result<Selected, &'static str> {
        let ty = self.body.values[id.0 as usize].ty;
        let selected = self.value(id)?;
        let symbol = if matches!(self.context.types.kind(ty), Some(Kind::Closure { .. })) {
            Some(self.context.demand_closure(ty)?)
        } else if let Selected::Item(definition) = selected {
            Some(self.context.demand(definition)?)
        } else {
            None
        };
        if let Some(symbol) = symbol {
            self.builder
                .address(Operand::Symbol(symbol))
                .map(Selected::Runtime)
        } else {
            Ok(selected)
        }
    }
    fn scalar(&mut self, scalar: Scalar, ty: Type) -> Result<Value, &'static str> {
        match scalar {
            Scalar::Integer(bits) if ty == Type::Ptr => self
                .builder
                .literal(ty, Literal::Integer(bits & u128::from(u64::MAX))),
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
        self.constant_value(
            super::data::Key::Definition(definition),
            value,
            ty,
            representation,
        )
    }
    fn constant_value(
        &mut self,
        key: super::data::Key,
        value: crate::sema::values::Constant,
        ty: TypeId,
        representation: Representation,
    ) -> Result<Selected, &'static str> {
        match self.context.constants.view(value)? {
            View::Address { bits } if value.metadata().is_none() => self
                .builder
                .literal(Type::Ptr, Literal::Integer(u128::from(bits)))
                .map(Selected::Runtime),
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
                let global = self.context.constant_data(key, value, ty, true)?;
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
            Rvalue::Borrow { place, .. } | Rvalue::Address(place) => {
                let address = self
                    .place(place)?
                    .ok_or("borrowed place lacks stable storage")?;
                let pointer = self.pointer(address)?;
                if matches!(representation, Representation::Object(_)) {
                    let metadata = self.metadata(place)?;
                    return self.wide_pointer(pointer, metadata);
                }
                pointer
            }
            Rvalue::Unit | Rvalue::Validate(_) => return Ok(Selected::Unit),
            Rvalue::Aggregate { variant, fields } => {
                return self.aggregate(value.ty, variant, fields);
            }
            Rvalue::Union {
                field,
                value: operand,
            } => {
                return self.union(value.ty, field, operand);
            }
            Rvalue::Repeat { value: operand, .. } => return self.repeat(value.ty, operand),
            Rvalue::Function(definition) => {
                if !matches!(self.context.types.kind(value.ty), Some(Kind::FunctionItem { item, .. }) if item.0 == definition.0)
                {
                    return Err("function operand differs from its selected item");
                }
                return Ok(Selected::Item(
                    self.body
                        .callable_target(Instance::new(self.context.types, value.ty)?),
                ));
            }
            Rvalue::Scalar(scalar) => self.scalar(self.body.scalars[scalar.0 as usize], ty)?,
            Rvalue::Length(place) => self.slice_length(place)?,
            Rvalue::TextEquals { place, text } => self.text_equals(place, text)?,
            Rvalue::Text(text) => return self.text_literal(text, representation),
            Rvalue::IsVariant { place, variant } => return self.is_variant(place, variant),
            Rvalue::Read(place) => {
                if matches!(representation, Representation::Scalar(Type::Void)) {
                    return Ok(
                        if let Some(Kind::FunctionItem { .. }) = self.context.types.kind(value.ty) {
                            Selected::Item(
                                self.body
                                    .callable_target(Instance::new(self.context.types, value.ty)?),
                            )
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
                    if matches!(
                        representation,
                        Representation::Object(_) | Representation::PointerObject(_)
                    ) {
                        let destination = self
                            .allocate(value.ty)?
                            .ok_or("missing aggregate snapshot")?;
                        self.write(Some(destination), Selected::Object(address), value.ty)?;
                        return Ok(Selected::Object(destination));
                    }
                    self.builder.load(ty, address.operand)?
                } else {
                    return Ok(
                        if let Some(Kind::FunctionItem { .. }) = self.context.types.kind(value.ty) {
                            Selected::Item(
                                self.body
                                    .callable_target(Instance::new(self.context.types, value.ty)?),
                            )
                        } else {
                            Selected::Unit
                        },
                    );
                }
            }
            Rvalue::Constant(definition) => {
                return self.constant(definition, value.ty, representation);
            }
            Rvalue::InlineConstant(source) => {
                let constant = self
                    .context
                    .constants
                    .inline(source)
                    .ok_or("inline constant was not evaluated")?;
                return self.constant_value(
                    super::data::Key::Inline(source),
                    constant,
                    value.ty,
                    representation,
                );
            }
            Rvalue::InlineInstance(instance) => {
                let constant = self
                    .context
                    .constants
                    .inline_instance(instance, value.ty)
                    .ok_or("anonymous instance was not evaluated")?;
                return self.constant_value(
                    super::data::Key::InlineInstance(instance, value.ty),
                    constant,
                    value.ty,
                    representation,
                );
            }
            Rvalue::SourceAssociated(source) => {
                let (instance, constant) = self.context.constants.source_associated(source)?;
                return self.constant_value(
                    super::data::Key::Associated(instance),
                    constant,
                    value.ty,
                    representation,
                );
            }
            Rvalue::AssociatedConstant(instance) => {
                let constant = self
                    .context
                    .constants
                    .associated(instance)
                    .ok_or("associated constant was not evaluated")?;
                return self.constant_value(
                    super::data::Key::Associated(instance),
                    constant,
                    value.ty,
                    representation,
                );
            }
            Rvalue::ConstArgument(id) => {
                let constant = crate::sema::values::Constant::argument(self.context.types, id)?
                    .ok_or("generic const execution requires concrete instantiation")?;
                self.scalar(
                    constant
                        .as_scalar()
                        .map_err(crate::sema::values::Failure::message)?,
                    ty,
                )?
            }
            Rvalue::Reify(operand)
            | Rvalue::Cast {
                kind: Cast::FunctionPointer,
                operand,
                ..
            } => return self.reify(operand),
            Rvalue::Cast {
                kind: Cast::Pointer,
                operand,
                ..
            }
            | Rvalue::Constrain(operand) => {
                return self.constrain(operand, value.ty, representation);
            }
            Rvalue::Cast {
                kind: kind @ (Cast::PointerThin | Cast::PointerToInteger | Cast::IntegerToPointer),
                operand,
                ..
            } => self.pointer_cast(kind, operand, ty)?,
            Rvalue::Unsize {
                value: operand,
                conversion,
            } => {
                return match conversion {
                    crate::types::unsizing::Conversion::Slice(array) => {
                        self.selected_unsizing(operand, array)
                    }
                    crate::types::unsizing::Conversion::Object { concrete, target } => {
                        self.object_unsizing(operand, concrete, target)
                    }
                    crate::types::unsizing::Conversion::Upcast { source, target } => {
                        self.object_upcasting(operand, source, target)
                    }
                };
            }
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
                ..
            } => {
                if matches!(
                    self.context
                        .types
                        .kind(self.body.values[left.0 as usize].ty),
                    Some(crate::types::Kind::Pointer { .. })
                ) {
                    return self
                        .pointer_comparison(operator, left, right)
                        .map(Selected::Runtime);
                }
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
                ..
            } => {
                return self.enum_cast(operand, ty);
            }
            Rvalue::Cast { kind, operand, .. } => {
                let source = self.primitive(self.body.values[operand.0 as usize].ty)?;
                let target = self.primitive(value.ty)?;
                let value = self.value(operand)?.runtime()?;
                casts::cast(&mut self.builder, kind, source, target, value, ty)?
            }
            Rvalue::Intrinsic {
                intrinsic,
                instance,
                arguments,
            } => return self.intrinsic(intrinsic, instance, arguments),
            Rvalue::Invoke {
                kind,
                signature,
                arguments,
            } => return self.invoke(id, kind, signature, arguments, value.ty),
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
        self.call_selected(signature, self.value(callee)?, arguments, result)
    }
    fn call_selected(
        &mut self,
        signature: TypeId,
        callee: Selected,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let callee = match callee {
            Selected::Item(definition) => Operand::Symbol(self.context.demand(definition)?),
            Selected::Runtime(value) => value.operand,
            Selected::Unit | Selected::Object(_) => return Err("non-function used as call target"),
        };
        self.call_operand(signature, callee, arguments, result)
    }
    fn call_operand(
        &mut self,
        signature: TypeId,
        callee: Operand,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let signature = self.context.signature(signature)?;
        if matches!(
            self.context.output.signatures.get(signature.0 as usize),
            Some(crate::lowir::program::Signature::Object(_))
        ) || self.context.output.variadic(signature)?
        {
            return self.boundary_call(signature, callee, arguments, result);
        }
        let representation = self.context.representation(result)?;
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
                    let ty = self.body.values[id.0 as usize].ty;
                    let value = if matches!(
                        self.context.representation(ty)?,
                        Representation::PointerObject(_)
                    ) {
                        self.builder.load(Type::Ptr, address.operand)?
                    } else {
                        self.pointer(address)?
                    };
                    self.arguments.push(value);
                }
                Selected::Unit | Selected::Item(_) => {}
            }
        }
        let value =
            self.builder
                .call(signature, representation.result(), callee, &self.arguments)?;
        if matches!(representation, Representation::PointerObject(_)) {
            let destination = self
                .allocate(result)?
                .ok_or("missing nullable result storage")?;
            self.builder.store(
                value.ok_or("missing nullable result word")?,
                destination.operand,
            )?;
            return Ok(Selected::Object(destination));
        }
        Ok(destination.map_or_else(
            || value.map_or(Selected::Unit, Selected::Runtime),
            Selected::Object,
        ))
    }
}

/// A body's blocks in reverse postorder and their reachability. The buffers
/// are reused across the bodies one context lowers.
#[derive(Default)]
struct BlockOrder {
    blocks: Vec<usize>,
    seen: Vec<bool>,
    stack: Vec<(usize, bool)>,
}
impl BlockOrder {
    /// Reverse postorder places dominating snapshots before uses without
    /// recursion. Returns whether the body has checked (false) edges.
    fn compute(&mut self, body: &ir::Body) -> Result<bool, &'static str> {
        if body.blocks.is_empty() {
            return Err("body has no entry block");
        }
        let (order, seen, stack) = (&mut self.blocks, &mut self.seen, &mut self.stack);
        seen.clear();
        seen.resize(body.blocks.len(), false);
        stack.clear();
        stack.reserve(body.blocks.len() * 2 + 1);
        stack.push((0, false));
        order.clear();
        order.reserve(body.blocks.len());
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
        Ok(checked_edges)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn aggregate_addresses_keep_ordinary_snapshots_compact() {
        assert!(std::mem::size_of::<Option<super::Selected>>() <= 24);
    }
}
