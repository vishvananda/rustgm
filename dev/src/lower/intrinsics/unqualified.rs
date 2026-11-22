//! Memory shims for counted transfers, fills and byte comparisons.
use super::{
    Builder, Context, Instance, Intrinsic, Operand, SymbolId, Terminator, Type, TypeId, Value,
};
use crate::lowir::{
    program::Function,
    vocabulary::{Binary, Convert, Predicate},
};

impl Context<'_> {
    pub(in crate::lower) fn demand_compare_bytes(
        &mut self,
        instance: Instance,
        signature: TypeId,
    ) -> Result<SymbolId, &'static str> {
        if let Some(&symbol) = self.intrinsic_symbols.get(&instance) {
            return Ok(symbol);
        }
        self.install_memory_shim(
            instance,
            signature,
            format!("rg.source.compare_bytes.t{}", instance.ty().0),
            compared(None)?,
        )
    }

    /// A shim with the call's signature for an intrinsic over `element`: the
    /// bootstrap body of the same shape, or a bytewise comparison. `None`
    /// names an intrinsic without one. The caller waives the fence.
    pub(in crate::lower) fn demand_unqualified(
        &mut self,
        instance: Instance,
        signature: TypeId,
        name: &str,
        element: TypeId,
    ) -> Result<Option<SymbolId>, &'static str> {
        if let Some(&symbol) = self.intrinsic_symbols.get(&instance) {
            return Ok(Some(symbol));
        }
        let body = match name {
            "copy_nonoverlapping" => Self::counted_memory_body(
                Intrinsic::CopyNonoverlapping,
                self.memory_type(element)?,
            )?,
            "copy" => Self::counted_memory_body(Intrinsic::Copy, self.memory_type(element)?)?,
            "write_bytes" => Self::fill_memory_body(self.memory_type(element)?)?,
            "raw_eq" => compared(Some(self.layout(element)?.size))?,
            _ => return Ok(None),
        };
        self.install_memory_shim(
            instance,
            signature,
            format!("rg.unqualified.{name}.t{}", instance.ty().0),
            body,
        )
        .map(Some)
    }

    fn install_memory_shim(
        &mut self,
        instance: Instance,
        signature: TypeId,
        name: String,
        body: Function,
    ) -> Result<SymbolId, &'static str> {
        self.reserve_shim()?;
        let signature = self.signature(signature)?;
        let symbol = self
            .output
            .declare(self.internal_name(name), signature, false)?;
        self.output.define(symbol, body)?;
        self.intrinsic_symbols.insert(instance, symbol);
        Ok(symbol)
    }
}

/// Compare the bytes behind the first two parameters in address order. With
/// a fixed length the result is whether all are equal. Otherwise the third
/// parameter counts them and the result is the difference of the first
/// unequal pair, read as unsigned bytes, or zero.
fn compared(fixed: Option<u64>) -> Result<Function, &'static str> {
    let mut builder = Builder::default();
    let entry = builder.block()?;
    builder.enter(entry)?;
    let pointer = |parameter| Value {
        ty: Type::Ptr,
        operand: Operand::Parameter(parameter),
    };
    let count = match fixed {
        Some(bytes) => builder.integer(Type::I64, u128::from(bytes))?,
        None => Value {
            ty: Type::I64,
            operand: Operand::Parameter(2),
        },
    };
    let cursor = Operand::Slot(builder.slot(Type::I64)?);
    let zero = builder.integer(Type::I64, 0)?;
    builder.store(zero, cursor)?;
    let (header, pair) = (builder.block()?, builder.block()?);
    let (unequal, equal) = (builder.block()?, builder.block()?);
    builder.terminate(Terminator::Jump(header))?;
    builder.enter(header)?;
    let index = builder.load(Type::I64, cursor)?;
    let more = builder.compare(Predicate::Ult, index, count)?;
    builder.terminate(Terminator::Branch {
        condition: more.operand,
        yes: pair,
        no: equal,
    })?;
    builder.enter(pair)?;
    let left = builder.index(Type::U8, pointer(0), index)?;
    let left = builder.load(Type::U8, left.operand)?;
    let right = builder.index(Type::U8, pointer(1), index)?;
    let right = builder.load(Type::U8, right.operand)?;
    let one = builder.integer(Type::I64, 1)?;
    let next = builder.binary(Binary::Add, index, one)?;
    builder.store(next, cursor)?;
    let same = builder.compare(Predicate::Eq, left, right)?;
    builder.terminate(Terminator::Branch {
        condition: same.operand,
        yes: header,
        no: unequal,
    })?;
    builder.enter(unequal)?;
    let difference = if fixed.is_some() {
        builder.integer(Type::I1, 0)?
    } else {
        let left = builder.convert(Convert::Zext, Type::I32, left)?;
        let right = builder.convert(Convert::Zext, Type::I32, right)?;
        builder.binary(Binary::Sub, left, right)?
    };
    builder.terminate(Terminator::Return(Some(difference)))?;
    builder.enter(equal)?;
    let all = if fixed.is_some() {
        builder.integer(Type::I1, 1)?
    } else {
        builder.integer(Type::I32, 0)?
    };
    builder.terminate(Terminator::Return(Some(all)))?;
    builder.finish()
}
