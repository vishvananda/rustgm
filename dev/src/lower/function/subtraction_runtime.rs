//! Actual scalar subtraction returns the existing tuple representation and flag.
use super::{Lower, Selected, ir};
use crate::{
    abi::layout::Shape,
    lex::token::Punctuation,
    lowir::{
        program::Value,
        vocabulary::{Binary, Convert, Type},
    },
    sema::compiler_items::SubtractionSelection,
    types::{Kind, TypeId, instance::Instance},
};
impl Lower<'_, '_, '_> {
    pub(super) fn source_subtraction(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = *self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("source subtraction call disappeared")?;
        let ir::Rvalue::Call {
            signature,
            callee,
            arguments,
        } = value.kind
        else {
            return Ok(None);
        };
        let ty = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("source subtraction callee disappeared")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(None);
        }
        let instance = Instance::new(self.context.types, ty)?;
        let Some(selected) = self
            .context
            .constants
            .source_subtraction(instance, self.body.source)?
        else {
            return Ok(None);
        };
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let [left, right] = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source subtraction arguments disappeared")?
        else {
            return Err("source subtraction requires two evaluated arguments");
        };
        let (left, right) = (*left, *right);
        if !selected.opened_signature_matches(self.context.types, signature)
            || value.ty != selected.output
            || [(left, selected.input), (right, selected.rhs)]
                .into_iter()
                .any(|(id, ty)| {
                    self.body
                        .values
                        .get(id.0 as usize)
                        .is_none_or(|v| v.ty != ty)
                })
            || !matches!(self.value(callee)?, Selected::Item(actual) if actual==self.body.callable_target(instance))
        {
            return Err("source subtraction differs from its actual checked direct call");
        }
        let (left, right) = (self.value(left)?.runtime()?, self.value(right)?.runtime()?);
        self.lower_source_arithmetic(&selected, left, right)
            .map(Some)
    }
    fn lower_source_arithmetic(
        &mut self,
        selected: &SubtractionSelection,
        left: Value,
        right: Value,
    ) -> Result<Selected, &'static str> {
        if let Some(operator) = crate::sema::compiler_items::wrapping_operator(selected.kind) {
            return self.scalar_binary(operator, left, right, false);
        }
        let Some(Kind::Primitive(primitive)) = self.context.types.kind(selected.input) else {
            return Err("source subtraction lost its integer input");
        };
        let (signed, _) = primitive
            .integer()
            .ok_or("source subtraction requires an integer")?;
        if let Some(operator) = crate::sema::compiler_items::unchecked_shift(selected.kind) {
            let binary = match operator {
                Punctuation::ShiftLeft => Binary::Shl,
                _ if signed => Binary::Shr,
                _ => Binary::Ushr,
            };
            let right = super::casts::integer(&mut self.builder, right, left.ty, false)?;
            return self
                .builder
                .binary(binary, left, right)
                .map(Selected::Runtime);
        }
        if let Some(operator) = crate::sema::compiler_items::unchecked_operator(selected.kind) {
            return self.scalar_binary(operator, left, right, signed);
        }
        let saturation = crate::sema::compiler_items::saturating_operator(selected.kind);
        let operator =
            saturation.or_else(|| crate::sema::compiler_items::overflowing_operator(selected.kind));
        let (result, overflow) = self.arithmetic(|arithmetic| match operator {
            Some(Punctuation::Minus) => arithmetic.subtract_overflow(signed, left, right),
            Some(Punctuation::Star) => arithmetic.multiply_overflow(signed, left, right),
            Some(Punctuation::Plus) => arithmetic.add_overflow(signed, left, right),
            _ => Err("source arithmetic lost its selected overflow operation"),
        })?;
        if let Some(operator) = saturation {
            return self
                .arithmetic(|arithmetic| {
                    arithmetic.saturate(
                        operator == Punctuation::Minus,
                        signed,
                        left,
                        result,
                        overflow,
                    )
                })
                .map(Selected::Runtime);
        }
        self.store_subtraction_tuple(result, overflow, selected.input, selected.output)
    }
    /// Publish the real checked result and flag using its selected tuple layout.
    pub(super) fn store_subtraction_tuple(
        &mut self,
        result: crate::lowir::program::Value,
        overflow: crate::lowir::program::Value,
        input: TypeId,
        output: TypeId,
    ) -> Result<Selected, &'static str> {
        let layout = self.context.layout(output)?;
        let Shape::Aggregate { .. } = layout.shape else {
            return Err("source subtraction tuple lacks its real layout");
        };
        let (difference, flag) = {
            let [variant] = self.context.layouts.variants(&layout) else {
                return Err("source subtraction tuple requires its single field group");
            };
            let &[difference, flag] = self.context.layouts.offsets(variant) else {
                return Err("source subtraction tuple requires two real fields");
            };
            (difference, flag)
        };
        let address = self
            .allocate(output)?
            .ok_or("source subtraction tuple has no storage")?;
        let destination = self.offset(address, difference)?;
        self.write(Some(destination), Selected::Runtime(result), input)?;
        let destination = self.offset(address, flag)?;
        let overflow = self.builder.convert(Convert::Trunc, Type::I1, overflow)?;
        self.write(
            Some(destination),
            Selected::Runtime(overflow),
            TypeId::primitive(crate::types::Primitive::Bool),
        )?;
        Ok(Selected::Object(address))
    }
    /// Wrapping and valid unsafe arithmetic use the existing raw scalar operation.
    /// The builder validates its types; checked operations retain their own path.
    pub(super) fn scalar_binary(
        &mut self,
        operator: crate::lex::token::Punctuation,
        left: crate::lowir::program::Value,
        right: crate::lowir::program::Value,
        signed: bool,
    ) -> Result<Selected, &'static str> {
        let binary = match operator {
            Punctuation::Plus => Binary::Add,
            Punctuation::Minus => Binary::Sub,
            Punctuation::Star => Binary::Mul,
            Punctuation::Slash if signed => Binary::Div,
            Punctuation::Slash => Binary::Udiv,
            Punctuation::Percent if signed => Binary::Mod,
            Punctuation::Percent => Binary::Umod,
            _ => return Err("raw arithmetic descriptor has an invalid operation"),
        };
        if matches!(binary, Binary::Div | Binary::Mod) && left.ty.bits() < 64 {
            // Canonicalize narrow signed operands, including values entering
            // through a C parameter whose upper register bits are unspecified.
            let ty = left.ty;
            let left = super::casts::integer(&mut self.builder, left, Type::I64, true)?;
            let right = super::casts::integer(&mut self.builder, right, Type::I64, true)?;
            let result = self.builder.binary(binary, left, right)?;
            return super::casts::integer(&mut self.builder, result, ty, true)
                .map(Selected::Runtime);
        }
        self.builder
            .binary(binary, left, right)
            .map(Selected::Runtime)
    }
}
