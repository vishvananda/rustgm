//! A product flag never evaluates signed division or divides by a zero operand.
use super::{Arithmetic, Binary, Convert, Predicate, Type, Value, casts};

impl Arithmetic<'_, '_> {
    pub(in crate::lower) fn multiply_overflow(
        &mut self,
        signed: bool,
        left: Value,
        right: Value,
    ) -> Result<(Value, Value), &'static str> {
        let wide = match left.ty.bits() {
            8 => Type::I16,
            16 => Type::I32,
            32 => Type::I64,
            64 => Type::I128,
            128 => return self.full_product(signed, left, right),
            _ => return Err("invalid checked arithmetic width"),
        };
        let lhs = casts::integer(self.builder, left, wide, signed)?;
        let rhs = casts::integer(self.builder, right, wide, signed)?;
        let result = self.builder.binary(Binary::Mul, lhs, rhs)?;
        let narrowed = casts::integer(self.builder, result, left.ty, signed)?;
        let restored = casts::integer(self.builder, narrowed, wide, signed)?;
        let fault = self.builder.compare(Predicate::Ne, result, restored)?;
        Ok((narrowed, fault))
    }

    fn full_product(
        &mut self,
        signed: bool,
        left: Value,
        right: Value,
    ) -> Result<(Value, Value), &'static str> {
        let result = self.builder.binary(Binary::Mul, left, right)?;
        let (lhs, rhs, limit) = if signed {
            let shift = self.builder.integer(Type::I128, 127)?;
            let signs = self.builder.binary(Binary::Xor, left, right)?;
            let negative = self.builder.binary(Binary::Ushr, signs, shift)?;
            let maximum = self.builder.integer(Type::I128, (1_u128 << 127) - 1)?;
            let limit = self.builder.binary(Binary::Add, maximum, negative)?;
            (
                self.product_magnitude(left, shift)?,
                self.product_magnitude(right, shift)?,
                limit,
            )
        } else {
            (left, right, self.builder.integer(Type::I128, u128::MAX)?)
        };
        // rhs | (rhs == 0) is nonzero and equals rhs for every nonzero
        // magnitude. A zero product cannot exceed either unsigned limit.
        let zero = self.builder.integer(Type::I128, 0)?;
        let is_zero = self.builder.compare(Predicate::Eq, rhs, zero)?;
        let bit = self.builder.convert(Convert::Trunc, Type::I1, is_zero)?;
        let bit = self.builder.convert(Convert::Zext, Type::I128, bit)?;
        let denominator = self.builder.binary(Binary::Or, rhs, bit)?;
        let quotient = self.builder.binary(Binary::Udiv, limit, denominator)?;
        let fault = self.builder.compare(Predicate::Ult, quotient, lhs)?;
        Ok((result, fault))
    }

    fn product_magnitude(&mut self, value: Value, shift: Value) -> Result<Value, &'static str> {
        let sign = self.builder.binary(Binary::Shr, value, shift)?;
        let flipped = self.builder.binary(Binary::Xor, value, sign)?;
        self.builder.binary(Binary::Sub, flipped, sign)
    }
}
