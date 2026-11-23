//! Rust checked scalar operations, without C++ overflow metadata.
use super::casts;
mod products;
use crate::{
    lex::token::Punctuation,
    lowir::{
        builder::Builder,
        program::{BlockId, Terminator, Value},
        vocabulary::{Binary, Convert, Predicate, Type, Unary},
    },
    support::bindings::LangItem,
    syntax::ast,
    types::Primitive,
};

#[derive(Clone, Copy)]
pub(in crate::lower) enum ArithmeticCheck {
    Overflow(LangItem),
    Division {
        zero: LangItem,
        overflow: Option<LangItem>,
    },
    Shift(LangItem),
}

impl ArithmeticCheck {
    pub(in crate::lower) fn binary(operator: Punctuation, primitive: Primitive) -> Option<Self> {
        let (signed, _) = primitive.integer()?;
        Some(match operator {
            Punctuation::Plus => Self::Overflow(LangItem::PanicAddOverflow),
            Punctuation::Minus => Self::Overflow(LangItem::PanicSubOverflow),
            Punctuation::Star => Self::Overflow(LangItem::PanicMulOverflow),
            Punctuation::Slash => Self::Division {
                zero: LangItem::PanicDivZero,
                overflow: signed.then_some(LangItem::PanicDivOverflow),
            },
            Punctuation::Percent => Self::Division {
                zero: LangItem::PanicRemZero,
                overflow: signed.then_some(LangItem::PanicRemOverflow),
            },
            Punctuation::ShiftLeft => Self::Shift(LangItem::PanicShlOverflow),
            Punctuation::ShiftRight => Self::Shift(LangItem::PanicShrOverflow),
            _ => return None,
        })
    }

    pub(in crate::lower) const fn roles(self) -> [Option<LangItem>; 2] {
        match self {
            Self::Overflow(cause) | Self::Shift(cause) => [Some(cause), None],
            Self::Division { zero, overflow } => [Some(zero), overflow],
        }
    }
}

pub(in crate::lower) fn negation(primitive: Primitive, literal: bool) -> Option<u16> {
    let (signed, bits) = primitive.integer()?;
    (signed && !literal).then_some(bits)
}

pub(super) type GuardBranch<'a> =
    dyn FnMut(&mut Builder, Value, LangItem) -> Result<(), &'static str> + 'a;

pub(super) struct Arithmetic<'a> {
    pub builder: &'a mut Builder,
    pub branch: &'a mut GuardBranch<'a>,
}
#[cfg(test)]
const _: [(); 24] = [(); size_of::<Arithmetic<'static>>()];

pub(super) fn abort_guard(
    builder: &mut Builder,
    panic: &mut Option<BlockId>,
    fault: Value,
) -> Result<(), &'static str> {
    let panic = if let Some(block) = *panic {
        block
    } else {
        let block = builder.block()?;
        *panic = Some(block);
        block
    };
    let next = guard_edge(builder, fault, panic)?;
    builder.enter(next)
}

pub(super) fn guard_edge(
    builder: &mut Builder,
    fault: Value,
    panic: BlockId,
) -> Result<BlockId, &'static str> {
    let next = builder.block()?;
    builder.terminate(Terminator::Branch {
        condition: fault.operand,
        yes: panic,
        no: next,
    })?;
    Ok(next)
}

