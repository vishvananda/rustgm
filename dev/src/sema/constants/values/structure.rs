//! Closed stored field types are read from published shapes without semantic queries.
use super::{Constant, Reader, View};
use crate::types::{Kind, TypeId, aggregate::ShapeReadiness};

pub enum FieldTypes<'a> {
    Sequence(&'a [TypeId]),
    Repeated { element: TypeId, length: u64 },
}
impl FieldTypes<'_> {
    #[must_use]
    pub fn get(&self, index: u64) -> Option<TypeId> {
        match self {
            Self::Sequence(fields) => usize::try_from(index)
                .ok()
                .and_then(|i| fields.get(i))
                .copied(),
            Self::Repeated { element, length } => (index < *length).then_some(*element),
        }
    }
    #[must_use]
    pub fn len(&self) -> u64 {
        match self {
            Self::Sequence(fields) => fields.len() as u64,
            Self::Repeated { length, .. } => *length,
        }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<'a> Reader<'a> {
    /// Read the active product's closed stored field types, preserving array compression.
    /// # Errors
    /// Rejects unpublished shapes, invalid variants and non-product types.
    pub fn field_types(
        self,
        mut ty: TypeId,
        value: Constant,
    ) -> Result<FieldTypes<'a>, &'static str> {
        if let Some(Kind::Closure { environment, .. }) = self.facts.types.kind(ty) {
            ty = environment;
        }
        let fields = match self
            .facts
            .types
            .kind(ty)
            .ok_or("missing stored field type")?
        {
            Kind::Array { element, length } => return Ok(FieldTypes::Repeated { element, length }),
            Kind::Tuple(fields) => fields,
            Kind::Adt(item, _) => {
                if self.facts.aggregates.get(item).is_some_and(|definition| {
                    definition.kind == crate::types::aggregate::Kind::Union
                }) {
                    return Err("union byte storage has no structural field view");
                }
                let ShapeReadiness::Ready(shape) =
                    self.facts.aggregates.try_shape(&self.facts.types, ty)?
                else {
                    return Err("constant view requires a completed aggregate shape");
                };
                let View::Product { variant, .. } = self.view(value)? else {
                    return Err("nominal constant lacks an active product");
                };
                let ordinal = if let Some(id) = variant {
                    let variant = self
                        .facts
                        .members
                        .variants
                        .get(id.0 as usize)
                        .ok_or("missing constant variant")?;
                    if variant.owner != item {
                        return Err("constant variant has a different owner");
                    }
                    variant.ordinal as usize
                } else if shape.variants().len() == 1 {
                    0
                } else {
                    return Err("constant enum lacks an active variant");
                };
                shape
                    .variants()
                    .nth(ordinal)
                    .ok_or("constant variant exceeds its shape")?
                    .fields
            }
            _ => return Err("constant view requires a closed product type"),
        };
        Ok(FieldTypes::Sequence(
            self.facts
                .types
                .list(fields)
                .ok_or("missing stored field list")?,
        ))
    }
}
