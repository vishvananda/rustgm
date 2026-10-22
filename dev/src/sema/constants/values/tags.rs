//! Enum reads expose only canonical tags published after dependency completion.
use super::{Primitive, Reader, VariantId};

impl Reader<'_> {
    /// Read an exact variant's completed representation and scalar value.
    /// # Errors
    /// Rejects malformed or non-enum members and missing canonical representations.
    pub fn discriminant(self, id: VariantId) -> Result<Option<(Primitive, u128)>, &'static str> {
        let member = self
            .facts
            .members
            .variants
            .get(id.0 as usize)
            .ok_or("missing constant variant")?;
        let shape = self
            .facts
            .aggregates
            .get(member.owner)
            .ok_or("missing enum representation")?;
        if shape.kind != crate::types::aggregate::Kind::Enum {
            return Err("constant variant does not belong to an enum");
        }
        if !self
            .facts
            .aggregates
            .discriminant_ready(member.owner, member.ordinal)
        {
            return Ok(None);
        }
        let variant = self
            .facts
            .aggregates
            .variants(shape)
            .get(member.ordinal as usize)
            .ok_or("missing enum discriminant")?;
        Ok(Some((
            shape.repr.tag.unwrap_or(Primitive::Isize),
            variant.discriminant,
        )))
    }
}
