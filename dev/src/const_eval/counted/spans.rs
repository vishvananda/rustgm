//! Span proofs retain canonical storage coordinates and the original pointer capability.
use super::{Constant, Failure, Machine, TransferSpan};
use crate::{
    const_eval::memory::{Liveness, Range},
    sema::values::Reader,
    types::{Kind, TypeId},
};
impl Machine<'_, '_> {
    pub(super) fn transfer_span(
        &mut self,
        address: Constant,
        ty: TypeId,
        count: u64,
        bytes: u64,
        retained: usize,
    ) -> Result<TransferSpan, Failure> {
        let address = self.evaluator.literal_address(address)?;
        if address.location().is_none() {
            return Err("constant counted numeric pointer access is not implemented".into());
        }
        let path = self.path_for(
            address,
            if bytes == 0 {
                Liveness::AddressOnly
            } else {
                Liveness::Required
            },
            retained,
        )?;
        let byte_offset = match path.terminal {
            Some(crate::const_eval::memory::Terminal::Bytes { offset, .. }) => Some(offset),
            _ => None,
        };
        let (root, mut fields) = if byte_offset.is_some() {
            (path.root, path.fields)
        } else {
            path.fields_only()?
        };
        let root_type = Reader::new(self.evaluator.facts()).location(root)?.ty;
        let layout = self.footprint(root_type)?;
        let (stored, offset, image) = if let Some(offset) = byte_offset {
            (ty, offset, None)
        } else {
            self.storage_coordinates::<true>(root_type, &fields)?
        };
        let reinterpreted = byte_offset.is_some()
            || !Reader::new(self.evaluator.facts()).same_storage_type(stored, ty);
        let target = self.footprint(ty)?;
        if layout.align < target.align || !offset.is_multiple_of(u64::from(target.align)) {
            return Err("constant counted pointer is unaligned".into());
        }
        let mut range = None;
        if bytes != 0 {
            if offset
                .checked_add(bytes)
                .is_none_or(|end| end > layout.size)
            {
                return Err("constant counted transfer exceeds allocation bounds".into());
            }
            let reader = Reader::new(self.evaluator.facts());
            let location =
                reader.location(address.location().ok_or("counted pointer lacks storage")?)?;
            if !reinterpreted
                && matches!(
                    location.origin,
                    crate::sema::values::Origin::Projection { .. }
                )
            {
                let (parent, start) = address.projection_parent(reader)?;
                let parent_type = reader
                    .location(parent.location().ok_or("counted parent lacks storage")?)?
                    .ty;
                if let Some(Kind::Array { element, length }) =
                    self.evaluator.facts().types.kind(parent_type)
                    && reader.same_storage_type(element, ty)
                {
                    if start.checked_add(count).is_none_or(|end| end > length) {
                        return Err("constant counted transfer exceeds its array bounds".into());
                    }
                    if fields.pop() != Some(start) {
                        return Err(
                            "constant counted array ancestry differs from its storage path".into(),
                        );
                    }
                    range = Some(Range {
                        from: start,
                        length: count,
                    });
                }
            }
            if !reinterpreted && range.is_none() && count != 1 {
                return Err(
                    "constant counted transfer requires contiguous array element storage".into(),
                );
            }
        }
        Ok(TransferSpan {
            address,
            ty,
            root,
            fields,
            range,
            offset,
            bytes,
            image,
        })
    }
}
