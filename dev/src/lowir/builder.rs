//! Function construction emits contiguous block ranges and diagnoses invalid state.
use super::{
    program::{
        self, Access, Block, BlockId, Function, Instruction, Literal, LiteralId, MemorySpan,
        Operand, Operation, Range, SignatureId, SlotId, SpanId, Storage, TempId, Terminator, Value,
    },
    vocabulary::{Binary, Convert, Predicate, Type, Unary},
};
use std::collections::HashMap;

#[derive(Default)]
pub struct Builder {
    function: Function,
    current: Option<BlockId>,
    opened: Vec<bool>,
    spans: HashMap<MemorySpan, SpanId>,
}
impl Builder {
    /// # Errors
    /// Rejects void storage and exhausted identities.
    pub fn slot(&mut self, ty: Type) -> Result<SlotId, &'static str> {
        if ty == Type::Void {
            return Err("void LowIR slot");
        }
        let id = SlotId(program::index(self.function.slots.len())?);
        self.function.slots.push(Storage::Scalar(ty));
        Ok(id)
    }
    fn span(&mut self, span: MemorySpan) -> Result<SpanId, &'static str> {
        if span.bytes == 0 || span.bytes > i64::MAX as u64 || !span.align.is_power_of_two() {
            return Err("invalid LowIR object span");
        }
        if let Some(id) = self.spans.get(&span) {
            return Ok(*id);
        }
        let id = SpanId(program::index(self.function.spans.len())?);
        self.function.spans.push(span);
        self.spans.insert(span, id);
        Ok(id)
    }
    /// Reserve positive-sized addressable storage. Rust ZSTs need no object slot.
    ///
    /// # Errors
    /// Rejects invalid spans and exhausted identities.
    pub fn object_slot(&mut self, span: MemorySpan) -> Result<SlotId, &'static str> {
        let span = self.span(span)?;
        let id = SlotId(program::index(self.function.slots.len())?);
        self.function.slots.push(Storage::Object(span));
        Ok(id)
    }
    /// Emit a proved non-overlapping object transfer; span alignment is an address fact.
    ///
    /// # Errors
    /// Rejects invalid spans, non-pointer addresses and invalid construction state.
    pub fn copy_object(
        &mut self,
        span: MemorySpan,
        source: Value,
        destination: Value,
    ) -> Result<(), &'static str> {
        if source.ty != Type::Ptr || destination.ty != Type::Ptr {
            return Err("object copy requires pointer operands");
        }
        let span = self.span(span)?;
        self.emit(
            Operation::CopyObject {
                span,
                source: source.operand,
                destination: destination.operand,
            },
            None,
        )
    }
    /// Advance a pointer by an already checked count of scalar elements.
    ///
    /// # Errors
    /// Rejects non-pointer bases, noninteger offsets and void strides.
    pub fn index(
        &mut self,
        element: Type,
        base: Value,
        offset: Value,
    ) -> Result<Value, &'static str> {
        if element == Type::Void || base.ty != Type::Ptr || !offset.ty.is_integer() {
            return Err("invalid LowIR address projection");
        }
        self.value(
            Type::Ptr,
            Operation::Index {
                element,
                base: base.operand,
                offset: offset.operand,
            },
        )
    }
    /// # Errors
    /// Rejects exhausted block identities.
    pub fn block(&mut self) -> Result<BlockId, &'static str> {
        let id = BlockId(program::index(self.function.blocks.len())?);
        self.function.blocks.push(Block {
            instructions: Range { start: 0, end: 0 },
            terminator: Terminator::Unreachable,
        });
        self.opened.push(false);
        Ok(id)
    }
    /// # Errors
    /// Rejects unknown/revisited blocks and unterminated preceding blocks.
    pub fn enter(&mut self, block: BlockId) -> Result<(), &'static str> {
        if self.current.is_some() || self.opened.get(block.0 as usize) != Some(&false) {
            return Err("invalid LowIR block transition");
        }
        let start = program::index(self.function.instructions.len())?;
        self.function.blocks[block.0 as usize].instructions = Range { start, end: start };
        self.opened[block.0 as usize] = true;
        self.function.order.push(block);
        self.current = Some(block);
        Ok(())
    }
    /// # Errors
    /// Rejects instructions outside an open block and exhausted instruction space.
    fn emit(&mut self, operation: Operation, result: Option<TempId>) -> Result<(), &'static str> {
        let block = self.current.ok_or("LowIR instruction without open block")?;
        let end = program::index(self.function.instructions.len() + 1)?;
        self.function
            .instructions
            .push(Instruction { result, operation });
        self.function.blocks[block.0 as usize].instructions.end = end;
        Ok(())
    }
    fn value(&mut self, ty: Type, operation: Operation) -> Result<Value, &'static str> {
        if ty == Type::Void {
            return Err("void LowIR temporary");
        }
        let result = TempId(program::index(self.function.temporaries.len())?);
        self.function.temporaries.push(ty);
        self.emit(operation, Some(result))?;
        Ok(Value {
            ty,
            operand: Operand::Temporary(result),
        })
    }
    /// # Errors
    /// Rejects exhausted literal identities.
    pub fn literal(&mut self, ty: Type, literal: Literal) -> Result<Value, &'static str> {
        let id = LiteralId(program::index(self.function.literals.len())?);
        self.function.literals.push(literal);
        Ok(Value {
            ty,
            operand: Operand::Literal(id),
        })
    }
    /// # Errors
    /// Rejects noninteger types and exhausted literal identities.
    pub fn integer(&mut self, ty: Type, bits: u128) -> Result<Value, &'static str> {
        if !ty.is_integer() {
            return Err("noninteger LowIR literal");
        }
        let mask = if ty.bits() == 128 {
            u128::MAX
        } else {
            (1_u128 << ty.bits()) - 1
        };
        self.literal(ty, Literal::Integer(bits & mask))
    }
    /// # Errors
    /// Rejects width-changing copies and invalid construction state.
    pub fn copy(&mut self, ty: Type, value: Value) -> Result<Value, &'static str> {
        if ty.bits() != value.ty.bits() {
            return Err("LowIR copy changes storage width");
        }
        self.value(ty, Operation::Copy(value))
    }
    /// # Errors
    /// Rejects invalid construction state.
    pub fn address(&mut self, value: Operand) -> Result<Value, &'static str> {
        self.value(Type::Ptr, Operation::Address(value))
    }
    /// # Errors
    /// Rejects unsupported widths and invalid construction state.
    pub fn atomic_load(
        &mut self,
        ty: Type,
        address: Operand,
        ordering: super::atomic::LoadOrder,
    ) -> Result<Value, &'static str> {
        if ty != Type::U32 {
            return Err("atomic load currently requires u32");
        }
        self.value(
            ty,
            Operation::AtomicLoad {
                ty,
                address,
                ordering,
            },
        )
    }
    /// # Errors
    /// Rejects unsupported widths and invalid construction state.
    pub fn atomic_store(
        &mut self,
        value: Value,
        address: Operand,
        ordering: super::atomic::StoreOrder,
    ) -> Result<(), &'static str> {
        if value.ty != Type::U32 {
            return Err("atomic store currently requires u32");
        }
        self.emit(
            Operation::AtomicStore {
                value,
                address,
                ordering,
            },
            None,
        )
    }
    /// # Errors
    /// Rejects unsupported widths and invalid construction state.
    pub fn atomic_exchange(
        &mut self,
        value: Value,
        address: Operand,
        ordering: super::atomic::RmwOrder,
    ) -> Result<Value, &'static str> {
        if value.ty != Type::U32 {
            return Err("atomic exchange currently requires u32");
        }
        self.value(
            value.ty,
            Operation::AtomicExchange {
                value,
                address,
                ordering,
            },
        )
    }
    /// # Errors
    /// Rejects unsupported widths and invalid construction state.
    pub fn atomic_add_fetch(
        &mut self,
        value: Value,
        address: Operand,
        ordering: super::atomic::RmwOrder,
    ) -> Result<Value, &'static str> {
        if value.ty != Type::U32 {
            return Err("atomic add-fetch currently requires u32");
        }
        self.value(
            value.ty,
            Operation::AtomicAddFetch {
                value,
                address,
                ordering,
            },
        )
    }
    /// # Errors
    /// Rejects invalid construction state or void loads.
    pub fn load(&mut self, ty: Type, address: Operand) -> Result<Value, &'static str> {
        self.memory_load(Access::Ordinary, ty, address)
    }
    /// # Errors
    /// Rejects invalid construction state or void loads.
    pub fn volatile_load(&mut self, ty: Type, address: Operand) -> Result<Value, &'static str> {
        self.memory_load(Access::Volatile, ty, address)
    }
    fn memory_load(
        &mut self,
        access: Access,
        ty: Type,
        address: Operand,
    ) -> Result<Value, &'static str> {
        self.value(
            ty,
            Operation::Load {
                access,
                ty,
                address,
            },
        )
    }
    /// # Errors
    /// Rejects invalid construction state or void stores.
    pub fn store(&mut self, value: Value, address: Operand) -> Result<(), &'static str> {
        self.memory_store(Access::Ordinary, value, address)
    }
    /// # Errors
    /// Rejects invalid construction state or void stores.
    pub fn volatile_store(&mut self, value: Value, address: Operand) -> Result<(), &'static str> {
        self.memory_store(Access::Volatile, value, address)
    }
    fn memory_store(
        &mut self,
        access: Access,
        value: Value,
        address: Operand,
    ) -> Result<(), &'static str> {
        if value.ty == Type::Void {
            return Err("void LowIR store");
        }
        self.emit(
            Operation::Store {
                access,
                value,
                address,
            },
            None,
        )
    }
    /// # Errors
    /// Rejects invalid construction state or void operations.
    pub fn unary(&mut self, op: Unary, value: Value) -> Result<Value, &'static str> {
        self.value(value.ty, Operation::Unary { op, value })
    }
    /// # Errors
    /// Rejects mismatched operands and invalid construction state.
    pub fn binary(&mut self, op: Binary, left: Value, right: Value) -> Result<Value, &'static str> {
        if left.ty != right.ty {
            return Err("LowIR binary operand type mismatch");
        }
        self.value(
            left.ty,
            Operation::Binary {
                op,
                left,
                right: right.operand,
            },
        )
    }
    /// # Errors
    /// Rejects mismatched operands and invalid construction state.
    pub fn compare(
        &mut self,
        predicate: Predicate,
        left: Value,
        right: Value,
    ) -> Result<Value, &'static str> {
        if left.ty != right.ty {
            return Err("LowIR comparison operand type mismatch");
        }
        self.value(
            Type::I64,
            Operation::Compare {
                predicate,
                left,
                right: right.operand,
            },
        )
    }
    /// # Errors
    /// Rejects invalid construction state and void results.
    pub fn convert(&mut self, op: Convert, ty: Type, value: Value) -> Result<Value, &'static str> {
        self.value(ty, Operation::Convert { op, value })
    }
    /// # Errors
    /// Rejects invalid construction state or exhausted argument identities.
    pub fn call(
        &mut self,
        signature: SignatureId,
        result: Type,
        callee: Operand,
        arguments: &[Value],
    ) -> Result<Option<Value>, &'static str> {
        let start = program::index(self.function.arguments.len())?;
        let end = program::index(
            self.function
                .arguments
                .len()
                .checked_add(arguments.len())
                .ok_or("LowIR argument space exhausted")?,
        )?;
        self.function.arguments.extend_from_slice(arguments);
        let operation = Operation::Call {
            signature,
            callee,
            arguments: Range { start, end },
        };
        if result == Type::Void {
            self.emit(operation, None)?;
            Ok(None)
        } else {
            self.value(result, operation).map(Some)
        }
    }
    /// # Errors
    /// Rejects termination outside a block and unknown branch targets.
    pub fn terminate(&mut self, terminator: Terminator) -> Result<(), &'static str> {
        let target_valid = |id: BlockId| (id.0 as usize) < self.function.blocks.len();
        match terminator {
            Terminator::Jump(target) if !target_valid(target) => {
                return Err("invalid LowIR jump target");
            }
            Terminator::Branch { yes, no, .. } if !target_valid(yes) || !target_valid(no) => {
                return Err("invalid LowIR branch target");
            }
            _ => {}
        }
        let block = self
            .current
            .take()
            .ok_or("LowIR terminator without open block")?;
        self.function.blocks[block.0 as usize].terminator = terminator;
        Ok(())
    }
    /// # Errors
    /// Rejects unfinished blocks, missing entry blocks and unplaced blocks.
    pub fn finish(self) -> Result<Function, &'static str> {
        if self.current.is_some()
            || self.opened.is_empty()
            || self.opened.iter().any(|opened| !opened)
        {
            return Err("unfinished LowIR function");
        }
        Ok(self.function)
    }
}
