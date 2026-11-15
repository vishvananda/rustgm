//! Sign-bit operations on 32- and 64-bit floating values.
use super::{
    super::{Lower, Selected},
    Call,
};
use crate::{
    body::ir,
    lowir::{
        program::{Operand, Value},
        vocabulary::{Binary, Type},
    },
};

impl Lower<'_, '_, '_> {
    /// The magnitude with a cleared sign bit, or with the sign bit of the
    /// second input. Every other bit is kept, as for a NaN payload.
    pub(super) fn unqualified_float(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        let (magnitude, sign) = match (call.name, call.inputs) {
            ("fabs", &[value]) => (value, None),
            ("copysignf32" | "copysignf64", &[magnitude, sign]) => (magnitude, Some(sign)),
            _ => return Ok(None),
        };
        let Some(magnitude) = self.float_input(call, magnitude)? else {
            return Ok(None);
        };
        let sign = match sign {
            Some(sign) => match self.float_input(call, sign)? {
                Some(sign) if sign.ty == magnitude.ty => Some(sign),
                _ => return Ok(None),
            },
            None => None,
        };
        let (carrier, top) = if magnitude.ty == Type::F32 {
            (Type::U32, 1_u128 << 31)
        } else {
            (Type::I64, 1_u128 << 63)
        };
        let builder = &mut self.builder;
        let kept = builder.integer(carrier, top - 1)?;
        let bits = builder.copy(carrier, magnitude)?;
        let mut bits = builder.binary(Binary::And, bits, kept)?;
        if let Some(sign) = sign {
            let top = builder.integer(carrier, top)?;
            let sign = builder.copy(carrier, sign)?;
            let sign = builder.binary(Binary::And, sign, top)?;
            bits = builder.binary(Binary::Or, bits, sign)?;
        }
        // The pinned backend does not select integer-register to float-register
        // copies; untyped local storage carries the bits across.
        let slot = Operand::Slot(builder.slot(magnitude.ty)?);
        builder.store(bits, slot)?;
        builder
            .load(magnitude.ty, slot)
            .map(|value| Some(Selected::Runtime(value)))
    }
    /// The input when it has the call's result type and a 32- or 64-bit
    /// floating value.
    fn float_input(&self, call: Call<'_>, id: ir::ValueId) -> Result<Option<Value>, &'static str> {
        if self.input_type(id)? != call.result {
            return Ok(None);
        }
        let Selected::Runtime(value) = self.value(id)? else {
            return Ok(None);
        };
        let named = match call.name {
            "copysignf32" => value.ty == Type::F32,
            "copysignf64" => value.ty == Type::F64,
            _ => value.ty.is_float(),
        };
        Ok(named.then_some(value))
    }
}
