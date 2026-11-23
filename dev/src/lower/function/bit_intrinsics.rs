//! Bit counts and permutations composed from shifts and masks at the operand's width.
use super::{Lower, Selected, casts, ir};
use crate::{
    lower::bit_kernels,
    lowir::{
        builder::Builder,
        program::Value,
        vocabulary::{Binary, Type, Unary},
    },
    sema::compiler_items::bit_counts::Effect,
    support::llvm_symbols::Operation,
    types::{Kind, TypeId, instance::Instance},
};

/// An evaluated integer input carried at its own width.
#[derive(Clone, Copy)]
struct Integer {
    ty: TypeId,
    value: Value,
}

impl Lower<'_, '_, '_> {
    pub(super) fn source_bit_count(
        &mut self,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Option<Selected>, &'static str> {
        let Some(&[argument]) = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
        else {
            return Err("source bit count requires one evaluated integer input");
        };
        let Some(input) = self.integer_input(argument)? else {
            if crate::support::bringup::selected().is_some() {
                return Ok(None);
            }
            return Err("source bit count execution requires a closed integer type");
        };
        let instance = Instance::new(self.context.types, callee)?;
        let selected = self
            .context
            .constants
            .source_bit_count(instance, self.body.source)?
            .ok_or("source bit count lost its validated instance")?;
        if selected.input() != input.ty
            || selected.output() != result
            || !selected.opened_matches(self.context.ledger, signature)
        {
            return Err("source bit count differs from its evaluated Call");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let effect = selected.effect();
        let builder = &mut self.builder;
        let word = casts::integer(builder, input.value, unsigned(input.value.ty)?, false)?;
        let value = match effect {
            Effect::Ones => bit_kernels::emit(builder, Operation::Population, word)?,
            Effect::LeadingZeros => leading_zeros(builder, word)?,
            Effect::TrailingZeros => trailing_zeros(builder, word)?,
            Effect::SwappedBytes if word.ty.bits() == 8 => word,
            Effect::SwappedBytes => bit_kernels::emit(builder, Operation::SwapBytes, word)?,
            Effect::ReversedBits => bit_kernels::emit(builder, Operation::ReverseBits, word)?,
        };
        let target = if matches!(effect, Effect::SwappedBytes | Effect::ReversedBits) {
            input.value.ty
        } else {
            Type::U32
        };
        casts::integer(builder, value, target, false).map(|value| Some(Selected::Runtime(value)))
    }

    /// The input when its type is an integer primitive.
    fn integer_input(&self, id: ir::ValueId) -> Result<Option<Integer>, &'static str> {
        let ty = self.input_type(id)?;
        let Some(Kind::Primitive(primitive)) = self.context.types.kind(ty) else {
            return Ok(None);
        };
        let Some((_, bits)) = primitive.integer() else {
            return Ok(None);
        };
        let value = self.value(id)?.runtime()?;
        if !value.ty.is_integer() || value.ty.bits() != u32::from(bits) {
            return Err("integer intrinsic input differs from its scalar width");
        }
        Ok(Some(Integer { ty, value }))
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
