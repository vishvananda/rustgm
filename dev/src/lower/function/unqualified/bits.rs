//! Bit counts and permutations composed from shifts and masks at the operand's width.
use super::{
    super::{Lower, Selected, casts},
    Call,
};
use crate::{
    lower::bit_kernels,
    lowir::{
        builder::Builder,
        program::Value,
        vocabulary::{Binary, Type, Unary},
    },
    support::llvm_symbols::Operation,
    types::{Primitive, TypeId},
};

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_bits(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        let &[input] = call.inputs else {
            return Ok(None);
        };
        let Some(input) = self.integer_input(input)? else {
            return Ok(None);
        };
        let counting = !matches!(call.name, "bswap" | "bitreverse");
        let expected = if counting {
            TypeId::primitive(Primitive::U32)
        } else {
            input.ty
        };
        if call.result != expected {
            return Ok(None);
        }
        let builder = &mut self.builder;
        let word = casts::integer(builder, input.value, unsigned(input.value.ty)?, false)?;
        let result = match call.name {
            "ctpop" => bit_kernels::emit(builder, Operation::Population, word)?,
            // The nonzero forms may assume their input; the total forms are valid for it.
            "ctlz" | "ctlz_nonzero" => leading_zeros(builder, word)?,
            "cttz" | "cttz_nonzero" => trailing_zeros(builder, word)?,
            "bswap" if word.ty.bits() == 8 => word,
            "bswap" => bit_kernels::emit(builder, Operation::SwapBytes, word)?,
            "bitreverse" => bit_kernels::emit(builder, Operation::ReverseBits, word)?,
            _ => return Err("bit intrinsic lost its selected operation"),
        };
        let target = if counting { Type::U32 } else { input.value.ty };
        casts::integer(builder, result, target, false).map(|value| Some(Selected::Runtime(value)))
    }
}

/// The unsigned scalar of the same width; 64 and 128 bits have a single carrier.
fn unsigned(ty: Type) -> Result<Type, &'static str> {
    Ok(match ty.bits() {
        8 => Type::U8,
        16 => Type::U16,
        32 => Type::U32,
        64 => Type::I64,
        128 => Type::I128,
        _ => return Err("bit intrinsic operand has no integer width"),
    })
}
/// Setting every bit below the highest set bit leaves exactly the leading
/// zeros clear, so zero yields the width.
fn leading_zeros(builder: &mut Builder, word: Value) -> Result<Value, &'static str> {
    let mut smeared = word;
    let mut shift = 1;
    while shift < word.ty.bits() {
        let count = builder.integer(word.ty, u128::from(shift))?;
        let lower = builder.binary(Binary::Ushr, smeared, count)?;
        smeared = builder.binary(Binary::Or, smeared, lower)?;
        shift *= 2;
    }
    let clear = builder.unary(Unary::BitNot, smeared)?;
    bit_kernels::emit(builder, Operation::Population, clear)
}
/// The clear bits below the lowest set bit are exactly the trailing zeros, so
/// zero yields the width.
fn trailing_zeros(builder: &mut Builder, word: Value) -> Result<Value, &'static str> {
    let one = builder.integer(word.ty, 1)?;
    let below = builder.binary(Binary::Sub, word, one)?;
    let clear = builder.unary(Unary::BitNot, word)?;
    let trailing = builder.binary(Binary::And, clear, below)?;
    bit_kernels::emit(builder, Operation::Population, trailing)
}
