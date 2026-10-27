//! Slice and str references validate initialized elements in their retained range.
use super::{Constant, Failure, Machine, Read, Reader, TypeId};
use crate::{
    const_eval::static_reads::ReadPurpose,
    sema::values::{Origin, StoredShape},
};
impl Machine<'_, '_> {
    pub(in crate::const_eval) fn checked_sequence_reference(
        &mut self,
        place: crate::const_eval::memory::Resolved,
        mutable: bool,
    ) -> Result<Option<Constant>, Failure> {
        let access = place
            .raw
            .ok_or("constant sequence reference lacks its raw anchor")?;
        if access.ty != place.ty
            || access.location != place.address.location()
            || place.union.is_some()
        {
            return Err("constant sequence reference differs from its raw anchor".into());
        }
        self.path(place.address)?;
        let (target, metadata) = self.evaluator.pointer_words(place.address)?;
        let crate::sema::values::WordTarget::Value(pointer) = target else {
            return Err("constant sequence reference lacks a data pointer".into());
        };
        let Some(crate::sema::values::Metadata::Length(length)) = metadata else {
            return Err("constant sequence reference lacks length metadata".into());
        };
        let (address, work) = self
            .evaluator
            .word_address(pointer, self.limits.steps.saturating_sub(self.work.steps))?;
        self.charge(work)?;
        let selected = self.reconstruct_sequence_word(address, place.ty, length, true)?;
        Ok(
            match self.validate_sequence_word_reference(selected, place.ty, mutable)? {
                Read::Pending => None,
                Read::Value(value) => Some(value),
            },
        )
    }
    pub(super) fn validate_sequence_word_reference(
        &mut self,
        address: Constant,
        ty: TypeId,
        mutable: bool,
    ) -> Result<Read<Constant>, Failure> {
        let path = self.path(address)?;
        let range = path
            .range()?
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
        let expected = Reader::new(self.evaluator.facts()).sequence_element(ty)?;
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
