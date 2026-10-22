//! Raw byte transport is selected only when typed snapshots cannot preserve facts.
use super::{Constant, Failure, Machine, TransferSpan};
impl Machine<'_, '_> {
    pub(super) fn needs_byte_transfer(
        &self,
        source: &TransferSpan,
        destination: &TransferSpan,
    ) -> Result<bool, Failure> {
        Ok(self.reinterpreted_span(source)?
            || self.reinterpreted_span(destination)?
            || self.has_byte_root(source.root)
            || self.has_byte_root(destination.root)
            || source.image.is_some()
            || destination.image.is_some())
    }
    fn reinterpreted_span(&self, span: &TransferSpan) -> Result<bool, Failure> {
        let reader = crate::sema::values::Reader::new(self.evaluator.facts());
        let location = reader.location(
            span.address
                .location()
                .ok_or("constant byte span lacks provenance")?,
        )?;
        Ok(matches!(
            location.origin,
            crate::sema::values::Origin::ByteOffset { .. }
        ) || !reader.same_storage_type(location.ty, span.ty))
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
        } else if let Some(union) = source.image {
            let path = source
                .fields
                .get(..union.prefix)
                .ok_or("counted union prefix exceeds its retained path")?;
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
            self.snapshot_image_bytes(
                image,
                union.owner,
                source
                    .offset
                    .checked_sub(union.offset)
                    .ok_or("counted union image offset is reversed")?,
                source.bytes,
            )?
        } else if self.reinterpreted_span(source)? {
            let Some(snapshot) =
                self.snapshot_byte_access(source.root, source.offset, source.bytes, source.ty)?
            else {
                return Ok(None);
            };
            snapshot
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
