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
        let (result, overflow) = Arithmetic {
            builder: &mut self.builder,
            panic: &mut self.panic,
        }
        .subtract_overflow(signed, left, right)?;
        let layout = self.context.layout(selected.output)?;
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
            .allocate(selected.output)?
            .ok_or("source subtraction tuple has no storage")?;
        let destination = self.offset(address, difference)?;
        self.write(Some(destination), Selected::Runtime(result), selected.input)?;
        let destination = self.offset(address, flag)?;
        let overflow = self.builder.convert(Convert::Trunc, Type::I1, overflow)?;
        self.write(
            Some(destination),
            Selected::Runtime(overflow),
            TypeId::primitive(crate::types::Primitive::Bool),
        )?;
        Ok(Some(Selected::Object(address)))
    }
}
