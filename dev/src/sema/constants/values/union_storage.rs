//! Scalar-only union families share byte images, never active field products.
use super::super::images::{self, Family, Image};
use super::{Constant, Evaluator, Failure, Reader, Value};
use crate::{
    sema::members::Selection,
    types::{Kind, Primitive, TypeId, aggregate::ShapeReadiness},
};

#[derive(Clone, Copy)]
pub struct ImageView<'a> {
    pub ty: TypeId,
    pub bytes: &'a [u8],
    /// Bit i (least significant first) describes byte i, independently of payload bits.
    pub initialized: &'a [u64],
}
impl ImageView<'_> {
    #[must_use]
    pub fn initialized(self, index: usize) -> bool {
        index < self.bytes.len()
            && self
                .initialized
                .get(index / 64)
                .is_some_and(|word| word & (1u64 << (index % 64)) != 0)
    }
}
impl<'a> Reader<'a> {
    /// Read immutable typed byte storage without exposing arena identity.
    /// # Errors
    /// Rejects non-images and malformed internal storage.
    pub fn image(self, value: Constant) -> Result<ImageView<'a>, &'static str> {
        let Value::Image(id) = value.0 else {
            return Err("constant value has no union image");
        };
        let image = self
            .facts
            .constant_images
            .as_ref()
            .ok_or("missing constant image arena")?
            .get(id)?;
        Ok(ImageView {
            ty: image.ty,
            bytes: &image.bytes,
            initialized: &image.initialized,
        })
    }
}
impl Evaluator<'_> {
    fn prepare_union(&mut self, ty: TypeId, limit: usize) -> Result<usize, Failure> {
        if self
            .facts()
            .constant_images
            .as_ref()
            .is_some_and(|a| a.families.contains_key(&ty))
        {
            return (limit != 0)
                .then_some(1)
                .ok_or("constant union image work budget exceeded".into());
        }
        let facts = self.facts();
        let Some(Kind::Adt(item, _)) = facts.types.kind(ty) else {
            return Err("union image requires a nominal type".into());
        };
        let ShapeReadiness::Ready(shape) = facts.aggregates.try_shape(&facts.types, ty)? else {
            return Err("union image requires completed fields".into());
        };
        if shape.definition.kind != crate::types::aggregate::Kind::Union {
            return Err("union image requires a union type".into());
        }
        let variant = shape
            .variants()
            .next()
            .ok_or("missing union storage fields")?;
        let fields = variant.fields;
        let types = facts
            .types
            .list(fields)
            .ok_or("missing scalar union field list")?;
        let count = types.len();
        let mut work = count
            .checked_mul(2)
            .and_then(|n| n.checked_add(1))
            .filter(|n| *n <= limit)
            .ok_or("constant union image work budget exceeded")?;
        for &field in types {
            if !matches!(facts.types.kind(field), Some(Kind::Primitive(p)) if p != Primitive::Str) {
                return Err("constant union images require an all-fields primitive scalar family; pointer and aggregate provenance storage is not implemented".into());
            }
        }
        let [member] = facts.members.variants(item) else {
            return Err("union storage requires one declaration field group".into());
        };
        let member = facts
            .members
            .variants
            .get(member.0 as usize)
            .ok_or("missing union declaration fields")?;
        if member.owner != item
            || member.end - member.start
                != u32::try_from(count).map_err(|_| "too many union fields")?
        {
            return Err("union declaration differs from completed storage fields".into());
        }
        let start = member.start;
        let (layout, layout_work) = self.storage_layout(ty, limit - work)?;
        work = work
            .checked_add(layout_work)
            .filter(|n| *n <= limit)
            .ok_or("constant union image work budget exceeded")?;
        let size = usize::try_from(layout.size)
            .ok()
            .filter(|n| *n <= images::MAX_IMAGE)
            .ok_or("constant union image exceeds 64 KiB limit")?;
        let facts = &mut *self.queries.typing.facts;
        let arena = facts.constant_images.get_or_insert_with(Default::default);
        arena.reserve_family(count)?;
        let mut primitives = Vec::with_capacity(count);
        for &ty in facts
            .types
            .list(fields)
            .ok_or("missing scalar union fields")?
        {
            let Some(Kind::Primitive(primitive)) = facts.types.kind(ty) else {
                return Err("union field changed after preparation".into());
            };
            primitives.push(primitive);
        }
        arena.families.insert(
            ty,
            Family {
                start,
                size,
                fields: primitives.into_boxed_slice(),
            },
        );
        Ok(work)
    }
    fn union_field(
        &mut self,
        ty: TypeId,
        field: Selection,
        limit: usize,
    ) -> Result<(Primitive, usize, usize), Failure> {
        let work = self.prepare_union(ty, limit)?;
        let family = self
            .facts()
            .constant_images
            .as_ref()
            .and_then(|a| a.families.get(&ty))
            .ok_or("missing prepared scalar union")?;
        let Selection::Nominal(field) = field else {
            return Err("union image requires a nominal field identity".into());
        };
        let index = field
            .0
            .checked_sub(family.start)
            .ok_or("union field has a different owner")?;
        let primitive = *family
            .fields
            .get(index as usize)
            .ok_or("union field has a different owner")?;
        Ok((primitive, family.size, work))
    }
    pub(crate) fn union_address(
        &mut self,
        base: Constant,
        owner: TypeId,
        field: Selection,
        limit: usize,
    ) -> Result<(Constant, usize), Failure> {
        let location = base
            .location()
            .ok_or("constant union address lacks allocation provenance")?;
        if Reader::new(self.facts()).location(location)?.ty != owner {
            return Err("constant union address differs from its canonical owner".into());
        }
        let (primitive, _, work) = self.union_field(owner, field, limit)?;
        let Selection::Nominal(field) = field else {
            return Err("union address requires a nominal field".into());
        };
        let index = self
            .facts()
            .members
            .ordinal(field)
            .ok_or("missing union address field ordinal")?;
        let work = work
            .checked_add(1)
            .filter(|&work| work <= limit)
            .ok_or("constant union address work budget exceeded")?;
        let value = self.projection(base, u64::from(index), TypeId::primitive(primitive))?;
        Ok((value, work))
    }
    pub(crate) fn union_value(
        &mut self,
        ty: TypeId,
        field: Selection,
        value: Constant,
        limit: usize,
    ) -> Result<(Constant, usize), Failure> {
        self.update_union(None, ty, field, value, limit)
    }
    pub(crate) fn union_write(
        &mut self,
        image: Constant,
        ty: TypeId,
        field: Selection,
        value: Constant,
        limit: usize,
    ) -> Result<(Constant, usize), Failure> {
        self.update_union(Some(image), ty, field, value, limit)
    }
    fn update_union(
        &mut self,
        prior: Option<Constant>,
        ty: TypeId,
        field: Selection,
        value: Constant,
        limit: usize,
    ) -> Result<(Constant, usize), Failure> {
        let (primitive, size, work) = self.union_field(ty, field, limit)?;
        let (bits, width) = images::scalars::encode(primitive, value.as_scalar()?)?;
        let work = work
            .checked_add(size * 2 + size.div_ceil(64) * 2 + width * 2)
            .filter(|n| *n <= limit)
            .ok_or("constant union image work budget exceeded")?;
        if width > size {
            return Err("union scalar exceeds storage bounds".into());
        }
        if let Some(prior) = prior {
            let image = Reader::new(self.facts()).image(prior)?;
            if image.ty != ty || image.bytes.len() != size {
                return Err("union image differs from its checked storage type".into());
            }
        }
        let arena = self
            .queries
            .typing
            .facts
            .constant_images
            .as_mut()
            .ok_or("missing union image arena")?;
        arena.reserve_image(size)?;
        let (mut bytes, mut initialized) = if let Some(Constant(Value::Image(id))) = prior {
            let image = arena.get(id)?;
            (image.bytes.to_vec(), image.initialized.to_vec())
        } else {
            (vec![0; size], vec![0; size.div_ceil(64)])
        };
        bytes[..width].copy_from_slice(&bits.to_le_bytes()[..width]);
        for index in 0..width {
            initialized[index / 64] |= 1u64 << (index % 64);
        }
        let value = arena.insert(Image {
            ty,
            bytes: bytes.into_boxed_slice(),
            initialized: initialized.into_boxed_slice(),
        })?;
        Ok((Constant(value), work))
    }
    pub(crate) fn union_read(
        &mut self,
        image: Constant,
        ty: TypeId,
        field: Selection,
        limit: usize,
    ) -> Result<(Constant, usize), Failure> {
        let (primitive, size, work) = self.union_field(ty, field, limit)?;
        let width = images::scalars::width(primitive)?;
        let work = work
            .checked_add(width * 2)
            .filter(|n| *n <= limit)
            .ok_or("constant union image work budget exceeded")?;
        let image = Reader::new(self.facts()).image(image)?;
        if image.ty != ty || image.bytes.len() != size {
            return Err("union image differs from its checked storage type".into());
        }
        let scalar = images::scalars::decode(primitive, image.bytes, image.initialized)
            .map_err(Failure::Fault)?;
        Ok((Constant::scalar(scalar), work))
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/union_images.rs"]
mod tests;
