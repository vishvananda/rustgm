//! Canonical generic products are borrowed views, never reconstructed constant arenas.
use super::{Constant, Reader, View};
use crate::{
    sema::members::VariantId,
    types::{
        Kind, TypeId, aggregate,
        arguments::trees::{Aggregate, AggregateId},
    },
};
impl Reader<'_> {
    pub(super) fn canonical_view(self, id: AggregateId) -> Result<View, &'static str> {
        Ok(
            match self
                .facts
                .types
                .aggregate_constant(id)
                .ok_or("missing canonical const product")?
            {
                Aggregate::Fields {
                    ty,
                    variant,
                    values,
                } => View::Product {
                    length: values.len() as u64,
                    variant: self.canonical_variant(ty, variant)?,
                },
                Aggregate::Repeat {
                    element, length, ..
                } => View::Repeat {
                    value: Constant::argument(&self.facts.types, element)?
                        .ok_or("canonical repeat retains a parameter")?,
                    length,
                },
            },
        )
    }
    pub(super) fn canonical_field(
        self,
        id: AggregateId,
        index: u64,
    ) -> Result<Constant, &'static str> {
        let child = match self
            .facts
            .types
            .aggregate_constant(id)
            .ok_or("missing canonical const product")?
        {
            Aggregate::Fields { values, .. } => *usize::try_from(index)
                .ok()
                .and_then(|index| values.get(index))
                .ok_or("constant index is out of bounds")?,
            Aggregate::Repeat {
                element, length, ..
            } if index < length => element,
            Aggregate::Repeat { .. } => return Err("constant index is out of bounds"),
        };
        Constant::argument(&self.facts.types, child)?
            .ok_or("canonical const product retains a parameter")
    }
    fn canonical_variant(
        self,
        ty: TypeId,
        ordinal: Option<u32>,
    ) -> Result<Option<VariantId>, &'static str> {
        let Some(Kind::Adt(item, _)) = self.facts.types.kind(ty) else {
            return if ordinal.is_none() {
                Ok(None)
            } else {
                Err("canonical sequence has a nominal variant")
            };
        };
        let definition = self
            .facts
            .aggregates
            .get(item)
            .ok_or("canonical const nominal lacks its actual declaration")?;
        let index = match (definition.kind, ordinal) {
            (aggregate::Kind::Struct | aggregate::Kind::PhantomData, None) => 0,
            (aggregate::Kind::Enum, Some(index)) => index,
            _ => return Err("canonical const variant differs from its nominal kind"),
        };
        let id = *self
            .facts
            .members
            .variants(item)
            .get(index as usize)
            .ok_or("canonical const variant exceeds its actual owner")?;
        let actual = self
            .facts
            .members
            .variants
            .get(id.0 as usize)
            .ok_or("canonical const lost its source variant")?;
        if actual.owner != item || actual.ordinal != index {
            return Err("canonical const variant belongs to another actual declaration");
        }
        Ok(Some(id))
    }
}
