//! Checked unsized field selection forwards metadata without reading its contents.
use super::{Constant, Failure, Machine, Reader, Resolved, TypeId};
use crate::sema::members::Selection;

#[cfg(test)]
#[path = "../../../tests/invariants/dst_metadata.rs"]
mod tests;
impl Machine<'_, '_> {
    pub(super) fn field_projection(
        &mut self,
        place: Resolved,
        field: Selection,
        target: TypeId,
    ) -> Result<Constant, Failure> {
        self.charge(1)?;
        let index = match field {
            Selection::Tuple(index) => index,
            Selection::Nominal(field) => self
                .evaluator
                .facts()
                .members
                .ordinal(field)
                .ok_or("missing constant field ordinal")?,
        };
        self.project_raw_type(place)?;
        let metadata = Reader::new(self.evaluator.facts()).projected_metadata(
            place.ty,
            field,
            target,
            place.address.metadata(),
        )?;
        let stored = if let Some(metadata) = metadata {
            let reader = Reader::new(self.evaluator.facts());
            let actual = reader
                .location(
                    place
                        .address
                        .location()
                        .ok_or("tail projection lacks storage")?,
                )?
                .ty;
            reader.tail_storage(place.ty, actual, u64::from(index), target, metadata)?
        } else {
            target
        };
        let value = self
            .evaluator
            .projection(place.address, u64::from(index), stored)?;
        metadata.map_or(Ok(value), |metadata| value.with_metadata(metadata))
    }
}
