//! Counted transfers, fills and byte comparisons call one shim per instance.
use super::{
    super::{Lower, Selected},
    Call,
};
use crate::{
    lowir::program::Operand,
    types::{Primitive, TypeId, instance::Instance},
};

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_transfer(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        // The leading thin pointers to one element type, then the scalars.
        let (pointers, scalars, result): (usize, &[Primitive], TypeId) = match call.name {
            "copy_nonoverlapping" | "copy" => (2, &[Primitive::Usize], TypeId::UNIT),
            "write_bytes" => (1, &[Primitive::U8, Primitive::Usize], TypeId::UNIT),
            "compare_bytes" => (2, &[Primitive::Usize], TypeId::primitive(Primitive::I32)),
            "raw_eq" => (2, &[], TypeId::primitive(Primitive::Bool)),
            _ => return Ok(None),
        };
        if call.inputs.len() != pointers + scalars.len() || call.result != result {
            return Ok(None);
        }
        let (addressed, counted) = call.inputs.split_at(pointers);
        let mut element = None;
        for &pointer in addressed {
            let Some((_, referent)) = self.thin_input(pointer)? else {
                return Ok(None);
            };
            if element.is_some_and(|element| element != referent) {
                return Ok(None);
            }
            element = Some(referent);
        }
        for (&input, &scalar) in counted.iter().zip(scalars) {
            if self.input_type(input)? != TypeId::primitive(scalar) {
                return Ok(None);
            }
        }
        let Some(element) = element else {
            return Ok(None);
        };
        if call.name == "compare_bytes" && element != TypeId::primitive(Primitive::U8) {
            return Ok(None);
        }
        let instance = self
            .body
            .callable_target(Instance::new(self.context.types, call.callee)?);
        let Some(symbol) =
            self.context
                .demand_unqualified(instance, call.signature, call.name, element)?
        else {
            return Ok(None);
        };
        self.call_operand(
            call.signature,
            Operand::Symbol(symbol),
            call.arguments,
            call.result,
        )
        .map(Some)
    }
}
