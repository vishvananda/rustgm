//! Rust checked scalar operations, without C++ overflow metadata.
use super::casts;
use crate::{
    lex::token::Punctuation,
    lowir::{
        builder::Builder,
        program::{BlockId, Terminator, Value},
        vocabulary::{Binary, Convert, Predicate, Type, Unary},
    },
    syntax::ast,
    types::Primitive,
};

pub(super) struct Arithmetic<'a> {
    pub builder: &'a mut Builder,
    pub panic: &'a mut Option<BlockId>,
}
impl Arithmetic<'_> {
    pub fn guard(&mut self, fault: Value) -> Result<(), &'static str> {
        let panic = if let Some(block) = *self.panic {
            block
        } else {
            let block = self.builder.block()?;
            *self.panic = Some(block);
            block
        };
        let next = self.builder.block()?;
        self.builder.terminate(Terminator::Branch {
            condition: fault.operand,
            yes: panic,
            no: next,
        })?;
        self.builder.enter(next)
    }
    pub fn unary(
        &mut self,
        operator: ast::Unary,
        primitive: Primitive,
        value: Value,
        literal: bool,
    ) -> Result<Value, &'static str> {
        let op = match operator {
            ast::Unary::Not if primitive == Primitive::Bool => Unary::Not,
            ast::Unary::Not => Unary::BitNot,
            ast::Unary::Negate => {
                if let Some((true, bits)) = primitive.integer()
                    && !literal
                {
                    let minimum = self.builder.integer(value.ty, 1_u128 << (bits - 1))?;
                    let fault = self.builder.compare(Predicate::Eq, value, minimum)?;
                    self.guard(fault)?;
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
        match op {
            Binary::Add | Binary::Sub | Binary::Mul => self.checked(op, signed, left, right),
            Binary::Div | Binary::Udiv | Binary::Mod | Binary::Umod => {
                let zero = self.builder.integer(right.ty, 0)?;
                let fault = self.builder.compare(Predicate::Eq, right, zero)?;
                self.guard(fault)?;
                if signed {
                    self.division_overflow(left, right)?;
                }
                self.builder.binary(op, left, right)
            }
            Binary::Shl | Binary::Shr | Binary::Ushr => {
                let limit = self.builder.integer(right.ty, u128::from(left.ty.bits()))?;
                let fault = self.builder.compare(Predicate::Uge, right, limit)?;
                self.guard(fault)?;
                let right = casts::integer(self.builder, right, left.ty, false)?;
                self.builder.binary(op, left, right)
            }
            _ => self.builder.binary(op, left, right),
        }
    }
    fn division_overflow(&mut self, left: Value, right: Value) -> Result<(), &'static str> {
        let minimum = self
            .builder
            .integer(left.ty, 1_u128 << (left.ty.bits() - 1))?;
        let minus_one = self.builder.integer(right.ty, u128::MAX)?;
        let lhs = self.builder.compare(Predicate::Eq, left, minimum)?;
        let rhs = self.builder.compare(Predicate::Eq, right, minus_one)?;
        let fault = self.builder.binary(Binary::And, lhs, rhs)?;
        self.guard(fault)
    }
    fn checked(
        &mut self,
        op: Binary,
        signed: bool,
        left: Value,
        right: Value,
    ) -> Result<Value, &'static str> {
        // Same-width sum/difference checks need two operations; widening needs
        // five and makes 64-bit operands 128-bit. Products keep one wide multiply.
        if matches!(op, Binary::Add | Binary::Sub) {
            return self.wide(op, signed, left, right);
        }
        let wide = match left.ty.bits() {
            8 => Type::I16,
            16 => Type::I32,
            32 => Type::I64,
            64 => Type::I128,
            128 => return self.wide(op, signed, left, right),
            _ => return Err("invalid checked arithmetic width"),
        };
        let lhs = casts::integer(self.builder, left, wide, signed)?;
        let rhs = casts::integer(self.builder, right, wide, signed)?;
        let result = self.builder.binary(op, lhs, rhs)?;
        let narrowed = casts::integer(self.builder, result, left.ty, signed)?;
        let restored = casts::integer(self.builder, narrowed, wide, signed)?;
        let fault = self.builder.compare(Predicate::Ne, result, restored)?;
        self.guard(fault)?;
        Ok(narrowed)
    }
    fn wide(
        &mut self,
        op: Binary,
        signed: bool,
        left: Value,
        right: Value,
    ) -> Result<Value, &'static str> {
        let result = self.builder.binary(op, left, right)?;
        if matches!(op, Binary::Mul) {
            if signed {
                self.division_overflow(left, right)?;
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
                self.division_overflow(right, left)?;
            }
            let quotient = self.builder.binary(
                if signed { Binary::Div } else { Binary::Udiv },
                result,
                right,
            )?;
            let fault = self.builder.compare(Predicate::Ne, quotient, left)?;
            self.guard(fault)?;
            self.builder.terminate(Terminator::Jump(done))?;
            self.builder.enter(done)?;
            return Ok(result);
        }
        let fault = if signed {
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
            self.builder.compare(Predicate::Lt, sign, zero)?
        } else if matches!(op, Binary::Add) {
            self.builder.compare(Predicate::Ult, result, left)?
        } else {
            self.builder.compare(Predicate::Ult, left, right)?
        };
        self.guard(fault)?;
        Ok(result)
    }
}
