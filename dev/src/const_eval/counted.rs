//! Initialized typed snapshots implement the first bounded counted-copy profile.
mod bytes;
mod fill;
mod spans;
#[cfg(test)]
#[path = "../../tests/invariants/counted_memory.rs"]
mod tests;
use super::{
    Constant, Failure, Machine,
    memory::{Range, RawAccess, RawAlignment, Resolved},
};
use crate::{sema::values::LocationId, support::intrinsics::Intrinsic, types::TypeId};
#[cfg(test)]
#[path = "../../tests/invariants/overlapping_memory.rs"]
mod overlapping_tests;
struct TransferSpan {
    address: Constant,
    ty: TypeId,
    root: LocationId,
    fields: Vec<u64>,
    range: Option<Range>,
    offset: u64,
    bytes: u64,
    image: Option<super::memory::ImagePath>,
}
const HEADER_SLOTS: usize = (2 * std::mem::size_of::<TransferSpan>()
    + std::mem::size_of::<Constant>())
.div_ceil(std::mem::size_of::<Constant>());
impl TransferSpan {
    fn place(&self) -> Resolved {
        Resolved {
            address: self.address,
            ty: self.ty,
            raw: Some(RawAccess {
                alignment: RawAlignment::Required,
                location: self.address.location(),
                ty: self.ty,
            }),
            union: None,
        }
    }
}
impl Machine<'_, '_> {
    pub(super) fn counted_copy(
        &mut self,
        intrinsic: Intrinsic,
        ty: TypeId,
        arguments: &[Constant],
    ) -> Result<Option<Constant>, Failure> {
        if !intrinsic.is_counted_memory() {
            return Err("constant counted copy has an invalid intrinsic identity".into());
        }
        let [source, destination, count] = arguments else {
            return Err("counted memory operation argument count mismatch".into());
        };
        let count = u64::try_from(super::values::integer(*count)?)
            .map_err(|_| "constant copy count exceeds usize")?;
        let layout = self.footprint(ty)?;
        let bytes = count
            .checked_mul(layout.size)
            .filter(|&n| i64::try_from(n).is_ok())
            .ok_or("constant copy byte count exceeds allocation size")?;
        let source = self.transfer_span(*source, ty, count, bytes, HEADER_SLOTS)?;
        let destination = self.transfer_span(
            *destination,
            ty,
            count,
            bytes,
            source.fields.len().saturating_add(HEADER_SLOTS + 2),
        )?;
        let retained = source
            .fields
            .len()
            .saturating_add(destination.fields.len())
            .saturating_add(HEADER_SLOTS + 4);
        self.reserve_cells(retained)?;
        let previous = self.slots;
        self.slots = previous
            .checked_add(retained)
            .ok_or("constant counted span retention overflow")?;
        let result = self.copy_spans(intrinsic, &source, &destination);
        self.slots = previous;
        result
    }
    fn copy_spans(
        &mut self,
        intrinsic: Intrinsic,
        source: &TransferSpan,
        destination: &TransferSpan,
    ) -> Result<Option<Constant>, Failure> {
        if source.bytes == 0 {
            return Ok(Some(self.evaluator.fields(&[], None)?));
        }
        if intrinsic == Intrinsic::CopyNonoverlapping
            && source.root == destination.root
            && source.offset < destination.offset + destination.bytes
            && destination.offset < source.offset + source.bytes
        {
            return Err("constant copy_nonoverlapping regions overlap".into());
        }
        self.check_raw_write_authority(destination.address, destination.root)?;
        if self.needs_byte_transfer(source, destination)? {
            return self.copy_byte_spans(source, destination);
        }
        let Some(snapshot) = self.transfer_snapshot(source)? else {
            return Ok(None);
        };
        if let Some(range) = destination.range {
            self.write_range(destination.root, &destination.fields, range, snapshot)?;
        } else {
            let value = self.stored_field(snapshot, 0)?;
            self.write_place(destination.place(), value)?;
        }
        Ok(Some(self.evaluator.fields(&[], None)?))
    }
    fn transfer_snapshot(&mut self, span: &TransferSpan) -> Result<Option<Constant>, Failure> {
        if span.range.is_some() {
            self.read_storage(
                span.root,
                &span.fields,
                span.range,
                span.ty,
                super::static_reads::ReadPurpose::Value,
            )
        } else {
            let place = span.place();
            if self.storage_target(place)?.field().is_some() {
                return Err(
                    "constant counted union-field byte transport is not implemented".into(),
                );
            }
            let Some(value) = self.read_storage(
                span.root,
                &span.fields,
                None,
                span.ty,
                super::static_reads::ReadPurpose::Value,
            )?
            else {
                return Ok(None);
            };
            Ok(Some(self.evaluator.repeat(value, 1)?))
        }
    }
}
