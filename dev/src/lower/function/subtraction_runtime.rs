//! Actual scalar subtraction returns the existing tuple representation and flag.
use super::{Arithmetic, Lower, Selected, ir};
use crate::{
    abi::layout::Shape,
    lowir::vocabulary::{Binary, Convert, Type},
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
            || [left, right].into_iter().any(|id| {
                self.body
                    .values
                    .get(id.0 as usize)
                    .is_none_or(|v| v.ty != selected.input)
            })
            || !matches!(self.value(callee)?, Selected::Item(actual) if actual==self.body.callable_target(instance))
        {
            return Err("source subtraction differs from its actual checked direct call");
        }
        let (left, right) = (self.value(left)?.runtime()?, self.value(right)?.runtime()?);
        if let Some(operator) = crate::sema::compiler_items::wrapping_operator(selected.kind) {
            return self.modular_binary(operator, left, right).map(Some);
        }
        let Some(Kind::Primitive(primitive)) = self.context.types.kind(selected.input) else {
            return Err("source subtraction lost its integer input");
        };
        let (signed, _) = primitive
            .integer()
            .ok_or("source subtraction requires an integer")?;
        if selected.kind == crate::support::bindings::SourceIntrinsic::UncheckedSub {
            return self
                .builder
                .binary(Binary::Sub, left, right)
                .map(|result| Some(Selected::Runtime(result)));
        }
        let mut arithmetic = Arithmetic {
            constants: self.context.constants,
            builder: &mut self.builder,
            panic: &mut self.panic,
        };
        let (result, overflow) =
            match crate::sema::compiler_items::overflowing_operator(selected.kind) {
                Some(crate::lex::token::Punctuation::Minus) => {
                    arithmetic.subtract_overflow(signed, left, right)?
                }
                Some(crate::lex::token::Punctuation::Star) => {
                    arithmetic.multiply_overflow(signed, left, right)?
                }
                _ => return Err("source arithmetic lost its selected overflow operation"),
            };
        self.store_subtraction_tuple(result, overflow, selected.input, selected.output)
            .map(Some)
    }
    /// Publish the real checked result and flag using its selected tuple layout.
    fn store_subtraction_tuple(
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
    /// Modular operations never acquire the checked arithmetic panic path or
    /// C++ overflow assumptions. The builder still validates the scalar types.
    fn modular_binary(
        &mut self,
        operator: crate::lex::token::Punctuation,
        left: crate::lowir::program::Value,
        right: crate::lowir::program::Value,
    ) -> Result<Selected, &'static str> {
        let binary = match operator {
            crate::lex::token::Punctuation::Plus => Binary::Add,
            crate::lex::token::Punctuation::Minus => Binary::Sub,
            crate::lex::token::Punctuation::Star => Binary::Mul,
            _ => return Err("wrapping scalar descriptor has an invalid operation"),
        };
        self.builder
            .binary(binary, left, right)
            .map(Selected::Runtime)
    }
}
