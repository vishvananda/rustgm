//! Exact typed pointer transfers reuse raw storage checks without creating references.
use super::{
    Constant, Failure, Machine,
    memory::{RawAccess, RawAlignment, Resolved},
};
use crate::{
    resolve::model::DefId,
    support::intrinsics::{ConstPermission, Intrinsic},
    types::{Kind, TypeId, callable::Convention, instance::Instance},
};
impl Machine<'_, '_> {
    fn memory_pointee(&self, intrinsic: Intrinsic, instance: Instance) -> Result<TypeId, Failure> {
        let facts = self.evaluator.facts();
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = facts.types.kind(instance.ty())
        else {
            return Err("pointer memory operation requires a selected callable".into());
        };
        if facts.intrinsics.identify(DefId(item.0)) != Some(intrinsic) {
            return Err("pointer memory operation identity mismatch".into());
        }
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_: true,
            convention: Convention::RUST,
            ..
        }) = facts.types.kind(signature)
        else {
            return Err("pointer memory operation requires an unsafe Rust signature".into());
        };
        let inputs = facts
            .types
            .list(parameters)
            .ok_or("missing pointer memory parameters")?;
        let Some(Kind::Pointer { mutable, pointee }) =
            inputs.first().and_then(|&ty| facts.types.kind(ty))
        else {
            return Err("pointer memory operation requires a typed raw pointer".into());
        };
        if intrinsic.is_fill() {
            if !mutable
                || inputs.len() != 3
                || result != TypeId::UNIT
                || inputs[1] != TypeId::primitive(crate::types::Primitive::U8)
                || inputs[2] != TypeId::primitive(crate::types::Primitive::Usize)
            {
                return Err("pointer byte fill signature mismatch".into());
            }
            return Ok(pointee);
        }
        if intrinsic.is_counted_memory() {
            if mutable
                || inputs.len() != 3
                || result != TypeId::UNIT
                || inputs[2] != TypeId::primitive(crate::types::Primitive::Usize)
                || !matches!(facts.types.kind(inputs[1]), Some(Kind::Pointer { mutable: true, pointee: target }) if target == pointee)
            {
                return Err("counted pointer memory operation signature mismatch".into());
            }
            return Ok(pointee);
        }
        let writing = intrinsic.writes_pointer();
        if mutable != writing
            || inputs.len() != if writing { 2 } else { 1 }
            || writing && (inputs[1] != pointee || result != TypeId::UNIT)
            || !writing && result != pointee
        {
            return Err("pointer memory operation signature mismatch".into());
        }
        Ok(pointee)
    }
    pub(super) fn intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        instance: Instance,
        arguments: &[Constant],
    ) -> Result<Option<Constant>, Failure> {
        if intrinsic.descriptor().constant == ConstPermission::RuntimeOnly {
            return Err("runtime-only intrinsic is not allowed during constant evaluation".into());
        }
        if intrinsic.is_pointer_offset() {
            return self
                .pointer_offset(intrinsic, instance, arguments)
                .map(Some);
        }
        if !intrinsic.is_memory() {
            return Err("constant intrinsic execution is not implemented".into());
        }
        let ty = self.memory_pointee(intrinsic, instance)?;
        self.charge(1)?;
        if intrinsic.is_fill() {
            return self.counted_fill(ty, arguments);
        }
        if intrinsic.is_counted_memory() {
            return self.counted_copy(intrinsic, ty, arguments);
        }
        let writing = intrinsic.writes_pointer();
        if arguments.len() != if writing { 2 } else { 1 } {
            return Err("pointer memory operation argument count mismatch".into());
        }
        let address = arguments[0];
        let place = Resolved {
            address,
            ty,
            raw: Some(RawAccess {
                alignment: if intrinsic.is_unaligned() {
                    RawAlignment::Unaligned
                } else {
                    RawAlignment::Required
                },
                location: address.location(),
                ty,
            }),
            union: None,
        };
        if writing {
            self.check_raw_write(place)?;
            self.write_place(place, arguments[1])?;
            Ok(Some(self.evaluator.fields(&[], None)?))
        } else {
            self.read(place)
        }
    }
}
