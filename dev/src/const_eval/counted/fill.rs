//! A byte fill validates one destination span and updates its backing once.
use super::{Constant, Failure, Machine, TransferSpan};
use crate::types::TypeId;
#[cfg(test)]
#[path = "../../../tests/invariants/fill_memory.rs"]
mod tests;
const RETAINED: usize =
    (size_of::<TransferSpan>() + size_of::<Constant>()).div_ceil(size_of::<Constant>());
impl Machine<'_, '_> {
    pub(in crate::const_eval) fn counted_fill(
        &mut self,
        ty: TypeId,
        arguments: &[Constant],
    ) -> Result<Option<Constant>, Failure> {
        let [destination, byte, count] = arguments else {
            return Err("byte fill argument count mismatch".into());
        };
        let byte = u8::try_from(super::super::values::integer(*byte)?)
            .map_err(|_| "constant fill value exceeds u8")?;
        let count = u64::try_from(super::super::values::integer(*count)?)
            .map_err(|_| "constant fill count exceeds usize")?;
        let bytes = count
            .checked_mul(self.footprint(ty)?.size)
            .filter(|&n| i64::try_from(n).is_ok())
            .ok_or("constant fill byte count exceeds allocation size")?;
        // Even an empty fill retains address/type/alignment obligations. Only its
        // memory access, liveness and write authority are absent.
        let span = self.transfer_span(*destination, ty, count, bytes, RETAINED)?;
        let retained = RETAINED.saturating_add(span.fields.len());
        self.reserve_cells(retained)?;
        let previous = self.slots;
        self.slots = previous
            .checked_add(retained)
            .ok_or("constant fill span retention overflow")?;
        let result = (|| {
            if bytes != 0 {
                self.check_raw_write_authority(span.address, span.root)?;
                self.fill_byte_root(span.root, span.offset, bytes, byte)?;
            }
            Ok(Some(self.evaluator.fields(&[], None)?))
        })();
        self.slots = previous;
        result
    }
}
