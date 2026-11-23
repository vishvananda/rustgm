//! The typed swap copies object representations with bounded scratch storage.
use super::{Builder, Context, Instance, Operand, SymbolId, Terminator, Type, TypeId, Value};
use crate::lowir::{
    program::{Function, MemorySpan},
    vocabulary::{Binary, Predicate},
};

const CHUNK_BYTES: u32 = 64;

impl Context<'_> {
    pub(in crate::lower) fn demand_typed_swap(
        &mut self,
        instance: Instance,
        signature: TypeId,
        element: TypeId,
    ) -> Result<SymbolId, &'static str> {
        if let Some(&symbol) = self.intrinsic_symbols.get(&instance) {
            return Ok(symbol);
        }
        let layout = self.layout(element)?;
        let body = exchange_body(MemorySpan {
            bytes: layout.size,
            align: layout.align,
        })?;
        self.install_memory_shim(
            instance,
            signature,
            format!("rg.source.typed_swap_nonoverlapping.t{}", instance.ty().0),
            body,
        )
    }
}

fn exchange_body(layout: MemorySpan) -> Result<Function, &'static str> {
    let mut builder = Builder::default();
    let entry = builder.block()?;
    builder.enter(entry)?;
    if layout.bytes > 0 {
        let chunk_bytes = u64::from(CHUNK_BYTES);
        let span = MemorySpan {
            bytes: layout.bytes.min(chunk_bytes),
            align: layout.align.min(CHUNK_BYTES),
        };
        let slot = builder.object_slot(span)?;
        let scratch = builder.address(Operand::Slot(slot))?;
        if layout.bytes <= chunk_bytes {
            exchange(&mut builder, span, parameter(0), parameter(1), scratch)?;
        } else {
            exchange_chunks(&mut builder, layout.bytes, span, scratch)?;
        }
    }
    builder.terminate(Terminator::Return(None))?;
    builder.finish()
}

fn exchange_chunks(
    builder: &mut Builder,
    bytes: u64,
    chunk: MemorySpan,
    scratch: Value,
) -> Result<(), &'static str> {
    let end = bytes - bytes % chunk.bytes;
    let cursor = Operand::Slot(builder.slot(Type::I64)?);
    let zero = builder.integer(Type::I64, 0)?;
    let limit = builder.integer(Type::I64, u128::from(end))?;
    let stride = builder.integer(Type::I64, u128::from(chunk.bytes))?;
    builder.store(zero, cursor)?;
    let (header, body, tail) = (builder.block()?, builder.block()?, builder.block()?);
    builder.terminate(Terminator::Jump(header))?;
    builder.enter(header)?;
    let offset = builder.load(Type::I64, cursor)?;
    let more = builder.compare(Predicate::Ult, offset, limit)?;
    builder.terminate(Terminator::Branch {
        condition: more.operand,
        yes: body,
        no: tail,
    })?;
    builder.enter(body)?;
    exchange_at(builder, offset, chunk, scratch)?;
    let next = builder.binary(Binary::Add, offset, stride)?;
    builder.store(next, cursor)?;
    builder.terminate(Terminator::Jump(header))?;
    builder.enter(tail)?;
    if bytes > end {
        exchange_at(
            builder,
            limit,
            MemorySpan {
                bytes: bytes - end,
                align: chunk.align,
            },
            scratch,
        )?;
    }
    Ok(())
}

fn parameter(index: u32) -> Value {
    Value {
        ty: Type::Ptr,
        operand: Operand::Parameter(index),
    }
}

fn exchange_at(
    builder: &mut Builder,
    offset: Value,
    span: MemorySpan,
    scratch: Value,
) -> Result<(), &'static str> {
    let left = builder.index(Type::U8, parameter(0), offset)?;
    let right = builder.index(Type::U8, parameter(1), offset)?;
    exchange(builder, span, left, right, scratch)
}

fn exchange(
    builder: &mut Builder,
    span: MemorySpan,
    left: Value,
    right: Value,
    scratch: Value,
) -> Result<(), &'static str> {
    // The intrinsic requires non-overlapping addresses. Scratch is a fresh
    // slot, and raw object copies preserve padding and run no destructors.
    builder.copy_object(span, left, scratch)?;
    builder.copy_object(span, right, left)?;
    builder.copy_object(span, scratch, right)
}
