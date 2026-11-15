//! Bring-up only: direct calls of bodiless source intrinsics are lowered here
//! without qualification. Each lowered name is waived; any other call, or a
//! call whose actual types this module does not handle, keeps its trap.
mod arithmetic;
mod atomics;
mod bits;
mod memory;
mod offsets;
use super::{Lower, Selected, ir};
use crate::{
    lowir::{
        program::{Operand, Value},
        vocabulary::Type,
    },
    types::{Kind, TypeId},
};

/// One evaluated direct call: its inputs in source order and its result type.
#[derive(Clone, Copy)]
struct Call<'body> {
    name: &'static str,
    inputs: &'body [ir::ValueId],
    result: TypeId,
}
/// An evaluated integer input carried at its own width.
#[derive(Clone, Copy)]
struct Integer {
    ty: TypeId,
    value: Value,
    signed: bool,
}

impl Lower<'_, '_, '_> {
    /// Emit the operation in place of the call. `None` leaves the trapping shim.
    pub(super) fn unqualified_call(
        &mut self,
        name: &'static str,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("unqualified intrinsic call disappeared")?;
        let ir::Rvalue::Call { arguments, .. } = value.kind else {
            return Ok(None);
        };
        let call = Call {
            name,
            inputs: self
                .body
                .arguments
                .get(arguments.start as usize..arguments.end as usize)
                .ok_or("unqualified intrinsic arguments disappeared")?,
            result: value.ty,
        };
        let lowered = match name {
            "add_with_overflow" | "saturating_add" | "saturating_sub" | "exact_div" => {
                self.unqualified_arithmetic(call)?
            }
            "ctpop" | "ctlz" | "ctlz_nonzero" | "cttz" | "cttz_nonzero" | "bswap"
            | "bitreverse" => self.unqualified_bits(call)?,
            "offset" | "slice_get_unchecked" | "ptr_offset_from" | "ptr_offset_from_unsigned" => {
                self.unqualified_offset(call)?
            }
            "read_via_copy" | "write_via_move" | "black_box" => self.unqualified_memory(call)?,
            "unreachable" => self.unqualified_unreachable(call)?,
            _ if name.starts_with("atomic_") => self.unqualified_atomic(call)?,
            _ => None,
        };
        if lowered.is_some()
            && !crate::support::bringup::waive(&format!(
                "source intrinsic uses unqualified lowering: {name}"
            ))
        {
            return Err("source intrinsic executable lowering is not implemented");
        }
        Ok(lowered)
    }
    /// Reaching `unreachable` is undefined; the program stops as at a trap.
    fn unqualified_unreachable(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        if !call.inputs.is_empty() || call.result != TypeId::NEVER {
            return Ok(None);
        }
        let abort = self.context.abort()?;
        let signature = self.context.output.symbols[abort.0 as usize].signature;
        self.builder
            .call(signature, Type::Void, Operand::Symbol(abort), &[])?;
        Ok(Some(Selected::Unit))
    }
    fn input_type(&self, id: ir::ValueId) -> Result<TypeId, &'static str> {
        self.body
            .values
            .get(id.0 as usize)
            .map(|value| value.ty)
            .ok_or("unqualified intrinsic input disappeared")
    }
    /// The input when its type is an integer primitive.
    fn integer_input(&self, id: ir::ValueId) -> Result<Option<Integer>, &'static str> {
        let ty = self.input_type(id)?;
        let Some(Kind::Primitive(primitive)) = self.context.types.kind(ty) else {
            return Ok(None);
        };
        let Some((signed, bits)) = primitive.integer() else {
            return Ok(None);
        };
        let value = self.value(id)?.runtime()?;
        if !value.ty.is_integer() || value.ty.bits() != u32::from(bits) {
            return Err("integer intrinsic input differs from its scalar width");
        }
        Ok(Some(Integer { ty, value, signed }))
    }
    /// The input and its referent when it is a thin raw pointer or reference.
    fn thin_input(&mut self, id: ir::ValueId) -> Result<Option<(Value, TypeId)>, &'static str> {
        let Some(
            Kind::Pointer { pointee, .. }
            | Kind::Reference {
                referent: pointee, ..
            },
        ) = self.context.types.kind(self.input_type(id)?)
        else {
            return Ok(None);
        };
        if self.context.wide_referent(pointee)? {
            return Ok(None);
        }
        let value = self.value(id)?.runtime()?;
        if value.ty != Type::Ptr {
            return Err("thin pointer intrinsic input has no pointer value");
        }
        Ok(Some((value, pointee)))
    }
}
