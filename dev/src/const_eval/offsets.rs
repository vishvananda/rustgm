//! Pointer arithmetic preserves the incoming value for zero effective displacement.
#[cfg(test)]
#[path = "../../tests/invariants/offset_access.rs"]
mod tests;
use super::{Constant, Failure, Machine, memory::Resolved};
use crate::{
    resolve::model::DefId,
    support::intrinsics::Intrinsic,
    types::{Kind, Primitive, TypeId, callable::Convention, instance::Instance},
};
impl Machine<'_, '_> {
    pub(super) fn pointer_offset(
        &mut self,
        intrinsic: Intrinsic,
        instance: Instance,
        arguments: &[Constant],
    ) -> Result<Constant, Failure> {
        let facts = self.evaluator.facts();
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = facts.types.kind(instance.ty())
        else {
            return Err("pointer add requires a canonical callable".into());
        };
        if !intrinsic.is_pointer_offset()
            || facts.intrinsics.identify(DefId(item.0)) != Some(intrinsic)
        {
            return Err("pointer add identity mismatch".into());
        }
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_: true,
            convention: Convention::RUST,
            ..
        }) = facts.types.kind(signature)
        else {
            return Err("pointer add requires an unsafe Rust signature".into());
        };
        let Some(&[pointer, count]) = facts.types.list(parameters) else {
            return Err("pointer add parameter count mismatch".into());
        };
        let Some(Kind::Pointer { mutable, pointee }) = facts.types.kind(pointer) else {
            return Err("pointer add requires a raw pointer".into());
        };
        if result != pointer
            || count != TypeId::primitive(Primitive::Usize)
            || mutable != (intrinsic == Intrinsic::MutPointerAdd)
        {
            return Err("pointer add signature mismatch".into());
        }
        let [address, count] = arguments else {
            return Err("pointer add argument count mismatch".into());
        };
        let count = u64::try_from(super::values::integer(*count)?)
            .map_err(|_| "pointer add count exceeds usize")?;
        let (layout, work) = self
            .evaluator
            .address_layout(pointee, self.limits.steps.saturating_sub(self.work.steps))?;
        self.charge(work)?;
        let displacement = count
            .checked_mul(layout.size)
            .filter(|&n| i64::try_from(n).is_ok())
            .ok_or("pointer add byte displacement exceeds isize")?;
        if displacement == 0 {
            return Ok(*address);
        }
        self.byte_address(
            Resolved {
                address: *address,
                ty: pointee,
                raw: None,
                union: None,
            },
            displacement,
            pointee,
        )
    }
}
