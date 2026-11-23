//! Bring-up only: direct calls of bodiless source intrinsics are lowered here
//! without qualification. Each lowered name is waived; any other call, or a
//! call whose actual types this module does not handle, keeps its trap.
mod arithmetic;
mod atomics;
mod floats;
mod layouts;
mod memory;
mod offsets;
mod transfers;
use super::{Lower, Selected, ir};
use crate::{
    lowir::{
        program::{Operand, Value},
        vocabulary::Type,
    },
    types::{Kind, TypeId},
};

/// One evaluated direct call: the function item called, its inputs in source
/// order and its result type.
#[derive(Clone, Copy)]
struct Call<'body> {
    name: &'static str,
    callee: TypeId,
    signature: TypeId,
    arguments: ir::ValueList,
    inputs: &'body [ir::ValueId],
    result: TypeId,
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
        let ir::Rvalue::Call {
            signature,
            callee,
            arguments,
        } = value.kind
        else {
            return Ok(None);
        };
        let call = Call {
            name,
            callee: self.input_type(callee)?,
            signature,
            arguments,
            inputs: self
                .body
                .arguments
                .get(arguments.start as usize..arguments.end as usize)
                .ok_or("unqualified intrinsic arguments disappeared")?,
            result: value.ty,
        };
        let lowered = match name {
            "exact_div" => self.unqualified_arithmetic(call)?,
            "ctpop" | "ctlz" | "ctlz_nonzero" | "cttz" | "cttz_nonzero" => match call.inputs {
                &[input] => self.runtime_bits(name, input, call.result)?,
                _ => None,
            },
            "slice_get_unchecked" | "ptr_offset_from" | "ptr_offset_from_unsigned" => {
                self.unqualified_offset(call)?
            }
            "write_box_via_move" | "forget" | "volatile_load" | "volatile_store" => {
                self.unqualified_memory(call)?
            }
            "copy_nonoverlapping" | "copy" | "write_bytes" | "raw_eq" => {
                self.unqualified_transfer(call)?
            }
            "assert_zero_valid" => self.unqualified_assertion(call)?,
            "fabs" | "copysignf32" | "copysignf64" => self.unqualified_float(call)?,
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
    pub(super) fn input_type(&self, id: ir::ValueId) -> Result<TypeId, &'static str> {
        self.body
            .values
            .get(id.0 as usize)
            .map(|value| value.ty)
            .ok_or("unqualified intrinsic input disappeared")
    }
    /// The input and its referent when it is a thin raw pointer or reference.
    pub(super) fn thin_input(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<(Value, TypeId)>, &'static str> {
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
