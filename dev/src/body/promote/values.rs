//! Selected value operations reuse the constant engine and MIR-compatible eligibility.
use super::{Engine, Error, Evaluated, Key, Ready, Type};
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
            Some(Type::ArrayTerm { .. }) => Ok(false),
            _ => Err("promotion repeat is not an array".into()),
        }
    }
    fn fields(
        &mut self,
        fields: ValueList,
        variant: Option<VariantId>,
    ) -> Result<Option<Evaluated>, Error> {
        let ids = self
            .body
            .arguments
            .get(fields.start as usize..fields.end as usize)
            .ok_or("invalid promotion field range")?;
        self.charge(ids.len())?;
        let mut fields = Vec::with_capacity(ids.len());
        let mut deferred = false;
        for &id in ids {
            let Some(value) = self.ready(Key::Value(id))? else {
                return Ok(None);
            };
            match value.value {
                Evaluated::Constant(value) if !deferred => fields.push(value),
                Evaluated::Deferred => deferred = true,
                Evaluated::Constant(_) => {}
            }
        }
        if deferred {
            return Ok(Some(Evaluated::Deferred));
        }
        self.evaluator
            .fields(&fields, variant)
            .map(|value| Some(Evaluated::Constant(value)))
            .map_err(Error::from)
    }
    fn text_value(
        &self,
        id: crate::sema::text::Id,
        ty: crate::types::TypeId,
    ) -> Result<Constant, Error> {
        let Some(Type::Reference { referent, .. }) = self.evaluator.facts().types.kind(ty) else {
            return Err("text literal has no reference type".into());
        };
        Ok(
            if referent == crate::types::TypeId::primitive(crate::types::Primitive::Str) {
                Constant::string(id)
            } else {
                Constant::bytes(id)
            },
        )
    }
    fn immediate_value(
        &mut self,
        kind: Rvalue,
        ty: crate::types::TypeId,
    ) -> Result<Option<Evaluated>, Error> {
        let value = match kind {
            Rvalue::Text(id) => self.text_value(id, ty)?,
            Rvalue::Scalar(id) => Constant::scalar(
                *self
                    .body
                    .scalars
                    .get(id.0 as usize)
                    .ok_or("missing promotion scalar")?,
            ),
            Rvalue::Function(definition) => {
                let store = &self.evaluator.facts().types;
                let instance = crate::types::instance::Instance::new(store, ty)?;
                if !matches!(store.kind(ty), Some(Type::FunctionItem { item, .. }) if item.0 == definition.0)
                {
                    return Err("promotion function differs from selected item".into());
                }
                Constant::function(instance)
            }
            Rvalue::Constant(definition) => self
                .evaluator
                .definition(definition)
                .map_err(Error::Constant)?,
            Rvalue::AssociatedConstant(instance) => {
                if self.body.instance.is_none()
                    && match self.body.owner {
                        crate::body::ir::Owner::Function { definition, .. } => {
                            self.evaluator.facts().generics.has_parameters(definition)
                        }
                        crate::body::ir::Owner::Initializer { definition, .. } => {
                            self.evaluator
                                .facts()
                                .associated
                                .owner(definition)
                                .is_some()
                                || self.evaluator.facts().generics.has_parameters(definition)
                        }
                        crate::body::ir::Owner::AnonymousConstant
                        | crate::body::ir::Owner::Closure { .. } => false,
                    }
                {
                    return Ok(Some(Evaluated::Deferred));
                }
                self.evaluator
                    .associated(instance)
                    .map_err(Error::Constant)?
            }
            Rvalue::ConstArgument(id) => {
                return Ok(Some(
                    Constant::argument(&self.evaluator.facts().types, id)?
                        .map_or(Evaluated::Deferred, Evaluated::Constant),
                ));
            }
            Rvalue::Unit => self.evaluator.fields(&[], None)?,
            _ => return Ok(None),
        };
        Ok(Some(Evaluated::Constant(value)))
    }
    pub(super) fn finish_value(&mut self, id: ValueId) -> Result<Option<Ready>, Error> {
        let value = self.value(id)?;
        let (ty, kind) = (value.ty, value.kind);
        if let Some(value) = self.immediate_value(kind, ty)? {
            return Ok(Some(Ready {
                value,
                immediate: true,
                constant_assignment: true,
            }));
        }
        let mut immediate = false;
        let mut constant_assignment = false;
        let value = match kind {
            Rvalue::Aggregate { variant, fields } => self.fields(fields, variant)?,
            Rvalue::Repeat { .. } if self.empty_repeat(id)? => {
                Some(Evaluated::Constant(self.evaluator.fields(&[], None)?))
            }
            Rvalue::Repeat { value, .. } => self.repeat(ty, value)?,
            Rvalue::Read(place) => self.place_value(place)?.map(|v| v.value),
            Rvalue::Borrow { place, .. } => self.borrow_value(place, ty)?,
            Rvalue::Constrain(operand) => return self.ready(Key::Value(operand)),
            Rvalue::Unsize { value, conversion } => match conversion {
                crate::types::unsizing::Conversion::Slice(array) => self.unsize(value, array)?,
                crate::types::unsizing::Conversion::Object { concrete, target } => {
                    self.object_unsizing(value, concrete, target)?
                }
                crate::types::unsizing::Conversion::Upcast { target, .. } => {
                    self.object_upcasting(value, target)?
                }
            },
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
                Some(match value.value {
                    Evaluated::Constant(value) => Evaluated::Constant(self.evaluator.unary(
                        operator,
                        self.value(operand)?.ty,
                        value,
                        literal,
                        Overflow::Wrapping,
                    )?),
                    Evaluated::Deferred => Evaluated::Deferred,
                })
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
                Some(match value.value {
                    Evaluated::Constant(value) => {
                        Evaluated::Constant(self.evaluator.cast(kind, source, ty, value)?)
                    }
                    Evaluated::Deferred => Evaluated::Deferred,
                })
            }
            Rvalue::Call {
                callee, arguments, ..
            } => self.call(callee, arguments)?,
            Rvalue::Invoke { .. }
            | Rvalue::Intrinsic { .. }
            | Rvalue::Union { .. }
            | Rvalue::Address(_)
            | Rvalue::IsVariant { .. }
            | Rvalue::Length(_)
            | Rvalue::TextEquals { .. }
            | Rvalue::Validate(_) => None,
            Rvalue::Unit
            | Rvalue::Text(_)
            | Rvalue::Scalar(_)
            | Rvalue::Constant(_)
            | Rvalue::AssociatedConstant(_)
            | Rvalue::ConstArgument(_)
            | Rvalue::Function(_) => return Err("missing immediate promotion value".into()),
        };
        Ok(value.map(|value| Ready {
            value,
            immediate,
            constant_assignment: constant_assignment || immediate,
        }))
    }
    fn repeat(
        &mut self,
        ty: crate::types::TypeId,
        operand: ValueId,
    ) -> Result<Option<Evaluated>, Error> {
        let Some(value) = self.ready(Key::Value(operand))? else {
            return Ok(None);
        };
        Ok(Some(
            match (self.evaluator.facts().types.kind(ty), value.value) {
                (Some(Type::Array { length, .. }), Evaluated::Constant(value)) => {
                    Evaluated::Constant(self.evaluator.repeat(value, length)?)
                }
                (Some(Type::Array { .. } | Type::ArrayTerm { .. }), _) => Evaluated::Deferred,
                _ => return Err("promotion repeat is not an array".into()),
            },
        ))
    }
    fn unsize(
        &self,
        operand: ValueId,
        array: crate::types::TypeId,
    ) -> Result<Option<Evaluated>, Error> {
        let Some(value) = self.ready(Key::Value(operand))? else {
            return Ok(None);
        };
        Ok(Some(
            match (self.evaluator.facts().types.kind(array), value.value) {
                (Some(Type::Array { length, .. }), Evaluated::Constant(value)) => {
                    Evaluated::Constant(value.unsize(length)?)
                }
                (Some(Type::Array { .. } | Type::ArrayTerm { .. }), _) => Evaluated::Deferred,
                _ => return Err("selected promotion unsizing lacks an array".into()),
            },
        ))
    }
    fn object_unsizing(
        &self,
        operand: ValueId,
        concrete: crate::types::TypeId,
        target: crate::types::TypeId,
    ) -> Result<Option<Evaluated>, Error> {
        let Some(value) = self.ready(Key::Value(operand))? else {
            return Ok(None);
        };
        let types = &self.evaluator.facts().types;
        Ok(Some(match value.value {
            Evaluated::Constant(value)
                if types.contains_parameters(concrete) == Some(false)
                    && types.contains_parameters(target) == Some(false) =>
            {
                Evaluated::Constant(
                    value.with_metadata(crate::sema::values::Metadata::Object {
                        concrete,
                        target,
                    })?,
                )
            }
            _ => Evaluated::Deferred,
        }))
    }
    fn object_upcasting(
        &self,
        operand: ValueId,
        target: crate::types::TypeId,
    ) -> Result<Option<Evaluated>, Error> {
        let Some(value) = self.ready(Key::Value(operand))? else {
            return Ok(None);
        };
        Ok(Some(match value.value {
            Evaluated::Constant(value)
                if self.evaluator.facts().types.contains_parameters(target) == Some(false) =>
            {
                Evaluated::Constant(value.upcast(target)?)
            }
            _ => Evaluated::Deferred,
        }))
    }
    fn call(&mut self, callee: ValueId, arguments: ValueList) -> Result<Option<Evaluated>, Error> {
        let Some((definition, variant)) = self.constructor(callee)? else {
            return Ok(None);
        };
        let Some(value) = self.ready(Key::Value(callee))? else {
            return Ok(None);
        };
        let selected = value.value.constant().and_then(Constant::as_function);
        if !matches!(selected.and_then(|i| self.evaluator.facts().types.kind(i.ty())), Some(Type::FunctionItem { item, .. }) if item.0 == definition.0)
        {
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
    ) -> Result<Option<Evaluated>, Error> {
        let (Some(lhs), Some(rhs)) = (
            self.ready(Key::Value(left))?,
            self.ready(Key::Value(right))?,
        ) else {
            return Ok(None);
        };
        let ty = self.value(left)?.ty;
        if matches!(
            self.evaluator.facts().types.kind(ty),
            Some(Type::Pointer { .. })
        ) {
            // Addresses have no compile-time ordering or equality guarantee.
            return Ok(None);
        }
        if matches!(op, Op::Slash | Op::Percent)
            && let Some(Type::Primitive(primitive)) = self.evaluator.facts().types.kind(ty)
            && let Some((signed, width)) = primitive.integer()
        {
            // The divisor must be known nonzero; signed MIN / -1 must be impossible.
            // Immediate syntax is required independently of a foldable value DAG.
            let Some(divisor) = rhs.value.constant() else {
                return Ok(None);
            };
            let Scalar::Integer(divisor) = divisor.as_scalar()? else {
                return Err("promotion divisor is not an integer".into());
            };
            if !rhs.immediate || divisor == 0 || !self.immediate_operand(source, false)? {
                return Ok(None);
            }
            if signed && divisor == u128::MAX >> (128 - width) {
                let Some(dividend) = lhs.value.constant() else {
                    return Ok(None);
                };
                let Scalar::Integer(dividend) = dividend.as_scalar()? else {
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
        match (lhs.value, rhs.value) {
            (Evaluated::Constant(lhs), Evaluated::Constant(rhs)) => self
                .evaluator
                .binary(op, ty, lhs, rhs, Overflow::Wrapping)
                .map(|value| Some(Evaluated::Constant(value)))
                .map_err(Error::from),
            _ => Ok(Some(Evaluated::Deferred)),
        }
    }
}
