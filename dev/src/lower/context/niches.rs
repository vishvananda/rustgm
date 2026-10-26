//! The nullable ABI changes transport only; aggregate snapshots retain their own storage.
use super::Context;
use crate::{
    abi::layout::{Layout, Shape},
    types::{Kind, TypeId, aggregate::Representation},
};
impl Context<'_> {
    pub(super) fn pointer_object(
        &mut self,
        ty: TypeId,
        layout: &Layout,
    ) -> Result<bool, &'static str> {
        if layout.size != 8 {
            return Ok(false);
        }
        if matches!(layout.shape, Shape::Niche { .. }) {
            let encoding = self
                .layouts
                .niche(layout)?
                .ok_or("missing nullable ABI encoding")?;
            if encoding.owner != ty {
                return Err("nullable ABI encoding belongs to another type");
            }
            return Ok(true);
        }
        let Some(Kind::Adt(item, _)) = self.types.kind(ty) else {
            return Ok(false);
        };
        if self
            .aggregates
            .get(item)
            .is_some_and(|d| d.repr.representation == Representation::Transparent)
        {
            return self
                .layouts
                .pointer_carrier(self.types, self.aggregates, ty)
                .map(|carrier| carrier.is_some());
        }
        Ok(false)
    }
}
