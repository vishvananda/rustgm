//! Existing physical enum decoders read only tag or niche words.
use super::{Constant, Failure, Machine, Reader, TypeId, continuations::Source};
use crate::{const_eval::memory::Read, sema::{members::VariantId, values::LocationId}};

impl Machine<'_, '_> {
    pub(in crate::const_eval) fn read_byte_variant_access(&mut self, root: LocationId, offset: u64, ty: TypeId)
        -> Result<Read<VariantId>, Failure> {
        let Some(id) = self.prepare_byte_access(root, ty, crate::const_eval::static_reads::ReadPurpose::Value)? else {
            return Ok(Read::Pending);
        };
        self.decode_byte_variant(Source::Backing(id), offset, ty).map(Read::Value)
    }

    pub(in crate::const_eval) fn image_active_variant(&mut self, value: Constant, ty: TypeId, path: &[u64])
        -> Result<VariantId, Failure> {
        let (actual, offset) = self.storage_offset(ty, path)?;
        Reader::new(self.evaluator.facts()).storage_shape(actual)?;
        self.decode_byte_variant(Source::Image(value), offset, actual)
    }
}
