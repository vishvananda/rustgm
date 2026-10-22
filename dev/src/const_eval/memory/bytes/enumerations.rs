//! A cold per-type tag index never turns byte transport into a typed enum read.
use super::{Backing, Constant, Failure, Machine, Reader, TypeId, continuations::Source};
use crate::sema::{members::VariantId, values::View};
use std::collections::HashMap;
#[cfg(test)]
#[path = "../../../../tests/invariants/enum_bytes.rs"]
mod tests;
struct Index {
    tags: Vec<(u128, VariantId)>,
}
#[derive(Default)]
pub(super) struct Enumerations {
    index: HashMap<TypeId, Index>,
    variants: usize,
}
impl Enumerations {
    pub(super) fn retained_slots(&self) -> usize {
        self.index
            .capacity()
            .saturating_mul(4 * size_of::<(TypeId, Index)>())
            .saturating_add(4 * size_of::<Self>())
            .div_ceil(size_of::<Constant>())
            .saturating_add(self.variants)
    }
}
impl Machine<'_, '_> {
    pub(super) fn prepare_byte_enum(&mut self, ty: TypeId) -> Result<bool, Failure> {
        self.charge(1)?;
        if self
            .memory
            .bytes
            .enumerations
            .as_ref()
            .is_some_and(|cache| cache.index.contains_key(&ty))
        {
            return Ok(true);
        }
        let count = Reader::new(self.evaluator.facts())
            .enum_variants(ty)?
            .variants
            .len();
        let sorting = count
            .checked_mul(count.checked_ilog2().unwrap_or(0) as usize + 2)
            .ok_or("constant enum tag indexing work overflow")?;
        self.charge(sorting)?;
        let growth = self.memory.bytes.enumerations.as_ref().map_or(
            4 * size_of::<Enumerations>() + 16 * size_of::<(TypeId, Index)>(),
            |cache| {
                if cache.index.len() == cache.index.capacity() {
                    cache
                        .index
                        .capacity()
                        .saturating_mul(2)
                        .max(4)
                        .saturating_mul(4 * size_of::<(TypeId, Index)>())
                } else {
                    0
                }
            },
        );
        let reserved = count
            .saturating_mul(size_of::<(u128, VariantId)>())
            .saturating_add(growth)
            .div_ceil(size_of::<Constant>());
        self.reserve_cells(reserved)?;
        let mut tags = Vec::new();
        tags.try_reserve_exact(count)
            .map_err(|_| "constant enum tag index allocation failed")?;
        for ordinal in 0..count {
            let reader = Reader::new(self.evaluator.facts());
            let stored = reader.enum_variants(ty)?;
            let id = stored.identities[ordinal];
            let bits = reader.enum_variant_bits(ty, id)?;
            let member = self
                .evaluator
                .facts()
                .members
                .variants
                .get(id.0 as usize)
                .ok_or("missing stored enum declaration")?;
            if member.owner != stored.owner || member.ordinal as usize != ordinal {
                return Err("stored enum declaration ordering differs from layout".into());
            }
            let count = reader.enum_variant_fields(ty, id)?.0.len();
            let start = member.start;
            self.charge(count)?;
            for index in 0..count {
                let field = crate::sema::members::FieldId(
                    start
                        .checked_add(
                            u32::try_from(index)
                                .map_err(|_| "enum field index exceeds declaration")?,
                        )
                        .ok_or("enum field identity overflow")?,
                );
                Reader::new(self.evaluator.facts()).enum_field_layout(ty, field)?;
            }
            tags.push((bits, id));
        }
        tags.sort_unstable_by_key(|&(bits, _)| bits);
        if tags.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err("stored enum contains duplicate physical tags".into());
        }
        let retained = tags
            .capacity()
            .saturating_mul(size_of::<(u128, VariantId)>())
            .div_ceil(size_of::<Constant>());
        let cache = self
            .memory
            .bytes
            .enumerations
            .get_or_insert_with(Default::default);
        cache
            .index
            .try_reserve(1)
            .map_err(|_| "constant enum type index allocation failed")?;
        cache.variants = cache
            .variants
            .checked_add(retained)
            .ok_or("constant enum tag retention overflow")?;
        cache.index.insert(ty, Index { tags });
        Ok(true)
    }
    pub(super) fn check_tagless_value(&self, ty: TypeId, value: Constant) -> Result<(), Failure> {
        let reader = Reader::new(self.evaluator.facts());
        if matches!(self.evaluator.facts().types.kind(ty), Some(crate::types::Kind::Adt(owner,_)) if self.evaluator.facts().aggregates.get(owner).is_some_and(|definition| definition.kind == crate::types::aggregate::Kind::Enum))
            && let crate::sema::values::StoredShape::Product {
                variant: Some(expected),
                fields,
                ..
            } = reader.storage_shape(ty)?
            && !matches!(reader.view(value)?, View::Product { length, variant: Some(actual) } if actual == expected && length == fields.len() as u64)
        {
            return Err("tagless enum encoding requires its exact sole variant and fields".into());
        }
        Ok(())
    }
    pub(super) fn encode_byte_enum(
        &mut self,
        backing: &mut Backing,
        offset: u64,
        ty: TypeId,
        value: Constant,
    ) -> Result<(), Failure> {
        self.prepare_byte_enum(ty)?;
        let View::Product {
            length,
            variant: Some(id),
        } = Reader::new(self.evaluator.facts()).view(value)?
        else {
            return Err("constant enum encoding requires its exact variant".into());
        };
        if Reader::new(self.evaluator.facts())
            .enum_variant_fields(ty, id)?
            .0
            .len() as u64
            != length
        {
            return Err("constant enum value differs from its variant fields".into());
        }
        self.encode_byte_enum_tag(backing, offset, ty, id)
    }
    pub(super) fn encode_byte_enum_tag(
        &mut self,
        backing: &mut Backing,
        offset: u64,
        ty: TypeId,
        id: VariantId,
    ) -> Result<(), Failure> {
        self.prepare_byte_enum(ty)?;
        let reader = Reader::new(self.evaluator.facts());
        let tag = reader.enum_variants(ty)?.tag;
        let bits = reader.enum_variant_bits(ty, id)?;
        let width = usize::from(
            tag.ty
                .integer()
                .ok_or("stored enum tag must be an integer")?
                .1
                / 8,
        );
        let offset = offset
            .checked_add(tag.offset)
            .ok_or("enum tag byte offset overflow")?;
        let start = usize::try_from(offset).map_err(|_| "enum tag offset exceeds host size")?;
        let end = start
            .checked_add(width)
            .filter(|&end| end <= backing.bytes.len())
            .ok_or("enum tag encoding exceeds backing bounds")?;
        self.charge(width)?;
        self.buffer_fragments(backing, offset, width as u64, &[])?;
        backing.bytes[start..end].copy_from_slice(&bits.to_le_bytes()[..width]);
        for index in start..end {
            backing.set_initialized(index, true);
        }
        Ok(())
    }
    pub(super) fn decode_byte_variant(
        &mut self,
        source: Source,
        offset: u64,
        ty: TypeId,
    ) -> Result<VariantId, Failure> {
        self.prepare_byte_enum(ty)?;
        let tag = Reader::new(self.evaluator.facts()).enum_variants(ty)?.tag;
        let width = usize::from(
            tag.ty
                .integer()
                .ok_or("stored enum tag must be an integer")?
                .1
                / 8,
        );
        let start = offset
            .checked_add(tag.offset)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("enum tag read offset overflow")?;
        let end = start
            .checked_add(width)
            .ok_or("enum tag read extent overflow")?;
        let crate::sema::model::Scalar::Integer(bits) = self
            .decode_byte_scalar(source, start, end, TypeId::primitive(tag.ty))?
            .as_scalar()?
        else {
            return Err("stored enum tag is not an integer".into());
        };
        let index = self
            .memory
            .bytes
            .enumerations
            .as_ref()
            .and_then(|cache| cache.index.get(&ty))
            .ok_or("missing prepared enum tag index")?;
        let count = index.tags.len();
        self.charge(count.checked_ilog2().unwrap_or(0) as usize + 2)?;
        let index = self
            .memory
            .bytes
            .enumerations
            .as_ref()
            .and_then(|cache| cache.index.get(&ty))
            .ok_or("missing prepared enum tag index")?;
        let found = index
            .tags
            .binary_search_by_key(&bits, |&(tag, _)| tag)
            .map_err(|_| "constant enum read has an invalid discriminant")?;
        Ok(index.tags[found].1)
    }
    #[cfg(test)]
    fn decode_byte_enum(
        &mut self,
        source: Source,
        offset: u64,
        ty: TypeId,
    ) -> Result<Constant, Failure> {
        let variant = self.decode_byte_variant(source, offset, ty)?;
        if !Reader::new(self.evaluator.facts())
            .enum_variant_fields(ty, variant)?
            .0
            .is_empty()
        {
            return Err("fieldless test helper received payload".into());
        }
        self.evaluator.fields(&[], Some(variant))
    }
}
