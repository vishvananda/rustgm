//! Selected value operations reuse the constant engine and MIR-compatible eligibility.
use super::{Engine, Error, Key, Ready, Type};
use crate::{
    body::ir::{Rvalue, ValueId, ValueList},
    lex::token::Punctuation as Op,
    resolve::model::DefId,
    sema::{
        members::VariantId,
        model::Scalar,
        values::{Constant, Overflow},
    },
    syntax::ast::Unary,
};
impl Engine<'_, '_> {
    pub(super) fn constructor(&self, callee: ValueId) -> Result<Option<(DefId, VariantId)>, Error> {
        let Some(Type::FunctionItem { item, .. }) =
            self.evaluator.facts().types.kind(self.value(callee)?.ty)
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        Ok(self
            .evaluator
            .facts()
            .members
            .constructors
            .get(&definition)
            .map(|&variant| (definition, variant)))
    }
    pub(super) fn empty_repeat(&self, id: ValueId) -> Result<bool, Error> {
        match self.evaluator.facts().types.kind(self.value(id)?.ty) {
            Some(Type::Array { length, .. }) => Ok(length == 0),
            _ => Err("promotion repeat is not an array".into()),
        }
    }
    fn fields(
        &mut self,
        fields: ValueList,
        variant: Option<VariantId>,
    ) -> Result<Option<Constant>, Error> {
        let ids = self
            .body
            .arguments
            .get(fields.start as usize..fields.end as usize)
            .ok_or("invalid promotion field range")?;
        self.charge(ids.len())?;
        let mut fields = Vec::with_capacity(ids.len());
        for &id in ids {
            let Some(value) = self.ready(Key::Value(id))? else {
                return Ok(None);
            };
            fields.push(value.value);
        }
        self.evaluator
            .fields(&fields, variant)
            .map(Some)
            .map_err(Error::from)
    }
    pub(super) fn finish_value(&mut self, id: ValueId) -> Result<Option<Ready>, Error> {
        let value = self.value(id)?;
        let (ty, kind) = (value.ty, value.kind);
        let mut immediate = false;
        let mut constant_assignment = false;
        let value = match kind {
            Rvalue::Scalar(id) => {
                immediate = true;
                Some(Constant::scalar(
                    *self
                        .body
                        .scalars
                        .get(id.0 as usize)
                        .ok_or("missing promotion scalar")?,
                ))
            }
            Rvalue::Function(definition) => {
                immediate = true;
                Some(Constant::function(definition))
            }
            Rvalue::Constant(definition) => {
                immediate = true;
                Some(
                    self.evaluator
                        .definition(definition)
                        .map_err(Error::Constant)?,
                )
            }
            Rvalue::Unit => {
                immediate = true;
                Some(self.evaluator.fields(&[], None)?)
            }
            Rvalue::Aggregate { variant, fields } => self.fields(fields, variant)?,
            Rvalue::Repeat { .. } if self.empty_repeat(id)? => {
                Some(self.evaluator.fields(&[], None)?)
            }
            Rvalue::Repeat { value } => {
                let Some(value) = self.ready(Key::Value(value))? else {
                    return Ok(None);
                };
                let Some(Type::Array { length, .. }) = self.evaluator.facts().types.kind(ty) else {
                    return Err("promotion repeat is not an array".into());
                };
                Some(self.evaluator.repeat(value.value, length)?)
            }
            Rvalue::Read(place) => self.place_value(place)?.map(|v| v.value),
            Rvalue::Borrow { place } => self.borrow_value(place, ty)?,
            Rvalue::Constrain(operand) => return self.ready(Key::Value(operand)),
            Rvalue::Reify(operand) => self.ready(Key::Value(operand))?.map(|v| v.value),
            Rvalue::Unary {
                operator,
                operand,
                literal,
            } => {
                let Some(value) = self.ready(Key::Value(operand))? else {
                    return Ok(None);
                };
                immediate = literal && operator == Unary::Negate;
                Some(self.evaluator.unary(
                    operator,
                    self.value(operand)?.ty,
                    value.value,
                    literal,
                    Overflow::Wrapping,
                )?)
            }
            Rvalue::Binary {
                operator,
                left,
                right,
            } => self.binary(id, operator, left, right)?,
            Rvalue::Cast { kind, operand } => {
                let Some(value) = self.ready(Key::Value(operand))? else {
                    return Ok(None);
                };
                let source = self.value(operand)?.ty;
                constant_assignment = source == ty && value.constant_assignment;
                Some(self.evaluator.cast(kind, source, ty, value.value)?)
            }
            Rvalue::Call {
                callee, arguments, ..
            } => self.call(callee, arguments)?,
            Rvalue::IsVariant { .. } | Rvalue::Validate(_) => None,
        };
        Ok(value.map(|value| Ready {
            value,
            immediate,
            constant_assignment: constant_assignment || immediate,
        }))
    }
    fn call(&mut self, callee: ValueId, arguments: ValueList) -> Result<Option<Constant>, Error> {
        let Some((definition, variant)) = self.constructor(callee)? else {
            return Ok(None);
        };
        let Some(value) = self.ready(Key::Value(callee))? else {
            return Ok(None);
        };
        if value.value.as_function() != Some(definition) {
            return Err("promotion callee differs from selected constructor".into());
        }
        self.fields(arguments, Some(variant))
    }
    fn binary(
        &mut self,
        source: ValueId,
        op: Op,
        left: ValueId,
        right: ValueId,
    ) -> Result<Option<Constant>, Error> {
        let (Some(lhs), Some(rhs)) = (
            self.ready(Key::Value(left))?,
            self.ready(Key::Value(right))?,
        ) else {
            return Ok(None);
        };
        let ty = self.value(left)?.ty;
        if matches!(op, Op::Slash | Op::Percent)
            && let Some(Type::Primitive(primitive)) = self.evaluator.facts().types.kind(ty)
            && let Some((signed, width)) = primitive.integer()
        {
            // rustc requires immediate operands here, not arbitrary foldable DAGs.
            let Scalar::Integer(divisor) = rhs.value.as_scalar()? else {
                return Err("promotion divisor is not an integer".into());
            };
            if !rhs.immediate || divisor == 0 || !self.immediate_operand(source, false)? {
                return Ok(None);
            }
            if signed && divisor == u128::MAX >> (128 - width) {
                let Scalar::Integer(dividend) = lhs.value.as_scalar()? else {
                    return Err("promotion dividend is not an integer".into());
                };
                if !lhs.immediate
                    || dividend == 1_u128 << (width - 1)
                    || !self.immediate_operand(source, true)?
                {
                    return Ok(None);
                }
            }
        }
        self.evaluator
            .binary(op, ty, lhs.value, rhs.value, Overflow::Wrapping)
            .map(Some)
            .map_err(Error::from)
    }
}
