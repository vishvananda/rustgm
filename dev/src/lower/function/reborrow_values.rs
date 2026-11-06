//! Authenticated value reborrows snapshot storage without invoking `Clone`.
use super::{Lower, Representation, Selected, ir};
use crate::{abi::layout::Layout, lowir::vocabulary::Type, types::TypeId};

impl Lower<'_, '_, '_> {
    pub(super) fn value_reborrow(
        &mut self,
        operation: ir::ValueId,
    ) -> Result<Selected, &'static str> {
        let selected = self.body.value_reborrow(self.context.types, operation)?;
        let source = self.context.layout(selected.source)?;
        let target = self.context.layout(selected.target)?;
        if source.size != target.size || source.align != target.align {
            return Err("value reborrow requires matching physical storage profiles");
        }
        if !selected.mutable {
            if let Some(field) = selected.field {
                let a = self.reborrow_field(selected.source, field.source, field.source_ty)?;
                let b = self.reborrow_field(selected.target, field.target, field.target_ty)?;
                if a != b {
                    return Err("shared value reborrow requires matching data-field storage");
                }
            } else if source.size != 0 {
                return Err("fieldless shared value reborrow requires zero-sized storage");
            }
        }
        let address = self.place(selected.place)?;
        if source.size == 0 {
            return Ok(Selected::Unit);
        }
        let source = address.ok_or("value reborrow lacks initialized source storage")?;
        let target = self
            .allocate(selected.target)?
            .ok_or("value reborrow lacks target snapshot storage")?;
        self.write(Some(target), Selected::Object(source), selected.source)?;
        match self.context.representation(selected.target)? {
            Representation::Scalar(Type::Void) => {
                Err("nonempty value reborrow has a void target representation")
            }
            Representation::Scalar(ty) => {
                Ok(Selected::Runtime(self.builder.load(ty, target.operand)?))
            }
            Representation::Object(_) | Representation::ScalarObject(_) => {
                Ok(Selected::Object(target))
            }
        }
    }

    fn reborrow_field(
        &mut self,
        owner: TypeId,
        field: ir::Selection,
        ty: TypeId,
    ) -> Result<(u64, u64, u32), &'static str> {
        let (physical, offset) = self.context.place_field(owner, field)?;
        let crate::abi::tails::Offset::Fixed(offset) = offset else {
            return Err("value reborrow requires a fixed data-field offset");
        };
        let physical = self.context.layout(physical)?;
        let actual = self.context.layout(ty)?;
        if !same_storage(physical, actual) {
            return Err("value reborrow data field differs from its physical storage");
        }
        Ok((offset, actual.size, actual.align))
    }
}

const fn same_storage(a: Layout, b: Layout) -> bool {
    a.size == b.size && a.align == b.align
}
