//! Raw byte transport is selected only when typed snapshots cannot preserve facts.
use super::{Constant, Failure, Machine, TransferSpan};
impl Machine<'_, '_> {
    pub(super) fn needs_byte_transfer(
        &self,
        source: &TransferSpan,
        destination: &TransferSpan,
    ) -> Result<bool, Failure> {
        Ok(self.has_byte_root(source.root)
            || self.has_byte_root(destination.root)
            || self.storage_target(source.place())?.field().is_some())
    }
    pub(super) fn copy_byte_spans(
        &mut self,
        source: &TransferSpan,
        destination: &TransferSpan,
    ) -> Result<Option<Constant>, Failure> {
        let snapshot = if let Some(snapshot) =
            self.snapshot_byte_root(source.root, source.offset, source.bytes)?
        {
            snapshot
        } else if let Some(union) = self.storage_target(source.place())?.field() {
            let target = self.storage_target(source.place())?;
            let path = &source.fields[..target.path_len(source.fields.len())?];
            let Some(image) = self.read_storage(
                source.root,
                path,
                None,
                union.owner,
                crate::const_eval::static_reads::ReadPurpose::Value,
            )?
            else {
                return Ok(None);
            };
            self.snapshot_image_bytes(image, union.owner, source.bytes)?
        } else {
            let Some(value) = self.transfer_snapshot(source)? else {
                return Ok(None);
            };
            let stride = self.footprint(source.ty)?.size;
            let count = source
                .bytes
                .checked_div(stride)
                .ok_or("raw byte transfer lacks nonzero stride")?;
            self.snapshot_typed_bytes(source.ty, value, count, source.bytes)?
        };
        let retained = snapshot.retained_slots();
        self.reserve_cells(retained)?;
        let previous = self.slots;
        self.slots = previous
            .checked_add(retained)
            .ok_or("raw byte snapshot retention overflow")?;
        let result = self.commit_byte_snapshot(destination.root, destination.offset, &snapshot);
        self.slots = previous;
        result?;
        Ok(Some(self.evaluator.fields(&[], None)?))
    }
}
