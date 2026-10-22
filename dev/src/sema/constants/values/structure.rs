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
    /// Physical reference-cell identity ignores only its outer lifetime.
    /// This establishes no source equality, conversion or lifetime permission.
    pub(crate) fn same_storage_type(self, left: TypeId, right: TypeId) -> bool {
        left == right
            || matches!((self.facts.types.kind(left), self.facts.types.kind(right)),
            (Some(Kind::Reference { mutable: a, referent: x, .. }),
             Some(Kind::Reference { mutable: b, referent: y, .. }))
                if a == b && x == y)
    }
    /// Read the active product's closed stored field types, preserving array compression.
    /// # Errors
    /// Rejects unpublished shapes, invalid variants and non-product types.
    pub fn field_types(
        self,
        mut ty: TypeId,
        value: Constant,
    ) -> Result<FieldTypes<'a>, &'static str> {
        ty = self.storage_type(ty)?;
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
            Kind::Slice(element) => {
                let length = match self.view(value)? {
                    View::Product {
                        length,
                        variant: None,
                    }
                    | View::Repeat { length, .. }
                    | View::ByteRange { length, .. } => length,
                    View::Bytes(text) => self.bytes(text)?.len() as u64,
                    _ => return Err("constant slice lacks completed sequence storage"),
                };
                return Ok(FieldTypes::Repeated { element, length });
            }
            Kind::Tuple(fields) => fields,
            Kind::Adt(item, _) => {
                if self.facts.aggregates.get(item).is_some_and(|definition| {
                    definition.kind == crate::types::aggregate::Kind::Union
                }) {
                    return Err("union byte storage has no structural field view");
                }
                let View::Product { variant, .. } = self.view(value)? else {
                    return Err("nominal constant lacks an active product");
                };
                self.nominal_stored_fields(ty, item, variant)?
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
    fn nominal_stored_fields(
        self,
        ty: TypeId,
        item: crate::types::ItemId,
        selected: Option<crate::sema::members::VariantId>,
    ) -> Result<crate::types::ListId, &'static str> {
        if self
            .facts
            .aggregates
            .get(item)
            .is_some_and(|definition| definition.kind == crate::types::aggregate::Kind::Union)
        {
            return Err("union byte storage has no structural field view");
        }
        let ShapeReadiness::Ready(shape) =
            self.facts.aggregates.try_shape(&self.facts.types, ty)?
        else {
            return Err("constant view requires a completed aggregate shape");
        };
        let ordinal = if let Some(id) = selected {
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
        Ok(shape
            .variants()
            .nth(ordinal)
            .ok_or("constant variant exceeds its shape")?
            .fields)
    }
    /// Select retained child types without manufacturing a product to inspect.
    pub(crate) fn stored_child(
        self,
        mut ty: TypeId,
        variant: Option<crate::sema::members::VariantId>,
        index: u64,
    ) -> Result<TypeId, &'static str> {
        if let Some(Kind::Closure { environment, .. }) = self.facts.types.kind(ty) {
            ty = environment;
        }
        let fields = match self
            .facts
            .types
            .kind(ty)
            .ok_or("missing stored child owner type")?
        {
            Kind::Array { element, length } if variant.is_none() && index < length => {
                return Ok(element);
            }
            Kind::Slice(element) if variant.is_none() => return Ok(element),
            Kind::Tuple(fields) if variant.is_none() => fields,
            Kind::Adt(item, _) => self.nominal_stored_fields(ty, item, variant)?,
            _ => return Err("stored child requires a matching product type and variant"),
        };
        self.facts
            .types
            .list(fields)
            .and_then(|fields| {
                usize::try_from(index)
                    .ok()
                    .and_then(|index| fields.get(index))
            })
            .copied()
            .ok_or("stored child ordinal exceeds its owner")
    }
}

impl<'a> Reader<'a> {
    /// Borrow the selected immutable byte payload without copying or scalar expansion.
    /// # Errors
    /// Rejects byte-range recipes outside their canonical literal storage.
    pub fn byte_range(
        self,
        text: crate::sema::text::Id,
        from: u64,
        length: u64,
    ) -> Result<&'a [u8], &'static str> {
        let start = usize::try_from(from).map_err(|_| "constant byte range exceeds host size")?;
        let end = from
            .checked_add(length)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("constant byte range exceeds host size")?;
        self.bytes(text)?
            .get(start..end)
            .ok_or("constant byte range exceeds storage")
    }
    /// Borrow a typed byte sequence without expanding one Constant per byte.
    /// # Errors
    /// Rejects non-byte storage, inconsistent lengths and invalid recipe bounds.
    pub(crate) fn stored_bytes(
        self,
        ty: TypeId,
        value: Constant,
    ) -> Result<&'a [u8], &'static str> {
        let bytes = match self.view(value)? {
            View::Bytes(text) => self.bytes(text)?,
            View::ByteRange { text, from, length } => self.byte_range(text, from, length)?,
            _ => return Err("constant byte storage requires a byte recipe"),
        };
        match self.facts.types.kind(ty) {
            Some(Kind::Array { element, length })
                if element == TypeId::primitive(crate::types::Primitive::U8)
                    && length == bytes.len() as u64 => {}
            Some(Kind::Slice(element))
                if element == TypeId::primitive(crate::types::Primitive::U8) => {}
            _ => return Err("constant byte range differs from its stored type"),
        }
        Ok(bytes)
    }
}
