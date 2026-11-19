//! Enum tag and niche queries use complete physical layouts.
use super::{Budget, Layouts, Shape, primitive};
use crate::types::{
    Kind, ListId, Primitive, Store, TypeId,
    aggregate::{Definitions, Representation},
};
impl Layouts {
    /// Select the active enum fields from the actual encoding of zero bytes.
    /// The selected fields still need their own validity checks.
    pub(crate) fn zero_enum_fields(
        &mut self,
        types: &Store,
        owner: TypeId,
    ) -> Result<Option<ListId>, &'static str> {
        let layout = *self
            .get(owner)
            .ok_or("zero assertion requires a completed enum layout")?;
        if !layout.inhabited {
            return Ok(None);
        }
        let variants = self.variants(&layout);
        if let Some(niche) = self.niche(&layout)? {
            if niche.owner != owner {
                return Err("zero assertion's nullable layout belongs to another owner");
            }
            let ordinal = if niche.carrier.empty_bits(types)? == 0 {
                niche.empty
            } else {
                niche.payload
            };
            return variants
                .get(ordinal as usize)
                .map(|variant| Some(variant.fields))
                .ok_or("zero assertion lost its selected nullable variant");
        }
        match (layout.shape, variants) {
            (Shape::Aggregate { tag: Some(_), .. }, [first, ..]) if first.discriminant == 0 => {
                Ok(Some(first.fields))
            }
            (Shape::Aggregate { tag: Some(_), .. }, variants) => {
                let count = variants.len();
                Budget::new(1_000_000)
                    .charge(&mut self.steps, count)
                    .map_err(|_| "zero assertion exceeds its enum layout work budget")?;
                Ok(self
                    .variants(&layout)
                    .iter()
                    .find(|variant| variant.discriminant == 0)
                    .map(|variant| variant.fields))
            }
            (Shape::Aggregate { tag: None, .. }, [variant]) => Ok(Some(variant.fields)),
            _ => Ok(None),
        }
    }

    pub(crate) fn fieldless_enum_scalar(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        owner: TypeId,
    ) -> Result<Option<TypeId>, &'static str> {
        let Some(Kind::Adt(item, _)) = types.kind(owner) else {
            return Ok(None);
        };
        let definition = definitions
            .get(item)
            .ok_or("enum scalar declaration is missing")?;
        let repr = definition.repr;
        if definition.kind != crate::types::aggregate::Kind::Enum
            || !matches!(
                (repr.representation, repr.tag),
                (Representation::Rust, Some(_)) | (Representation::C, _)
            )
            || repr.packed.is_some()
            || repr.align.is_some()
        {
            return Ok(None);
        }
        let shape = definitions.shape(types, owner)?;
        let count = shape.variants().len();
        let mut budget = Budget::new(1_000_000);
        budget
            .charge(&mut self.steps, count.saturating_mul(2).saturating_add(8))
            .map_err(|_| "enum scalar query exceeds its existing layout budget")?;
        let layout = *self
            .get(owner)
            .ok_or("enum scalar layout is not prepared")?;
        let Shape::Aggregate {
            tag: Some(actual), ..
        } = layout.shape
        else {
            return Ok(None);
        };
        let tag = repr.tag.unwrap_or(actual.ty);
        if tag.integer().is_none()
            || (repr.tag.is_none()
                && !matches!(
                    tag,
                    Primitive::I32 | Primitive::U32 | Primitive::I64 | Primitive::U64
                ))
        {
            return Err("enum scalar tag is not its actual integer representation");
        }
        if actual.ty != tag || actual.offset != 0 {
            return Err("enum scalar layout changed its actual declared tag");
        }
        let variants = self.variants(&layout);
        if variants.len() != count {
            return Err("enum scalar layout changed its actual variant coverage");
        }
        if count == 0 {
            return Ok(None);
        }
        for (declared, actual) in shape.variants().zip(variants) {
            let declared_fields = types
                .list(declared.fields)
                .ok_or("enum scalar source fields are missing")?;
            let actual_fields = types
                .list(actual.fields)
                .ok_or("enum scalar physical fields are missing")?;
            if !declared_fields.is_empty() || !actual_fields.is_empty() {
                return Ok(None);
            }
            if !self.offsets(actual).is_empty() || declared.discriminant != actual.discriminant {
                return Err("enum scalar layout differs from its actual fieldless variants");
            }
        }
        let storage = primitive(tag)?;
        if layout.size != storage.size || layout.align != storage.align || !layout.inhabited {
            return Ok(None);
        }
        Ok(Some(TypeId::primitive(tag)))
    }
}
