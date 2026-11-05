//! Explicit fieldless enum tags supply storage, without scalar validity promises.
use super::{Budget, Layouts, Shape, primitive};
use crate::types::{
    Kind, Store, TypeId,
    aggregate::{Definitions, Representation},
};
impl Layouts {
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
        let Some(tag) = definition.repr.tag else {
            return Ok(None);
        };
        if definition.kind != crate::types::aggregate::Kind::Enum
            || !matches!(
                definition.repr.representation,
                Representation::Rust | Representation::C
            )
            || definition.repr.packed.is_some()
            || definition.repr.align.is_some()
        {
            return Ok(None);
        }
        if tag.integer().is_none() {
            return Err("enum scalar tag is not its actual integer representation");
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
