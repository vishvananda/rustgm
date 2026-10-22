//! Stored products retain exact selected variants and canonical capture counts.
use super::{Constant, Failure, Machine, Reader, TypeId};
use crate::{
    sema::values::{StoredShape, View},
    types::Kind,
};

impl Machine<'_, '_> {
    pub(super) fn check_product_value(&self, ty: TypeId, value: Constant) -> Result<(), Failure> {
        let reader = Reader::new(self.evaluator.facts());
        let closure = matches!(
            self.evaluator.facts().types.kind(ty),
            Some(Kind::Closure { .. })
        );
        let enumeration = matches!(self.evaluator.facts().types.kind(ty), Some(Kind::Adt(owner,_))
            if self.evaluator.facts().aggregates.get(owner).is_some_and(|d| d.kind == crate::types::aggregate::Kind::Enum));
        if !closure && !enumeration {
            return Ok(());
        }
        let StoredShape::Product {
            variant: expected,
            fields,
            ..
        } = reader.storage_shape(ty)?
        else {
            return Ok(());
        };
        if !matches!(reader.view(value)?, View::Product { length, variant }
            if variant == expected && length == fields.len() as u64)
        {
            return Err(if closure {
                "closure encoding requires its exact capture product and fields"
            } else {
                "tagless enum encoding requires its exact sole variant and fields"
            }
            .into());
        }
        Ok(())
    }
}
