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
        self.field_projection_for::<false>(place, field, target)
    }
    pub(super) fn field_projection_for<const ADDRESS: bool>(
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
        if let Some(address) = self.enum_field_projection::<ADDRESS>(place, field, target)? {
            return Ok(address);
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
    fn enum_field_projection<const ADDRESS: bool>(
        &mut self,
        place: Resolved,
        field: Selection,
        target: TypeId,
    ) -> Result<Option<Constant>, Failure> {
        use crate::types::{
            Kind,
            aggregate::{Kind as AggregateKind, Representation},
        };
        let (Selection::Nominal(field), Some(Kind::Adt(owner, _))) =
            (field, self.evaluator.facts().types.kind(place.ty))
        else {
            return Ok(None);
        };
        let Some(definition) = self
            .evaluator
            .facts()
            .aggregates
            .get(owner)
            .filter(|definition| definition.kind == AggregateKind::Enum)
        else {
            return Ok(None);
        };
        let explicit =
            definition.repr.tag.is_some() || definition.repr.representation == Representation::C;
        let ordinary =
            definition.repr.tag.is_none() && definition.repr.representation == Representation::Rust;
        let tagless = definition.repr.tag.is_none()
            && definition.repr.representation == Representation::Rust
            && self.evaluator.facts().members.variants(owner).len() == 1;
        let nullable = self
            .evaluator
            .facts()
            .aggregates
            .nullable_enum(owner)
            .is_some();
        if !explicit && !ordinary && !nullable {
            return Ok(None);
        }
        self.footprint(place.ty)?;
        let shape = self
            .evaluator
            .facts()
            .layouts
            .get(place.ty)
            .ok_or("missing prepared enum field layout")?
            .shape;
        let tagged = matches!(
            shape,
            crate::abi::layout::Shape::Aggregate { tag: Some(_), .. }
        );
        let niche = matches!(shape, crate::abi::layout::Shape::Niche { .. });
        if !tagged && !tagless && !niche {
            return Ok(None);
        }
        let selected = Reader::new(self.evaluator.facts()).enum_field_layout(place.ty, field)?;
        if selected.ty != target {
            return Err(if tagged {
                "constant enum field differs from its checked type"
            } else {
                "constant tagless enum field differs from its checked type"
            }
            .into());
        }
        if tagged && ordinary && !explicit && !self.physical_default_field::<ADDRESS>(place)? {
            return Ok(None);
        }
        if tagged || niche {
            self.byte_address(place, selected.offset, target).map(Some)
        } else {
            Ok(None)
        }
    }
    fn physical_default_field<const ADDRESS: bool>(
        &mut self,
        place: Resolved,
    ) -> Result<bool, Failure> {
        let physical = ADDRESS || place.raw.is_some() || self.byte_projecting(place)?;
        let images = Reader::new(self.evaluator.facts()).has_image_storage();
        if !physical && !self.has_byte_roots() && !images {
            return Ok(false);
        }
        let path = self.path(place.address)?;
        let root = path.root;
        if self.byte_root(root).is_some() || images && self.path_has_image(root, &path.fields)? {
            return Ok(true);
        }
        let reader = Reader::new(self.evaluator.facts());
        let stored = reader.location(root)?;
        let persistent = matches!(stored.origin, crate::sema::values::Origin::Static(_));
        if !physical && !persistent {
            return Ok(false);
        }
        self.byte_family(stored.ty)
    }
}
