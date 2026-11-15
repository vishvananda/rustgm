//! Overflow-reporting, saturating and exact integer operations at their own width.
use super::{
    super::{Arithmetic, Lower, Selected, casts},
    Call, Integer,
};
use crate::{
    lex::token::Punctuation,
    lowir::{
        builder::Builder,
        program::Value,
        vocabulary::{Binary, Predicate},
    },
    types::{Kind, Primitive, TypeId},
};

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_arithmetic(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        let &[left, right] = call.inputs else {
            return Ok(None);
        };
        let (Some(left), Some(right)) = (self.integer_input(left)?, self.integer_input(right)?)
        else {
            return Ok(None);
        };
        let paired = call.name == "add_with_overflow";
        if left.ty != right.ty || !self.arithmetic_result(call.result, left.ty, paired) {
            return Ok(None);
        }
        if call.name == "exact_div" {
            // The remainder is zero by contract, so this is plain division.
            return self
                .scalar_binary(Punctuation::Slash, left.value, right.value, left.signed)
                .map(Some);
        }
        let subtract = call.name == "saturating_sub";
        let mut arithmetic = Arithmetic {
            constants: self.context.constants,
            builder: &mut self.builder,
            panic: &mut self.panic,
        };
        let (wrapped, overflow) = if subtract {
            arithmetic.subtract_overflow(left.signed, left.value, right.value)?
        } else {
            arithmetic.add_overflow(left.signed, left.value, right.value)?
        };
        if paired {
            return self
                .store_subtraction_tuple(wrapped, overflow, left.ty, call.result)
                .map(Some);
        }
        saturate(&mut self.builder, subtract, left, wrapped, overflow)
            .map(|value| Some(Selected::Runtime(value)))
    }
    /// The operand type itself, or its pair with the overflow flag.
    pub(super) fn arithmetic_result(&self, result: TypeId, operand: TypeId, paired: bool) -> bool {
        if !paired {
            return result == operand;
        }
        let Some(Kind::Tuple(fields)) = self.context.types.kind(result) else {
            return false;
        };
        self.context
            .types
            .list(fields)
            .is_some_and(|fields| fields == [operand, TypeId::primitive(Primitive::Bool)])
    }
}

/// Replace a wrapped sum or difference by its bound when the flag is set.
/// A signed operation overflows towards the sign of its left operand.
fn saturate(
    builder: &mut Builder,
    subtract: bool,
    left: Integer,
    wrapped: Value,
    overflow: Value,
) -> Result<Value, &'static str> {
    let ty = wrapped.ty;
    let zero = builder.integer(ty, 0)?;
    let flag = casts::integer(builder, overflow, ty, false)?;
    let mask = builder.binary(Binary::Sub, zero, flag)?;
    let bound = if left.signed {
        let negative = builder.compare(Predicate::Lt, left.value, zero)?;
        let negative = casts::integer(builder, negative, ty, false)?;
        let maximum = builder.integer(ty, (1_u128 << (ty.bits() - 1)) - 1)?;
        // The maximum plus one wraps to the minimum.
        builder.binary(Binary::Add, maximum, negative)?
    } else if subtract {
        zero
    } else {
        builder.integer(ty, u128::MAX)?
    };
    let differing = builder.binary(Binary::Xor, wrapped, bound)?;
    let replaced = builder.binary(Binary::And, differing, mask)?;
    builder.binary(Binary::Xor, wrapped, replaced)
}
