//! One numeric span replaces intersecting provenance and initializes its bytes.
use super::{Failure, LocationId, Machine};
#[cfg(test)]
#[path = "../../../../tests/invariants/fill_storage.rs"]
mod tests;
impl Machine<'_, '_> {
    pub(in crate::const_eval) fn fill_byte_root(
        &mut self,
        root: LocationId,
        offset: u64,
        length: u64,
        byte: u8,
    ) -> Result<(), Failure> {
        let id = self.convert_byte_root(root)?;
        if self.byte_backing(id)?.pending_reads != 0 {
            return Err("constant byte storage mutated while a typed read was pending".into());
        }
        let start = usize::try_from(offset).map_err(|_| "byte fill offset exceeds host size")?;
        let length = usize::try_from(length).map_err(|_| "byte fill length exceeds host size")?;
        let end = start
            .checked_add(length)
            .filter(|&end| end <= self.byte_backing(id).map_or(0, |b| b.bytes.len()))
            .ok_or("byte fill exceeds backing bounds")?;
        // Charge both payload and touched mask words before either is modified.
        self.charge(length.saturating_add(end.div_ceil(64).saturating_sub(start / 64)))?;
        self.commit_root_fragments(id, offset, length as u64, &[])?;
        let destination = self
            .memory
            .bytes
            .roots
            .get_mut(id.0 as usize)
            .ok_or("missing byte fill backing")?;
        destination.bytes[start..end].fill(byte);
        initialize_mask(&mut destination.initialized, start, end);
        Ok(())
    }
}
fn initialize_mask(mask: &mut [u64], start: usize, end: usize) {
    if start == end {
        return;
    }
    let first = start / 64;
    let last = (end - 1) / 64;
    let left = u64::MAX << (start % 64);
    let right = u64::MAX >> (63 - (end - 1) % 64);
    if first == last {
        mask[first] |= left & right;
    } else {
        mask[first] |= left;
        mask[first + 1..last].fill(u64::MAX);
        mask[last] |= right;
    }
}
