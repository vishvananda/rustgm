//! Promotion roots stop at dereferences; evaluated projections demand their inputs.
use super::{Engine, Error, Evaluated, Key, Ready};
use crate::{
    body::ir::{Place, Projection, ProjectionId, ProjectionKind, ValueId},
    sema::{members::Selection, model::Scalar},
};
impl Engine<'_, '_> {
    pub(super) fn index_roots(&mut self) -> Result<(), Error> {
        for projection in &self.body.projections {
            self.charge(1)?;
            let root = if projection.kind.is_dereference() {
                None
            } else {
                self.root(projection.base)?
            };
            self.roots.push(root);
        }
        Ok(())
    }
    pub(super) fn root(&self, place: Place) -> Result<Option<ValueId>, Error> {
        match place {
            Place::Temporary(id) => Ok(Some(id)),
            Place::Projection(id) => self
                .roots
                .get(id.0 as usize)
                .copied()
                .ok_or_else(|| "missing earlier promotion place base".into()),
            Place::Local(_) | Place::Static(_) => Ok(None),
        }
    }
    pub(super) fn place_key(place: Place) -> Option<Key> {
        match place {
            Place::Temporary(id) => Some(Key::Value(id)),
            Place::Projection(id) => Some(Key::Projection(id)),
            Place::Local(_) | Place::Static(_) => None,
        }
    }
    pub(super) fn place_value(&self, place: Place) -> Result<Option<Ready>, Error> {
        Self::place_key(place).map_or(Ok(None), |key| self.ready(key))
    }
    pub(super) fn projection(&self, id: ProjectionId) -> Result<&Projection, Error> {
        self.body
            .projections
            .get(id.0 as usize)
            .ok_or_else(|| "missing promotion projection".into())
    }
    pub(super) fn reborrow_base(&self, place: Place) -> Result<Place, Error> {
        if let Place::Projection(id) = place {
            let projection = self.projection(id)?;
            if projection.kind.is_dereference() {
                return Ok(projection.base);
            }
        }
        Ok(place)
    }
    pub(super) fn borrow_value(
        &mut self,
        place: Place,
        ty: crate::types::TypeId,
    ) -> Result<Option<Evaluated>, Error> {
        let base = self.reborrow_base(place)?;
        let Some(value) = self.place_value(base)? else {
            return Ok(None);
        };
        let source = self
            .body
            .place_type(place)
            .ok_or("missing borrowed place type")?;
        Ok(Some(match value.value {
            Evaluated::Constant(value) => {
                let address = if base == place {
                    self.evaluator.borrow(value, source)?
                } else {
                    value
                };
                Evaluated::Constant(self.evaluator.adjust_reference(address, source, ty)?)
            }
            Evaluated::Deferred => Evaluated::Deferred,
        }))
    }
    pub(super) fn finish_projection(&self, id: ProjectionId) -> Result<Option<Evaluated>, Error> {
        let projection = self.projection(id)?;
        let Some(base) = self.place_value(projection.base)? else {
            return Ok(None);
        };
        let index = match projection.kind {
            ProjectionKind::CaptureBoundary { .. } => return Ok(Some(base.value)),
            ProjectionKind::ConstantIndex(index) => index,
            ProjectionKind::Field(Selection::Tuple(index)) => u64::from(index),
            ProjectionKind::Field(Selection::Nominal(field)) => u64::from(
                self.evaluator
                    .facts()
                    .members
                    .ordinal(field)
                    .ok_or("missing promotion field ordinal")?,
            ),
            ProjectionKind::Index { index, length } => {
                let Some(index) = self
                    .ready(Key::Value(index))?
                    .filter(|v| v.constant_assignment)
                else {
                    return Ok(None);
                };
                let Some(index) = index.value.constant() else {
                    return Ok(None);
                };
                let Scalar::Integer(index) = index.as_scalar()? else {
                    return Err("promotion index is not an integer".into());
                };
                if index >= u128::from(length) {
                    return Ok(None);
                }
                u64::try_from(index).map_err(|_| "promotion index exceeds target usize")?
            }
            ProjectionKind::IndexParameter { .. }
            | ProjectionKind::UnionField(_)
            | ProjectionKind::Dereference { .. }
            | ProjectionKind::RawDereference { .. }
            | ProjectionKind::Subarray { .. }
            | ProjectionKind::Subslice { .. }
            | ProjectionKind::FromEnd { .. }
            | ProjectionKind::SliceIndex { .. } => {
                return Ok(None);
            }
        };
        let Some(base) = base.value.constant() else {
            return Ok(Some(Evaluated::Deferred));
        };
        self.evaluator
            .field(base, index)
            .map(|value| Some(Evaluated::Constant(value)))
            // Dynamic bounds have already been proved above. A missing fixed
            // field now indicates inconsistent typed facts, not a runtime fault.
            .map_err(|error| Error::from(super::Failure::Invalid(error.message())))
    }
}
