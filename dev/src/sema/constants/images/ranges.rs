//! Owned array ranges copy one checked span and preserve intersected word identities.
use super::{Id, Image, Images, TypeId, Value};
impl Images {
    fn selected_fragments(
        &self,
        id: Id,
        from: u64,
        length: u64,
    ) -> Result<std::ops::Range<usize>, &'static str> {
        let image = self.get(id)?;
        let end = from
            .checked_add(length)
            .filter(|&end| end <= image.bytes.len() as u64)
            .ok_or("constant image range is out of bounds")?;
        if length == 0 {
            return Ok(0..0);
        }
        let first = image
            .fragments
            .partition_point(|f| f.offset + u64::from(f.length) <= from);
        let stop = image.fragments.partition_point(|f| f.offset < end);
        Ok(first..stop)
    }
    pub(in crate::sema::constants) fn range_work(
        &self,
        id: Id,
        from: u64,
        length: u64,
    ) -> Result<usize, &'static str> {
        let selected = self.selected_fragments(id, from, length)?;
        let count = self.get(id)?.fragments.len();
        let height = count.checked_ilog2().map_or(1, |n| n as usize + 2);
        usize::try_from(length)
            .ok()
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| n.checked_add(selected.len().saturating_mul(16)))
            .and_then(|n| n.checked_add(height.saturating_mul(32)))
            .ok_or("constant image range work overflow")
    }
    /// Caller supplies a checked canonical array type and pays `range_work` before allocation.
    pub(in crate::sema::constants) fn array_range(
        &mut self,
        id: Id,
        ty: TypeId,
        from: u64,
        length: u64,
    ) -> Result<Value, &'static str> {
        let selected = self.selected_fragments(id, from, length)?;
        let size = usize::try_from(length).map_err(|_| "constant image range exceeds host size")?;
        self.reserve_array_fragments(size, selected.len())?;
        let image = self.get(id)?;
        let start = usize::try_from(from).map_err(|_| "constant image range exceeds host size")?;
        let end = from + length;
        let mut initialized = vec![0; size.div_ceil(64)];
        for index in 0..size {
            let absolute = start + index;
            if image.initialized[absolute / 64] & (1u64 << (absolute % 64)) == 0 {
                return Err("constant byte array read uses uninitialized bytes");
            }
            initialized[index / 64] |= 1u64 << (index % 64);
        }
        let bytes = image.bytes[start..start + size].to_vec().into_boxed_slice();
        let mut fragments = Vec::with_capacity(selected.len());
        for fragment in &image.fragments[selected] {
            fragments.push(fragment.clipped(from, end)?);
        }
        self.insert_array(Image {
            ty,
            bytes,
            initialized: initialized.into_boxed_slice(),
            fragments: fragments.into_boxed_slice(),
        })
    }
}
