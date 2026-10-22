//! Enum byte tags retain declaration identities independently of physical bit patterns.
use super::{Reader, StoredShape};
use crate::{
    abi::layout::{Layout, Shape, Tag, Variant},
    sema::members::VariantId,
    types::{ItemId, Kind, Primitive, TypeId, aggregate::Definition},
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
        let Some(expected @ (Primitive::U8 | Primitive::I8)) = definition.repr.tag else {
            return Ok(StoredShape::Unsupported);
        };
        let Shape::Aggregate { tag: Some(tag), .. } = layout.shape else {
            return Err("stored enum lacks its retained physical tag");
        };
        if tag.ty != expected {
            return Err("stored enum tag differs from its declared representation");
        }
        if tag
            .offset
            .checked_add(1)
            .is_none_or(|end| end > layout.size)
        {
            return Err("stored enum tag exceeds its physical bounds");
        }
        Ok(StoredShape::Enumeration { tag })
    }
    /// Borrow completed physical variants without assigning byte tags to ordinals.
    pub(crate) fn enum_variants(self, ty: TypeId) -> Result<StoredEnumeration<'a>, &'static str> {
        let StoredShape::Enumeration { tag } = self.storage_shape(ty)? else {
            return Err("constant enum byte storage requires an explicit u8 or i8 tag");
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
    /// The family cache checks all variants once; encoding still checks its actual ID.
    pub(crate) fn enum_variant_bits(self, ty: TypeId, id: VariantId) -> Result<u8, &'static str> {
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
        if member.start != member.end
            || !self
                .facts
                .types
                .list(variant.fields)
                .ok_or("missing stored enum fields")?
                .is_empty()
            || !self.facts.layouts.offsets(variant).is_empty()
        {
            return Err("constant enum byte storage with payload fields is not implemented");
        }
        // Negative semantic discriminants retain target-width two's-complement bits.
        u8::try_from(variant.discriminant & 255).map_err(|_| "invalid stored enum tag width")
    }
}
