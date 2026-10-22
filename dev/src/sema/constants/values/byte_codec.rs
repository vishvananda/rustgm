//! Primitive byte codecs reuse scalar validity without interpreting raw transfers.
use super::{Constant, Evaluator, Failure, Reader};
use crate::{
    sema::constants::images,
    types::{Kind, TypeId},
};
impl Reader<'_> {
    pub(crate) fn encode_primitive(
        self,
        ty: TypeId,
        value: Constant,
    ) -> Result<(u128, usize), Failure> {
        let Some(Kind::Primitive(primitive)) = self.facts.types.kind(ty) else {
            return Err("constant byte encoding requires a primitive type".into());
        };
        images::scalars::encode(primitive, value.as_scalar()?).map_err(Failure::Fault)
    }
    pub(crate) fn decode_primitive(
        self,
        ty: TypeId,
        bytes: &[u8],
        initialized: &[u64],
    ) -> Result<Constant, &'static str> {
        let Some(Kind::Primitive(primitive)) = self.facts.types.kind(ty) else {
            return Err("constant byte decoding requires a primitive type");
        };
        images::scalars::decode(primitive, bytes, initialized).map(Constant::scalar)
    }
}
impl Evaluator<'_> {
    pub(crate) fn union_snapshot(
        &mut self,
        ty: TypeId,
        bytes: &[u8],
        initialized: &[u64],
        limit: usize,
    ) -> Result<(Constant, usize), Failure> {
        // Revalidate the already prepared scalar family through its canonical field.
        let facts = self.facts();
        let family = facts
            .constant_images
            .as_ref()
            .and_then(|a| a.families.get(&ty))
            .ok_or("constant byte snapshot requires a prepared scalar union")?;
        if family.size != bytes.len() || initialized.len() != bytes.len().div_ceil(64) {
            return Err("constant byte snapshot differs from its union storage".into());
        }
        let work = bytes
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(initialized.len().saturating_mul(2)))
            .filter(|n| *n <= limit)
            .ok_or("constant union snapshot work budget exceeded")?;
        let arena = self
            .queries
            .typing
            .facts
            .constant_images
            .as_mut()
            .ok_or("missing constant image arena")?;
        arena.reserve_image(bytes.len())?;
        let value = arena.insert(images::Image {
            ty,
            bytes: bytes.to_vec().into_boxed_slice(),
            initialized: initialized.to_vec().into_boxed_slice(),
        })?;
        Ok((Constant(value), work))
    }
}