impl Arithmetic<'_> {
    pub fn guard(&mut self, fault: Value, cause: LangItem) -> Result<(), &'static str> {
        (self.branch)(self.builder, fault, cause)
    }
    pub fn unary(
        &mut self,
        operator: ast::Unary,
        primitive: Primitive,
        value: Value,
        literal: bool,
    ) -> Result<Value, &'static str> {
        if matches!(primitive, Primitive::F16 | Primitive::F128) {
            return Err("wide floating runtime arithmetic is not implemented");
        }
        let op = match operator {
            ast::Unary::Not if primitive == Primitive::Bool => Unary::Not,
            ast::Unary::Not => Unary::BitNot,
            ast::Unary::Negate => {
                if let Some(bits) = negation(primitive, literal) {
                    let minimum = self.builder.integer(value.ty, 1_u128 << (bits - 1))?;
                    let fault = self.builder.compare(Predicate::Eq, value, minimum)?;
                    self.guard(fault, LangItem::PanicNegOverflow)?;
                }
                Unary::Neg
            }
            _ => return Err("missing selected scalar unary operation"),
        };
        self.builder.unary(op, value)
    }
    pub fn binary(
        &mut self,
        operator: Punctuation,
        primitive: Primitive,
        left: Value,
        right: Value,
    ) -> Result<Value, &'static str> {
        if matches!(primitive, Primitive::F16 | Primitive::F128) {
            return Err("wide floating runtime arithmetic is not implemented");
        }
        let signed = primitive.integer().is_some_and(|(signed, _)| signed);
        let ordered = signed || primitive.is_float();
        let predicate = match operator {
            Punctuation::EqualEqual => Some(Predicate::Eq),
            Punctuation::BangEqual => Some(Predicate::Ne),
            Punctuation::Less => Some(if ordered {
                Predicate::Lt
            } else {
                Predicate::Ult
            }),
            Punctuation::LessEqual => Some(if ordered {
                Predicate::Le
            } else {
                Predicate::Ule
            }),
            Punctuation::Greater => Some(if ordered {
                Predicate::Gt
            } else {
                Predicate::Ugt
            }),
            Punctuation::GreaterEqual => Some(if ordered {
                Predicate::Ge
            } else {
                Predicate::Uge
            }),
            _ => None,
        };
        if let Some(predicate) = predicate {
            let value = self.builder.compare(predicate, left, right)?;
            return self.builder.convert(Convert::Trunc, Type::I1, value);
        }
        let op = match operator {
            Punctuation::Plus => Binary::Add,
            Punctuation::Minus => Binary::Sub,
            Punctuation::Star => Binary::Mul,
            Punctuation::Slash => {
                if ordered {
                    Binary::Div
                } else {
                    Binary::Udiv
                }
            }
            Punctuation::Percent => {
                if ordered {
                    Binary::Mod
                } else {
                    Binary::Umod
                }
            }
            Punctuation::And => Binary::And,
            Punctuation::Or => Binary::Or,
            Punctuation::Caret => Binary::Xor,
            Punctuation::ShiftLeft => Binary::Shl,
            Punctuation::ShiftRight => {
                if signed {
                    Binary::Shr
                } else {
                    Binary::Ushr
                }
            }
            _ => return Err("missing selected scalar binary operation"),
        };
        if primitive.is_float() {
            if matches!(op, Binary::Mod) {
                return Err("floating remainder needs its scalar helper");
            }
            return self.builder.binary(op, left, right);
        }
        self.integer_binary(
            op,
            ArithmeticCheck::binary(operator, primitive),
            signed,
            left,
            right,
        )
    }
    fn integer_binary(
        &mut self,
        op: Binary,
        check: Option<ArithmeticCheck>,
        signed: bool,
        left: Value,
        right: Value,
    ) -> Result<Value, &'static str> {
        match check {
            Some(ArithmeticCheck::Overflow(cause)) => self.checked(op, cause, signed, left, right),
            Some(ArithmeticCheck::Division { zero, overflow }) => {
                let divisor = self.builder.integer(right.ty, 0)?;
                let fault = self.builder.compare(Predicate::Eq, right, divisor)?;
                self.guard(fault, zero)?;
                if let Some(overflow) = overflow {
                    self.division_overflow(left, right, overflow)?;
                }
                self.builder.binary(op, left, right)
            }
            Some(ArithmeticCheck::Shift(cause)) => {
                let limit = self.builder.integer(right.ty, u128::from(left.ty.bits()))?;
                let fault = self.builder.compare(Predicate::Uge, right, limit)?;
                self.guard(fault, cause)?;
                let right = casts::integer(self.builder, right, left.ty, false)?;
                self.builder.binary(op, left, right)
            }
            None => self.builder.binary(op, left, right),
        }
    }
    fn division_overflow(
        &mut self,
        left: Value,
        right: Value,
        cause: LangItem,
    ) -> Result<(), &'static str> {
        let minimum = self
            .builder
            .integer(left.ty, 1_u128 << (left.ty.bits() - 1))?;
        let minus_one = self.builder.integer(right.ty, u128::MAX)?;
        let lhs = self.builder.compare(Predicate::Eq, left, minimum)?;
        let rhs = self.builder.compare(Predicate::Eq, right, minus_one)?;
        let fault = self.builder.binary(Binary::And, lhs, rhs)?;
        self.guard(fault, cause)
    }
    fn checked(
        &mut self,
        op: Binary,
        cause: LangItem,
        signed: bool,
        left: Value,
        right: Value,
    ) -> Result<Value, &'static str> {
        // Same-width sum/difference checks need two operations; widening needs
        // five and makes 64-bit operands 128-bit. Products keep one wide multiply.
        if matches!(op, Binary::Add | Binary::Sub) || left.ty == Type::I128 {
            return self.wide(op, cause, signed, left, right);
        }
        let (result, fault) = self.multiply_overflow(signed, left, right)?;
        self.guard(fault, cause)?;
        Ok(result)
    }
    fn wide(
        &mut self,
        op: Binary,
        cause: LangItem,
        signed: bool,
        left: Value,
        right: Value,
    ) -> Result<Value, &'static str> {
        let result = self.builder.binary(op, left, right)?;
        if matches!(op, Binary::Mul) {
            if signed {
                self.division_overflow(left, right, cause)?;
            }
            let zero = self.builder.integer(right.ty, 0)?;
            let nonzero = self.builder.compare(Predicate::Ne, right, zero)?;
            let check = self.builder.block()?;
            let done = self.builder.block()?;
            self.builder.terminate(Terminator::Branch {
                condition: nonzero.operand,
                yes: check,
                no: done,
            })?;
            self.builder.enter(check)?;
            // Exclude the reverse MIN * -1 pair before using signed division.
            if signed {
                self.division_overflow(right, left, cause)?;
            }
            let quotient = self.builder.binary(
                if signed { Binary::Div } else { Binary::Udiv },
                result,
                right,
            )?;
            let fault = self.builder.compare(Predicate::Ne, quotient, left)?;
            self.guard(fault, cause)?;
            self.builder.terminate(Terminator::Jump(done))?;
            self.builder.enter(done)?;
            return Ok(result);
        }
        let fault = self.overflow_flag(op, signed, left, right, result)?;
        self.guard(fault, cause)?;
        Ok(result)
    }
    /// Same target-width result/overflow kernel as ordinary checked arithmetic.
    /// The flag is the backend's i64 predicate carrier, suitable for branch guards.
    /// Rust bool values must truncate it to I1 before storage or value publication.
    pub(super) fn subtract_overflow(
        &mut self,
        signed: bool,
        left: Value,
        right: Value,
    ) -> Result<(Value, Value), &'static str> {
        let result = self.builder.binary(Binary::Sub, left, right)?;
        let overflow = self.overflow_flag(Binary::Sub, signed, left, right, result)?;
        Ok((result, overflow))
    }
    /// The wrapped sum and its flag, in the carrier `subtract_overflow` uses.
    pub(super) fn add_overflow(
        &mut self,
        signed: bool,
        left: Value,
        right: Value,
    ) -> Result<(Value, Value), &'static str> {
        let result = self.builder.binary(Binary::Add, left, right)?;
        let overflow = self.overflow_flag(Binary::Add, signed, left, right, result)?;
        Ok((result, overflow))
    }
    /// Select the target-width bound when the addition or subtraction overflows.
    pub(super) fn saturate(
        &mut self,
        subtract: bool,
        signed: bool,
        left: Value,
        wrapped: Value,
        overflow: Value,
    ) -> Result<Value, &'static str> {
        let ty = wrapped.ty;
        let zero = self.builder.integer(ty, 0)?;
        let flag = casts::integer(self.builder, overflow, ty, false)?;
        let mask = self.builder.binary(Binary::Sub, zero, flag)?;
        let bound = if signed {
            let negative = self.builder.compare(Predicate::Lt, left, zero)?;
            let negative = casts::integer(self.builder, negative, ty, false)?;
            let maximum = self.builder.integer(ty, (1_u128 << (ty.bits() - 1)) - 1)?;
            // Signed overflow heads toward the sign of the left operand.
            self.builder.binary(Binary::Add, maximum, negative)?
        } else if subtract {
            zero
        } else {
            self.builder.integer(ty, u128::MAX)?
        };
        let differing = self.builder.binary(Binary::Xor, wrapped, bound)?;
        let replaced = self.builder.binary(Binary::And, differing, mask)?;
        self.builder.binary(Binary::Xor, wrapped, replaced)
    }
    fn overflow_flag(
        &mut self,
        op: Binary,
        signed: bool,
        left: Value,
        right: Value,
        result: Value,
    ) -> Result<Value, &'static str> {
        if signed {
            let a = self.builder.binary(Binary::Xor, left, result)?;
            let b = self.builder.binary(
                Binary::Xor,
                if matches!(op, Binary::Add) {
                    result
                } else {
                    left
                },
                right,
            )?;
            let sign = self.builder.binary(Binary::And, a, b)?;
            let zero = self.builder.integer(sign.ty, 0)?;
            self.builder.compare(Predicate::Lt, sign, zero)
        } else if matches!(op, Binary::Add) {
            self.builder.compare(Predicate::Ult, result, left)
        } else {
            self.builder.compare(Predicate::Ult, left, right)
        }
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/guard_emitters.rs"]
mod tests;
