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
        if let Selection::Nominal(field) = field
            && let Some(crate::types::Kind::Adt(owner, _)) =
                self.evaluator.facts().types.kind(place.ty)
            && self
                .evaluator
                .facts()
                .aggregates
                .get(owner)
                .is_some_and(|definition| {
                    definition.kind == crate::types::aggregate::Kind::Enum
                        && (definition.repr.tag.is_some()
                            || definition.repr.representation
                                == crate::types::aggregate::Representation::C)
                })
        {
            self.footprint(place.ty)?;
            let selected =
                Reader::new(self.evaluator.facts()).enum_field_layout(place.ty, field)?;
            if selected.ty != target {
                return Err("constant enum field differs from its checked type".into());
            }
            return self.byte_address(place, selected.offset, target);
        }
        if self.byte_projecting(place)? {
            if !matches!(
                (self.evaluator.facts().types.kind(place.ty), field),
                (Some(crate::types::Kind::Tuple(_)), Selection::Tuple(_))
                    | (Some(crate::types::Kind::Adt(_, _)), Selection::Nominal(_))
            ) {
                return Err("constant byte field has a different projection capability".into());
            }
            let selected = Reader::new(self.evaluator.facts())
                .projection_layout(place.ty, u64::from(index))?;
            if selected.ty != target
                || !matches!(
                    selected.storage,
                    crate::sema::values::ProjectionStorage::Product
                )
            {
                return Err(
                    "constant byte field differs from its checked type or capability".into(),
                );
            }
            if let Selection::Nominal(field) = field {
                let members = &self.evaluator.facts().members;
                let variant = members
                    .variant(field)
                    .ok_or("constant byte field lacks owner")?;
                let group = members
                    .variants
                    .get(variant.0 as usize)
                    .ok_or("constant byte field lacks group")?;
                if !matches!(self.evaluator.facts().types.kind(place.ty),Some(crate::types::Kind::Adt(owner,_)) if owner == group.owner)
                    || group.start.checked_add(index) != Some(field.0)
                    || field.0 >= group.end
                {
                    return Err("constant byte field has a different owner or ordinal".into());
                }
            }
            return self.byte_address(place, selected.offset, target);
        }
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
