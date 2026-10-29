//! The nullable ABI changes transport only; aggregate snapshots retain their own storage.
use super::Context;
use crate::{
    abi::layout::{Layout, Shape},
    types::{Kind, TypeId, aggregate::Representation},
};
impl Context<'_> {
    pub(super) fn carrier_type(
        &self,
        carrier: crate::abi::layout::Carrier,
        bytes: u64,
    ) -> Result<Option<crate::lowir::vocabulary::Type>, &'static str> {
        if carrier.is_pointer() && bytes != 8 {
            return Ok(None);
        }
        super::representation(self.types, carrier.leaf()).map(Some)
    }

    pub(super) fn scalar_object(
        &mut self,
        ty: TypeId,
        layout: &Layout,
    ) -> Result<Option<crate::lowir::vocabulary::Type>, &'static str> {
        if matches!(layout.shape, Shape::Niche { .. }) {
            let encoding = self
                .layouts
                .niche(layout)?
                .ok_or("missing nullable ABI encoding")?;
            if encoding.owner != ty {
                return Err("nullable ABI encoding belongs to another type");
            }
            return self.carrier_type(encoding.carrier, layout.size);
        }
        let Some(Kind::Adt(item, _)) = self.types.kind(ty) else {
            return Ok(None);
        };
        if self
            .aggregates
            .get(item)
            .is_some_and(|d| d.repr.representation == Representation::Transparent)
        {
            return self
                .layouts
                .scalar_carrier(self.types, self.aggregates, ty)?
                .map(|carrier| self.carrier_type(carrier, layout.size))
                .transpose()
                .map(Option::flatten);
        }
        Ok(None)
    }
}
#[cfg(test)]
#[path = "../../../tests/invariants/integer_source.rs"]
mod tests;
