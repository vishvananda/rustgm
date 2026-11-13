//! Nullable storage witnesses preserve full payload types and canonical variant identities.
use super::{Reader, footprints::ProjectionLayout};
use crate::{
    abi::layout::Niche,
    sema::members::{FieldId, VariantId},
    types::{Kind, TypeId, aggregate::Companion},
};
#[cfg(test)]
#[path = "../../../../tests/invariants/nullable_storage.rs"]
mod tests;
pub(crate) struct StoredNiche<'a> {
    pub encoding: &'a Niche,
    pub empty: VariantId,
    pub payload: VariantId,
}
impl<'a> Reader<'a> {
    pub(crate) fn enum_niche(self, ty: TypeId) -> Result<StoredNiche<'a>, &'static str> {
        let Some(Kind::Adt(owner, _)) = self.facts.types.kind(ty) else {
            return Err("nullable storage lacks a nominal owner");
        };
        let policy = self
            .facts
            .aggregates
            .nullable_enum(owner)
            .ok_or("nullable storage lacks its canonical policy")?;
        let layout = self
            .facts
            .layouts
            .get(ty)
            .ok_or("missing nullable storage layout")?;
        let encoding = self
            .facts
            .layouts
            .niche(layout)?
            .ok_or("missing nullable storage encoding")?;
        if encoding.owner != ty
            || !policy.admits(encoding.empty, encoding.payload)
            || (matches!(
                encoding.carrier,
                crate::abi::layout::Carrier::Boolean { .. }
            ) && !policy.bool_niche)
            || encoding.field != policy.field
            || encoding.offset != 0
            || (encoding.carrier.is_pointer()
                && self.facts.layouts.retained_pointer_carrier(encoding.ty)
                    != Some(encoding.carrier.leaf()))
            || self.facts.layouts.retained_carrier(encoding.ty) != Some(encoding.carrier)
            || self
                .facts
                .layouts
                .get(encoding.ty)
                .is_none_or(|payload| payload.size != layout.size || payload.align != layout.align)
        {
            return Err("nullable storage differs from its canonical policy or carrier");
        }
        let identities = self.facts.members.variants(owner);
        if identities.len() != 2 {
            return Err("nullable storage has a different variant count");
        }
        let empty = *identities
            .get(encoding.empty as usize)
            .ok_or("missing nullable empty identity")?;
        let payload = *identities
            .get(encoding.payload as usize)
            .ok_or("missing nullable payload identity")?;
        let (empty_fields, empty_offsets) = self.enum_variant_fields(ty, empty)?;
        let (fields, offsets) = self.enum_variant_fields(ty, payload)?;
        // A zero-sized companion's one field shares the carrier's address.
        let companion = match policy.companion {
            Companion::Fieldless => 0,
            Companion::ZeroSized => 1,
        };
        if empty_fields.len() != companion
            || empty_offsets
                .iter()
                .any(|&offset| offset != encoding.offset)
            || empty_fields.iter().any(|&field| {
                self.facts
                    .layouts
                    .get(field)
                    .is_none_or(|companion| companion.size != 0)
            })
            || fields != [encoding.ty]
            || offsets != [encoding.offset]
        {
            return Err("nullable storage differs from its exact variant fields");
        }
        Ok(StoredNiche {
            encoding,
            empty,
            payload,
        })
    }
    pub(super) fn niche_projection(
        self,
        ty: TypeId,
        index: u64,
    ) -> Result<ProjectionLayout, &'static str> {
        let stored = self.enum_niche(ty)?;
        if index != u64::from(stored.encoding.field) {
            return Err("nullable field index is out of bounds");
        }
        let member = self
            .facts
            .members
            .variants
            .get(stored.payload.0 as usize)
            .ok_or("missing nullable field owner")?;
        self.enum_field_layout(ty, FieldId(member.start))
    }
}
