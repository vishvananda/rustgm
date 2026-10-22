//! Enum byte tags retain declaration identities independently of physical bit patterns.
use super::{ProjectionStorage, Reader, StoredShape, footprints::ProjectionLayout};
use crate::{
    abi::layout::{Layout, Shape, Tag, Variant},
    sema::members::{FieldId, VariantId},
    types::{ItemId, Kind, TypeId, aggregate::Definition},
};
#[cfg(test)]
#[path = "../../../../tests/invariants/enum_storage_facts.rs"]
mod tests;
pub(crate) struct StoredEnumeration<'a> {
    pub owner: ItemId,
    pub tag: Tag,
    pub variants: &'a [Variant],
    pub identities: &'a [VariantId],
}
impl<'a> Reader<'a> {
    pub(super) fn enumeration_shape(
        definition: &Definition,
        layout: &Layout,
    ) -> Result<StoredShape<'a>, &'static str> {
        let Some(expected) = definition.repr.tag else {
            return Ok(StoredShape::Unsupported);
        };
        let (_, bits) = expected
            .integer()
            .ok_or("stored enum tag must be an integer")?;
        let width = u64::from(bits / 8);
        let Shape::Aggregate { tag: Some(tag), .. } = layout.shape else {
            return Err("stored enum lacks its retained physical tag");
        };
        if tag.ty != expected {
            return Err("stored enum tag differs from its declared representation");
        }
        if tag
            .offset
            .checked_add(width)
            .is_none_or(|end| end > layout.size)
        {
            return Err("stored enum tag exceeds its physical bounds");
        }
        Ok(StoredShape::Enumeration { tag })
    }
    /// Borrow completed physical variants without assigning byte tags to ordinals.
    pub(crate) fn enum_variants(self, ty: TypeId) -> Result<StoredEnumeration<'a>, &'static str> {
        let StoredShape::Enumeration { tag } = self.storage_shape(ty)? else {
            return Err("constant enum byte storage requires an explicit integer tag");
        };
        let Some(Kind::Adt(owner, _)) = self.facts.types.kind(ty) else {
            return Err("stored enum lacks a nominal owner");
        };
        let layout = self
            .facts
            .layouts
            .get(ty)
            .ok_or("missing stored enum layout")?;
        let variants = self.facts.layouts.variants(layout);
        let identities = self.facts.members.variants(owner);
        if variants.len() != identities.len() || variants.is_empty() {
            return Err("stored enum variant count differs from its declarations");
        }
        Ok(StoredEnumeration {
            owner,
            tag,
            variants,
            identities,
        })
    }
    fn stored_variant(
        self,
        ty: TypeId,
        id: VariantId,
    ) -> Result<(StoredEnumeration<'a>, &'a Variant), &'static str> {
        let stored = self.enum_variants(ty)?;
        let member = self
            .facts
            .members
            .variants
            .get(id.0 as usize)
            .ok_or("missing stored enum variant identity")?;
        let ordinal = member.ordinal as usize;
        if member.owner != stored.owner || stored.identities.get(ordinal) != Some(&id) {
            return Err("stored enum variant has a different owner or ordinal");
        }
        let variant = stored
            .variants
            .get(ordinal)
            .ok_or("stored enum variant exceeds its layout")?;
        let fields = self
            .facts
            .types
            .list(variant.fields)
            .ok_or("missing stored enum fields")?;
        if member.end.checked_sub(member.start).map(u64::from) != Some(fields.len() as u64)
            || self.facts.layouts.offsets(variant).len() != fields.len()
        {
            return Err("stored enum fields differ from their declarations or layout");
        }
        Ok((stored, variant))
    }
    /// Borrow only the exact variant's retained physical children, without a value read.
    pub(crate) fn enum_variant_fields(
        self,
        ty: TypeId,
        id: VariantId,
    ) -> Result<(&'a [TypeId], &'a [u64]), &'static str> {
        let (_, variant) = self.stored_variant(ty, id)?;
        Ok((
            self.facts
                .types
                .list(variant.fields)
                .ok_or("missing stored enum fields")?,
            self.facts.layouts.offsets(variant),
        ))
    }
    /// Canonical declaration selection retains the variant-specific physical offset.
    pub(crate) fn enum_field_layout(
        self,
        ty: TypeId,
        field: FieldId,
    ) -> Result<ProjectionLayout, &'static str> {
        let id = self
            .facts
            .members
            .variant(field)
            .ok_or("missing stored enum field owner")?;
        let (fields, offsets) = self.enum_variant_fields(ty, id)?;
        let index = self
            .facts
            .members
            .ordinal(field)
            .ok_or("missing stored enum field ordinal")?;
        let member = self
            .facts
            .members
            .variants
            .get(id.0 as usize)
            .ok_or("missing stored enum field group")?;
        if member.start.checked_add(index) != Some(field.0) || field.0 >= member.end {
            return Err("stored enum field has a different ordinal");
        }
        let target = *fields
            .get(index as usize)
            .ok_or("stored enum field exceeds layout")?;
        if self.stored_child(ty, Some(id), u64::from(index))? != target {
            return Err("stored enum field differs from its checked type");
        }
        let offset = *offsets
            .get(index as usize)
            .ok_or("missing stored enum field offset")?;
        let extent = self
            .facts
            .layouts
            .get(target)
            .ok_or("missing stored enum field layout")?
            .size;
        let size = self
            .facts
            .layouts
            .get(ty)
            .ok_or("missing stored enum layout")?
            .size;
        if offset.checked_add(extent).is_none_or(|end| end > size) {
            return Err("stored enum field exceeds physical bounds");
        }
        Ok(ProjectionLayout {
            ty: target,
            offset,
            storage: ProjectionStorage::Product,
        })
    }
    /// The family cache checks all variants once; encoding still checks its actual ID.
    pub(crate) fn enum_variant_bits(self, ty: TypeId, id: VariantId) -> Result<u128, &'static str> {
        let (stored, variant) = self.stored_variant(ty, id)?;
        // Negative semantic discriminants retain target-width two's-complement bits.
        let (_, bits) = stored
            .tag
            .ty
            .integer()
            .ok_or("stored enum tag must be an integer")?;
        let mask = if bits == 128 {
            u128::MAX
        } else {
            (1_u128 << bits) - 1
        };
        Ok(variant.discriminant & mask)
    }
}
