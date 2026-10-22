//! A reconstructed slice reference validates its actual retained range, not raw bytes.
use super::{Constant, Failure, Machine, Read, Reader, TypeId};
use crate::{
    const_eval::static_reads::ReadPurpose,
    sema::values::{Origin, StoredShape},
    types::Kind,
};
impl Machine<'_, '_> {
    pub(super) fn validate_slice_word_reference(
        &mut self,
        address: Constant,
        ty: TypeId,
        mutable: bool,
    ) -> Result<Read<Constant>, Failure> {
        let path = self.path(address)?;
        let range = path
            .range
            .ok_or("symbolic slice reference lacks its normalized range")?;
        if address.length() != Some(range.length) {
            return Err("symbolic slice metadata differs from its retained range".into());
        }
        let location = Reader::new(self.evaluator.facts()).location(path.root)?;
        if mutable {
            self.check_raw_write_authority(address, path.root)?;
        }
        if let Origin::Static(id) = location.origin {
            self.check_static_read(id)?;
            if self.active_initializers.contains(&id) {
                return Err(
                    "constant slice references to cyclic static storage are not implemented".into(),
                );
            }
        }
        let root = self.footprint(location.ty)?;
        let (stored, offset) = self.storage_offset(location.ty, &path.fields)?;
        let Some(Kind::Slice(expected)) = self.evaluator.facts().types.kind(ty) else {
            return Err("symbolic slice validity requires a slice type".into());
        };
        let StoredShape::Array {
            element,
            stride,
            length,
        } = Reader::new(self.evaluator.facts()).storage_shape(stored)?
        else {
            return Err("symbolic slice validity requires original array storage".into());
        };
        if !Reader::new(self.evaluator.facts()).same_storage_type(element, expected)
            || range
                .from
                .checked_add(range.length)
                .is_none_or(|end| end > length)
        {
            return Err(
                "symbolic slice reference differs from its backing element or bounds".into(),
            );
        }
        let leaf = self.footprint(element)?;
        let start = stride
            .checked_mul(range.from)
            .and_then(|n| offset.checked_add(n))
            .ok_or("symbolic slice byte offset overflow")?;
        let end = stride
            .checked_mul(range.length)
            .and_then(|n| start.checked_add(n))
            .ok_or("symbolic slice byte extent overflow")?;
        if end > root.size {
            return Err("symbolic slice reference exceeds allocation bounds".into());
        }
        if root.align < leaf.align || !start.is_multiple_of(u64::from(leaf.align)) {
            return Err("constant symbolic slice reference is unaligned".into());
        }
        let value = self.read_storage(
            path.root,
            &path.fields,
            Some(range),
            ty,
            ReadPurpose::ReferenceValidation,
        )?;
        Ok(value.map_or(Read::Pending, |_| {
            Read::Value(if mutable {
                address
            } else {
                address.restrict_shared()
            })
        }))
    }
}
