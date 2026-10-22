//! Symbolic carrier words choose the payload without manufacturing numeric pointer bits.
use super::{Failure, Machine, Reader, TypeId, continuations::Source};
use crate::sema::members::VariantId;
#[cfg(test)]
#[path = "../../../../tests/invariants/nullable_bytes.rs"]
mod tests;
impl Machine<'_, '_> {
    pub(super) fn decode_nullable_variant(
        &mut self,
        source: Source,
        offset: u64,
        ty: TypeId,
    ) -> Result<VariantId, Failure> {
        let stored = Reader::new(self.evaluator.facts()).enum_niche(ty)?;
        let (empty, payload, delta) = (stored.empty, stored.payload, stored.encoding.offset);
        let offset = offset
            .checked_add(delta)
            .ok_or("nullable word read offset overflow")?;
        let start = usize::try_from(offset).map_err(|_| "nullable word exceeds host size")?;
        let end = start
            .checked_add(8)
            .ok_or("nullable word extent overflow")?;
        self.charge(8)?;
        let data = self.byte_data(source)?;
        let bytes = data
            .bytes
            .get(start..end)
            .ok_or("nullable word read exceeds bounds")?;
        if (start..end).any(|index| !data.initialized(index)) {
            return Err("nullable carrier word is not fully initialized".into());
        }
        let zero = bytes.iter().all(|&byte| byte == 0);
        let symbolic = self.source_has_fragments(source, offset, 8)?;
        Ok(if zero && !symbolic { empty } else { payload })
    }
}
