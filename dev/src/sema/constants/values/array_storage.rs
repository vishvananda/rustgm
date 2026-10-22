//! Byte-array objects retain symbolic storage without manufacturing integer elements.
use super::{Constant, Evaluator, Failure, ImageView, Reader};
use crate::{
    sema::constants::images,
    types::{Kind, Primitive, TypeId},
};
impl<'a> Reader<'a> {
    pub(crate) fn array_image(
        self,
        value: Constant,
    ) -> Result<Option<ImageView<'a>>, &'static str> {
        let image = self.image(value)?;
        let Some(Kind::Array { element, length }) = self.facts.types.kind(image.ty) else {
            return Ok(None);
        };
        if element != TypeId::primitive(Primitive::U8)
            || length != image.bytes.len() as u64
            || self
                .facts
                .layouts
                .get(image.ty)
                .is_none_or(|layout| layout.size != length)
        {
            return Err("constant byte array image differs from its exact storage type");
        }
        Ok(Some(image))
    }
    pub(crate) fn array_image_field(
        self,
        value: Constant,
        index: u64,
    ) -> Result<Constant, &'static str> {
        let image = self
            .array_image(value)?
            .ok_or("union byte storage has no structural field view")?;
        let index = usize::try_from(index)
            .ok()
            .filter(|&i| i < image.bytes.len())
            .ok_or("constant index is out of bounds")?;
        if !image.initialized(index) {
            return Err("constant byte array read uses uninitialized bytes");
        }
        let next = image.fragments.partition_point(|fragment| {
            fragment.offset + u64::from(fragment.length) <= index as u64
        });
        if image
            .fragments
            .get(next)
            .is_some_and(|fragment| fragment.offset <= index as u64)
        {
            return Err("constant numeric read uses symbolic pointer bytes");
        }
        Ok(Constant::scalar(crate::sema::model::Scalar::Integer(
            u128::from(image.bytes[index]),
        )))
    }
    pub(crate) fn require_publishable_array(self, value: Constant) -> Result<(), &'static str> {
        if let Some(image) = self.array_image(value)? {
            if !image.fragments.is_empty() {
                return Err("constant byte array publication contains pointer provenance");
            }
            if (0..image.bytes.len()).any(|index| !image.initialized(index)) {
                return Err("constant byte array publication uses uninitialized bytes");
            }
        }
        Ok(())
    }
    pub(crate) fn image_field_work(self, value: Constant) -> Result<usize, &'static str> {
        if !matches!(value.0, super::Value::Image(_)) {
            return Ok(0);
        }
        let image = self
            .array_image(value)?
            .ok_or("union byte storage has no structural field view")?;
        Ok(image
            .fragments
            .len()
            .checked_ilog2()
            .map_or(1, |n| n as usize + 2))
    }
}
impl Evaluator<'_> {
    /// Select only a canonical storage type; this grants no access or conversion proof.
    pub(crate) fn byte_array_range_type(
        &mut self,
        owner: TypeId,
        from: u64,
        length: u64,
    ) -> Result<TypeId, Failure> {
        let facts = &mut *self.queries.typing.facts;
        let Some(Kind::Array {
            element,
            length: bound,
        }) = facts.types.kind(owner)
        else {
            return Err("constant byte range requires its exact array owner".into());
        };
        if element != TypeId::primitive(Primitive::U8) {
            return Err("constant symbolic range requires u8 array storage".into());
        }
        if from.checked_add(length).is_none_or(|end| end > bound) {
            return Err("constant byte range is out of bounds".into());
        }
        Ok(facts.types.intern(Kind::Array { element, length })?)
    }
    pub(crate) fn byte_array_snapshot(
        &mut self,
        ty: TypeId,
        bytes: &[u8],
        initialized: &[u64],
        fragments: &[super::ImageFragment],
        limit: usize,
    ) -> Result<(Constant, usize), Failure> {
        if !matches!(self.facts().types.kind(ty), Some(Kind::Array { element,length }) if element == TypeId::primitive(Primitive::U8) && length == bytes.len() as u64)
        {
            return Err("constant byte snapshot requires its exact u8 array type".into());
        }
        let (layout, mut work) = self.storage_layout(ty, limit)?;
        if layout.size != bytes.len() as u64 || initialized.len() != bytes.len().div_ceil(64) {
            return Err("constant byte snapshot differs from array storage".into());
        }
        work = work
            .checked_add(bytes.len().saturating_mul(2))
            .and_then(|n| n.checked_add(initialized.len().saturating_mul(2)))
            .and_then(|n| n.checked_add(fragments.len().saturating_mul(16)))
            .filter(|&n| n <= limit)
            .ok_or("constant byte array snapshot work budget exceeded")?;
        for index in 0..bytes.len() {
            if initialized[index / 64] & (1u64 << (index % 64)) == 0 {
                return Err("constant byte array read uses uninitialized bytes".into());
            }
        }
        let arena = self
            .queries
            .typing
            .facts
            .constant_images
            .get_or_insert_with(Default::default);
        arena.reserve_array_fragments(bytes.len(), fragments.len())?;
        let value = arena.insert_array(images::Image {
            ty,
            bytes: bytes.to_vec().into_boxed_slice(),
            initialized: initialized.to_vec().into_boxed_slice(),
            fragments: fragments.to_vec().into_boxed_slice(),
        })?;
        Ok((Constant(value), work))
    }
}
