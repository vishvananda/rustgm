//! Slice and str references validate initialized elements in their retained range.
use super::{Constant, Failure, Machine, Read, Reader, TypeId};
use crate::{
    const_eval::{memory::Range, static_reads::ReadPurpose},
    sema::values::{Origin, StoredShape},
};
impl Machine<'_, '_> {
    pub(super) fn validate_cstr_word_reference(
        &mut self,
        value: Constant,
        referent: TypeId,
        profile: crate::sema::compiler_items::c_strings::LiteralProfile,
        mutable: bool,
    ) -> Result<Read<Constant>, Failure> {
        self.charge(1)?;
        self.path(value)?;
        let (target, metadata) = self.evaluator.pointer_words(value)?;
        let crate::sema::values::WordTarget::Value(pointer) = target else {
            return Err("CStr reference lacks its actual data word".into());
        };
        let Some(crate::sema::values::Metadata::Length(length)) = metadata else {
            return Err("CStr reference lacks its actual length metadata".into());
        };
        let (address, work) = self
            .evaluator
            .word_address(pointer, self.limits.steps.saturating_sub(self.work.steps))?;
        self.charge(work)?;
        // Authenticate the same finite nominal/backing relation before validating
        // its actual slice field. The internal slice is not the returned type.
        self.evaluator
            .word_sequence_range(address, length, referent)?;
        let slice = Reader::new(self.evaluator.facts()).stored_child(profile.referent, None, 0)?;
        if !matches!(self.evaluator.facts().types.kind(slice), Some(crate::types::Kind::Slice(element)) if element == profile.element)
        {
            return Err("CStr reference differs from its actual character-slice field".into());
        }
        let selected = self.evaluator.word_sequence_range(address, length, slice)?;
        Ok(
            match self.validate_sequence_word_reference(selected, slice, mutable)? {
                Read::Pending => Read::Pending,
                Read::Value(_) => Read::Value(if mutable {
                    value
                } else {
                    value.restrict_shared()
                }),
            },
        )
    }
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
        // Array unsizing may retain the original zero-offset place. Its actual
        // metadata selects a prefix; the backing, element, bounds and validity
        // checks below remain mandatory, as for an explicit range projection.
        let range = path
            .range()?
            .or_else(|| address.length().map(|length| Range { from: 0, length }))
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
        if (!Reader::new(self.evaluator.facts()).same_storage_type(element, expected)
            && !Reader::new(self.evaluator.facts()).byte_reinterpretation(element, expected))
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
