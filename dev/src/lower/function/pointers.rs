//! Raw pointer comparisons inspect both the data address and selected metadata.
use super::{Lower, Selected};
use crate::{
    body::ir::ValueId,
    lex::token::Punctuation as Op,
    lowir::{
        program::Value,
        vocabulary::{Binary, Convert, Predicate, Type},
    },
    sema::model::Cast,
};

impl Lower<'_, '_, '_> {
    pub(super) fn pointer_cast(
        &mut self,
        kind: Cast,
        operand: ValueId,
        target: Type,
    ) -> Result<Value, &'static str> {
        if kind == Cast::PointerThin {
            return self.pointer_parts(operand).map(|(data, _)| data);
        }
        let value = self.value(operand)?.runtime()?;
        if kind == Cast::PointerToInteger {
            let value = self.builder.copy(Type::I64, value)?;
            return super::super::casts::integer(&mut self.builder, value, target, false);
        }
        if kind == Cast::IntegerToPointer {
            let signed = self
                .primitive(self.body.values[operand.0 as usize].ty)?
                .integer()
                .is_some_and(|(signed, _)| signed);
            let value = super::super::casts::integer(&mut self.builder, value, Type::I64, signed)?;
            return self.builder.copy(Type::Ptr, value);
        }
        Err("invalid selected address cast")
    }
    pub(super) fn pointer_comparison(
        &mut self,
        operator: Op,
        left: ValueId,
        right: ValueId,
    ) -> Result<Value, &'static str> {
        let (left, left_length) = self.pointer_parts(left)?;
        let (right, right_length) = self.pointer_parts(right)?;
        let predicate = match operator {
            Op::EqualEqual => Predicate::Eq,
            Op::BangEqual => Predicate::Ne,
            Op::Less => Predicate::Ult,
            Op::LessEqual => Predicate::Ule,
            Op::Greater => Predicate::Ugt,
            Op::GreaterEqual => Predicate::Uge,
            _ => return Err("invalid selected pointer comparison"),
        };
        let result = match (left_length, right_length) {
            (None, None) => self.builder.compare(predicate, left, right)?,
            (Some(a), Some(b)) => {
                let metadata = self.builder.compare(predicate, a, b)?;
                if matches!(operator, Op::EqualEqual | Op::BangEqual) {
                    let data = self.builder.compare(predicate, left, right)?;
                    self.builder.binary(
                        if operator == Op::EqualEqual {
                            Binary::And
                        } else {
                            Binary::Or
                        },
                        data,
                        metadata,
                    )?
                } else {
                    // Ordering is lexicographic: length breaks equal-address ties.
                    let equal = self.builder.compare(Predicate::Eq, left, right)?;
                    let tied = self.builder.binary(Binary::And, equal, metadata)?;
                    let strict = if matches!(operator, Op::Less | Op::LessEqual) {
                        Predicate::Ult
                    } else {
                        Predicate::Ugt
                    };
                    let data = self.builder.compare(strict, left, right)?;
                    self.builder.binary(Binary::Or, data, tied)?
                }
            }
            _ => return Err("pointer comparison changes metadata representation"),
        };
        self.builder.convert(Convert::Trunc, Type::I1, result)
    }
    fn pointer_parts(&mut self, operand: ValueId) -> Result<(Value, Option<Value>), &'static str> {
        match self.value(operand)? {
            Selected::Runtime(value) if value.ty == Type::Ptr => Ok((value, None)),
            Selected::Object(address) => {
                let data = self.builder.load(Type::Ptr, address.operand)?;
                let tail = self.offset(address, 8)?;
                let length = self.builder.load(Type::I64, tail.operand)?;
                Ok((data, Some(length)))
            }
            _ => Err("pointer value has an invalid representation"),
        }
    }
}
