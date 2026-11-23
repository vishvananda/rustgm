//! Typed loads, stores and value moves through the ordinary snapshot paths.
use super::{
    super::{Lower, Representation, Selected},
    Call,
};
use crate::{
    body::ir,
    lowir::{program::Value, vocabulary::Type},
    types::TypeId,
};

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_memory(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        match (call.name, call.inputs) {
            ("volatile_load", &[pointer]) => {
                let Some((pointer, scalar)) = self.volatile_input(pointer, call.result)? else {
                    return Ok(None);
                };
                self.builder
                    .volatile_load(scalar, pointer.operand)
                    .map(|value| Some(Selected::Runtime(value)))
            }
            ("volatile_store", &[pointer, value]) => {
                let stored = self.input_type(value)?;
                let Some((pointer, scalar)) = self.volatile_input(pointer, stored)? else {
                    return Ok(None);
                };
                let value = self.value(value)?.runtime()?;
                if value.ty != scalar || call.result != TypeId::UNIT {
                    return Ok(None);
                }
                self.builder.volatile_store(value, pointer.operand)?;
                Ok(Some(Selected::Unit))
            }
            _ => Ok(None),
        }
    }
    /// The thin pointer to `target` and the scalar one volatile access moves.
    /// An aggregate or zero-sized target keeps the trap.
    fn volatile_input(
        &mut self,
        pointer: ir::ValueId,
        target: TypeId,
    ) -> Result<Option<(Value, Type)>, &'static str> {
        let Some((pointer, referent)) = self.thin_input(pointer)? else {
            return Ok(None);
        };
        let Representation::Scalar(scalar) = self.context.representation(target)? else {
            return Ok(None);
        };
        let whole = self.context.layout(target)?.size == u64::from(scalar.bits() / 8);
        Ok((referent == target && scalar != Type::Void && whole).then_some((pointer, scalar)))
    }
}
