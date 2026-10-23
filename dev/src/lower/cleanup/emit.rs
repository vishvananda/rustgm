//! Worklist emission keeps field/variant order without recursive code generation.
use super::{Address, Builder, Condition, Context, Plan, State, Type, TypeId, glue};
use crate::{
    abi::layout::{Layout, Shape},
    borrow::destruction::{NodeId, Operation},
    lowir::{
        program::{BlockId, MemorySpan, Operand, SlotId, Terminator, Value},
        vocabulary::{Binary, Predicate},
    },
};
#[derive(Clone, Copy)]
pub(super) enum Target {
    Glue(glue::Id),
    Selected(NodeId),
}
#[derive(Clone, Copy)]
enum Source {
    Glue,
    Selected,
}
#[derive(Clone, Copy)]
pub(super) struct Sequence {
    source: Source,
    range: glue::Range,
    ty: TypeId,
    base: Address,
}
pub(super) enum Work {
    Drop(Target, TypeId, Address),
    Fields(Sequence),
    Variants(Sequence, Option<Value>, BlockId),
    Intervals(glue::Range, TypeId, Address),
    Jump {
        target: BlockId,
        enter: BlockId,
    },
    Advance {
        slot: SlotId,
        index: Value,
        header: BlockId,
        end: BlockId,
    },
}
pub(super) struct Emitter<'a, 'b> {
    pub context: &'a mut Context<'b>,
    pub builder: &'a mut Builder,
    pub state: &'a mut State,
    pub plan: Option<&'a Plan>,
}
impl Emitter<'_, '_> {
    fn recipes(&self) -> Result<&glue::Recipes, &'static str> {
        self.context
            .destruction
            .map(crate::borrow::destruction::Proof::recipes)
            .ok_or("missing checked cleanup recipes")
    }
    fn plan(&self) -> Result<&Plan, &'static str> {
        self.plan.ok_or("missing selected cleanup plan")
    }
    pub fn selected(
        &mut self,
        node: NodeId,
        ty: TypeId,
        base: Address,
    ) -> Result<(), &'static str> {
        self.state
            .work
            .push(Work::Drop(Target::Selected(node), ty, base));
        self.run()
    }
    pub fn recipe(&mut self, id: glue::Id) -> Result<(), &'static str> {
        let recipe = *self.recipes()?.get(id).ok_or("invalid cleanup recipe")?;
        if let glue::Shape::Slice { .. } = recipe.kind {
            let (element, child) = self.recipes()?.slice(self.context.types, id)?;
            self.slice(element, child)?;
            return self.run();
        }
        let layout = self.context.layout(recipe.ty)?;
        let base = Address {
            operand: Operand::Parameter(0),
            align: layout.align,
        };
        if let Some(method) = recipe.method {
            let symbol = match method {
                glue::Method::Definition(method) => self.context.demand_definition(method)?,
                glue::Method::Instance(instance) => self.context.demand(instance)?,
                glue::Method::Unresolved(_) => {
                    return Err("generic destructor requires a selected instance");
                }
            };
            let signature = self.context.output.symbols[symbol.0 as usize].signature;
            let pointer = base.pointer(self.builder)?;
            self.builder
                .call(signature, Type::Void, Operand::Symbol(symbol), &[pointer])?;
        }
        match recipe.kind {
            glue::Shape::Fields(range) => self.state.work.push(Work::Fields(Sequence {
                source: Source::Glue,
                range,
                ty: recipe.ty,
                base,
            })),
            glue::Shape::Variants(range) => self.variants(Sequence {
                source: Source::Glue,
                range,
                ty: recipe.ty,
                base,
            })?,
            glue::Shape::Array { element, length } => {
                self.array(Target::Glue(element), recipe.ty, base, 0, length)?;
            }
            glue::Shape::Slice { .. } => return Err("slice cleanup lost its wide entry"),
        }
        self.run()
    }
    fn run(&mut self) -> Result<(), &'static str> {
        while let Some(work) = self.state.work.pop() {
            self.state.charge(1)?;
            match work {
                Work::Drop(target, ty, base) => self.drop(target, ty, base)?,
                Work::Fields(sequence) => self.field(sequence)?,
                Work::Variants(sequence, tag, join) => self.variant(sequence, tag, join)?,
                Work::Intervals(range, ty, base) => self.interval(range, ty, base)?,
                Work::Jump { target, enter } => {
                    self.builder.terminate(Terminator::Jump(target))?;
                    self.builder.enter(enter)?;
                }
                Work::Advance {
                    slot,
                    index,
                    header,
                    end,
                } => {
                    let one = self.builder.integer(Type::I64, 1)?;
                    let next = self.builder.binary(Binary::Add, index, one)?;
                    self.builder.store(next, Operand::Slot(slot))?;
                    self.builder.terminate(Terminator::Jump(header))?;
                    self.builder.enter(end)?;
                }
            }
        }
        Ok(())
    }
    fn guard(&mut self, condition: Condition) -> Result<(), &'static str> {
        if let Condition::Initialized(flag) = condition {
            let index = self
                .plan()?
                .flags
                .binary_search(&flag)
                .map_err(|_| "unknown cleanup condition flag")?;
            let slot = *self
                .state
                .flags
                .get(index)
                .ok_or("missing cleanup flag slot")?;
            let value = self.builder.load(Type::U8, Operand::Slot(slot))?;
            let yes = self.builder.block()?;
            let join = self.builder.block()?;
            self.builder.terminate(Terminator::Branch {
                condition: value.operand,
                yes,
                no: join,
            })?;
            self.builder.enter(yes)?;
            self.state.work.push(Work::Jump {
                target: join,
                enter: join,
            });
        }
        Ok(())
    }
    fn drop(&mut self, target: Target, ty: TypeId, base: Address) -> Result<(), &'static str> {
        let Target::Selected(id) = target else {
            let Target::Glue(id) = target else {
                return Err("invalid cleanup call");
            };
            return self.call(id, ty, base);
        };
        let node = self
            .plan()?
            .nodes
            .get(id.0 as usize)
            .ok_or("invalid selected cleanup node")?;
        let (condition, operation) = (node.condition, node.operation);
        self.guard(condition)?;
        match operation {
            Operation::Empty => {}
            Operation::Glue(id) => self.call(id, ty, base)?,
            Operation::Fields(range) => self.state.work.push(Work::Fields(Sequence {
                source: Source::Selected,
                range,
                ty,
                base,
            })),
            Operation::Variants(range) => self.variants(Sequence {
                source: Source::Selected,
                range,
                ty,
                base,
            })?,
            Operation::Array(range) => self.state.work.push(Work::Intervals(range, ty, base)),
        }
        Ok(())
    }
    fn call(&mut self, id: glue::Id, ty: TypeId, mut base: Address) -> Result<(), &'static str> {
        let layout = self.context.layout(ty)?;
        if base.align < layout.align {
            let slot = self.builder.object_slot(MemorySpan {
                bytes: layout.size.max(u64::from(layout.align)),
                align: layout.align,
            })?;
            let aligned = Address {
                operand: Operand::Slot(slot),
                align: layout.align,
            };
            if layout.size != 0 {
                let source = base.pointer(self.builder)?;
                let destination = aligned.pointer(self.builder)?;
                self.builder.copy_object(
                    MemorySpan {
                        bytes: layout.size,
                        align: base.align,
                    },
                    source,
                    destination,
                )?;
            }
            base = aligned;
        }
        let symbol = self.context.demand_drop(id)?;
        let signature = self.context.output.symbols[symbol.0 as usize].signature;
        let pointer = base.pointer(self.builder)?;
        self.builder
            .call(signature, Type::Void, Operand::Symbol(symbol), &[pointer])?;
        Ok(())
    }
    fn field(&mut self, mut sequence: Sequence) -> Result<(), &'static str> {
        if sequence.range.start == sequence.range.end {
            return Ok(());
        }
        let index = sequence.range.start;
        let (selection, target) = match sequence.source {
            Source::Glue => {
                let field = *self
                    .recipes()?
                    .fields(sequence.range)
                    .and_then(|f| f.first())
                    .ok_or("invalid glue field")?;
                (field.selection, Target::Glue(field.recipe))
            }
            Source::Selected => {
                let field = self
                    .plan()?
                    .fields
                    .get(index as usize)
                    .ok_or("invalid selected drop field")?;
                (field.selection, Target::Selected(field.node))
            }
        };
        sequence.range.start += 1;
        self.state.work.push(Work::Fields(sequence));
        let layout = self.context.layout(sequence.ty)?;
        let (ty, offset) = self.context.selected_field(&layout, selection)?;
        let base = sequence.base.offset(self.builder, offset)?;
        self.state.work.push(Work::Drop(target, ty, base));
        Ok(())
    }
    fn variants(&mut self, sequence: Sequence) -> Result<(), &'static str> {
        let layout = self.context.layout(sequence.ty)?;
        let tag = if let Shape::Aggregate { tag: Some(tag), .. } = layout.shape {
            let ty = super::super::context::representation(
                self.context.types,
                TypeId::primitive(tag.ty),
            )?;
            let address = sequence.base.offset(self.builder, tag.offset)?;
            Some(self.builder.load(ty, address.operand)?)
        } else if let Some(niche) = self.context.layouts.niche(&layout)? {
            let address = sequence.base.offset(self.builder, niche.offset)?;
            Some(self.builder.load(Type::Ptr, address.operand)?)
        } else {
            None
        };
        let join = self.builder.block()?;
        self.state.work.push(Work::Variants(sequence, tag, join));
        Ok(())
    }
    fn variant(
        &mut self,
        mut sequence: Sequence,
        tag: Option<Value>,
        join: BlockId,
    ) -> Result<(), &'static str> {
        if sequence.range.start == sequence.range.end {
            self.builder.terminate(Terminator::Jump(join))?;
            return self.builder.enter(join);
        }
        let (selection, work) = match sequence.source {
            Source::Glue => {
                let variant = *self
                    .recipes()?
                    .variants(sequence.range)
                    .and_then(|v| v.first())
                    .ok_or("invalid glue variant")?;
                (
                    variant.selection,
                    Work::Fields(Sequence {
                        range: variant.fields,
                        ..sequence
                    }),
                )
            }
            Source::Selected => {
                let variant = self
                    .plan()?
                    .variants
                    .get(sequence.range.start as usize)
                    .ok_or("invalid selected drop variant")?;
                (
                    variant.selection,
                    Work::Drop(Target::Selected(variant.node), sequence.ty, sequence.base),
                )
            }
        };
        sequence.range.start += 1;
        if let Some(tag) = tag {
            let layout = self.context.layout(sequence.ty)?;
            let ordinal = self
                .context
                .members
                .variants
                .get(selection.0 as usize)
                .ok_or("invalid drop variant identity")?
                .ordinal;
            let discriminant = self
                .context
                .layouts
                .variants(&layout)
                .get(ordinal as usize)
                .ok_or("missing drop variant layout")?
                .discriminant;
            let (expected, predicate) = if let Some(niche) = self.context.layouts.niche(&layout)? {
                let predicate = if ordinal == niche.empty {
                    Predicate::Eq
                } else if ordinal == niche.payload {
                    Predicate::Ne
                } else {
                    return Err("invalid nullable cleanup variant");
                };
                (
                    self.builder
                        .literal(Type::Ptr, crate::lowir::program::Literal::Null)?,
                    predicate,
                )
            } else {
                (self.builder.integer(tag.ty, discriminant)?, Predicate::Eq)
            };
            let condition = self.builder.compare(predicate, tag, expected)?;
            let yes = self.builder.block()?;
            let no = self.builder.block()?;
            self.builder.terminate(Terminator::Branch {
                condition: condition.operand,
                yes,
                no,
            })?;
            self.builder.enter(yes)?;
            self.state
                .work
                .push(Work::Variants(sequence, Some(tag), join));
            self.state.work.push(Work::Jump {
                target: join,
                enter: no,
            });
        } else {
            if sequence.range.start != sequence.range.end {
                return Err("multiple untagged cleanup variants");
            }
            self.state.work.push(Work::Jump {
                target: join,
                enter: join,
            });
        }
        self.state.work.push(work);
        Ok(())
    }
    fn interval(
        &mut self,
        mut range: glue::Range,
        ty: TypeId,
        base: Address,
    ) -> Result<(), &'static str> {
        if range.start == range.end {
            return Ok(());
        }
        let interval = self
            .plan()?
            .intervals
            .get(range.start as usize)
            .ok_or("invalid cleanup array interval")?;
        let (start, end, condition, element) = (
            interval.start,
            interval.end,
            interval.condition,
            interval.element,
        );
        range.start += 1;
        self.state.work.push(Work::Intervals(range, ty, base));
        self.guard(condition)?;
        self.array(Target::Selected(element), ty, base, start, end)
    }
    fn array(
        &mut self,
        target: Target,
        ty: TypeId,
        base: Address,
        start: u64,
        end: u64,
    ) -> Result<(), &'static str> {
        let Layout {
            shape:
                Shape::Array {
                    element,
                    stride,
                    length,
                },
            ..
        } = self.context.layout(ty)?
        else {
            return Err("cleanup array has no array layout");
        };
        if start > end || end > length {
            return Err("invalid cleanup array bounds");
        }
        if start == end {
            return Ok(());
        }
        if end - start == 1 {
            let base = base.offset(
                self.builder,
                start
                    .checked_mul(stride)
                    .ok_or("cleanup array offset overflow")?,
            )?;
            self.state.work.push(Work::Drop(target, element, base));
            return Ok(());
        }
        let initial = self.builder.integer(Type::I64, u128::from(start))?;
        let limit = self.builder.integer(Type::I64, u128::from(end))?;
        self.repeat(target, element, base, stride, initial, limit)
    }
    fn slice(&mut self, element: TypeId, child: glue::Id) -> Result<(), &'static str> {
        if self.context.types.is_sized(element) != Some(true) {
            return Err("slice cleanup requires a completed sized element");
        }
        let layout = self.context.layout(element)?;
        let base = Address {
            operand: Operand::Parameter(0),
            align: layout.align,
        };
        let initial = self.builder.integer(Type::I64, 0)?;
        let limit = Value {
            ty: Type::I64,
            operand: Operand::Parameter(1),
        };
        self.repeat(
            Target::Glue(child),
            element,
            base,
            layout.size,
            initial,
            limit,
        )
    }
    fn repeat(
        &mut self,
        target: Target,
        element: TypeId,
        base: Address,
        stride: u64,
        initial: Value,
        limit: Value,
    ) -> Result<(), &'static str> {
        let slot = self.builder.slot(Type::I64)?;
        self.builder.store(initial, Operand::Slot(slot))?;
        let header = self.builder.block()?;
        let body = self.builder.block()?;
        let done = self.builder.block()?;
        self.builder.terminate(Terminator::Jump(header))?;
        self.builder.enter(header)?;
        let index = self.builder.load(Type::I64, Operand::Slot(slot))?;
        let condition = self.builder.compare(Predicate::Ult, index, limit)?;
        self.builder.terminate(Terminator::Branch {
            condition: condition.operand,
            yes: body,
            no: done,
        })?;
        self.builder.enter(body)?;
        let align = self.context.layout(element)?.align;
        let base = base.indexed(self.builder, index, stride, align)?;
        self.state.work.push(Work::Advance {
            slot,
            index,
            header,
            end: done,
        });
        self.state.work.push(Work::Drop(target, element, base));
        Ok(())
    }
}
