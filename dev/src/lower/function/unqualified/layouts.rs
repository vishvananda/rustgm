//! Bring-up execution of the zero-validity assertion when it holds.
use super::{
    super::{Lower, Selected},
    Call,
};
use crate::types::{Kind, TypeId, aggregate::ZeroFields, arguments::Argument};

impl Lower<'_, '_, '_> {
    /// The assertion about the one type argument does nothing when it holds;
    /// one that fails, or that this module cannot decide, keeps the trap.
    pub(super) fn unqualified_assertion(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        let types = self.context.types;
        let Some(Kind::FunctionItem { arguments, .. }) = types.kind(call.callee) else {
            return Ok(None);
        };
        let Some(mut arguments) = types.arguments(arguments) else {
            return Ok(None);
        };
        let (Some(Argument::Type(ty)), None) = (arguments.next(), arguments.next()) else {
            return Ok(None);
        };
        if call.result != TypeId::UNIT {
            return Ok(None);
        }
        let context = &mut *self.context;
        let valid = context.aggregates.zero_valid(types, ty, |ty| {
            if let Some(Kind::Pointer { pointee, .. }) = types.kind(ty) {
                return Ok(if context.wide_referent(pointee)? {
                    ZeroFields::Unproven
                } else {
                    ZeroFields::Valid
                });
            }
            context.layout(ty)?;
            Ok(context
                .layouts
                .zero_enum_fields(types, ty)?
                .map_or(ZeroFields::Unproven, ZeroFields::Visit))
        })?;
        Ok(valid.then_some(Selected::Unit))
    }
}
