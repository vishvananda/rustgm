//! Indexed reads preserve compressed immutable products and original storage identity.
use super::{Cell, Constant, Failure, Frame, LocationId, Machine, ProjectionKind, Read, Resolved};

impl Machine<'_, '_> {
    pub(super) fn read_cell(
        &mut self,
        root: LocationId,
        path: &[u64],
        purpose: crate::const_eval::static_reads::ReadPurpose,
    ) -> Result<Read<Constant>, Failure> {
        if let Some(value) = self.byte_read(root, path, purpose)? {
            return Ok(value);
        }
        let (cell, ty, rest) = self.read_cell_selection(root, path)?;
        if let Cell::Frozen(value) = self.memory.cells[cell] {
            return self.read_frozen(value, ty, &path[rest..], None, purpose);
        }
        self.freeze(cell, purpose)
    }
    pub(super) fn read_cell_selection(
        &mut self,
        root: LocationId,
        path: &[u64],
    ) -> Result<(usize, crate::types::TypeId, usize), Failure> {
        let mut ty = super::Reader::new(self.evaluator.facts())
            .location(root)?
            .ty;
        let mut cell = *self
            .memory
            .roots
            .get(&root)
            .ok_or("constant read lacks local storage")?;
        for (position, &index) in path.iter().enumerate() {
            self.charge(1)?;
            match &self.memory.cells[cell] {
                Cell::Frozen(_) => return Ok((cell, ty, position)),
                Cell::Fields { values, variant } => {
                    let child = usize::try_from(index)
                        .ok()
                        .and_then(|index| values.get(index))
                        .copied()
                        .ok_or(Failure::Fault("constant index is out of bounds"))?;
                    ty = super::Reader::new(self.evaluator.facts())
                        .stored_child(ty, *variant, index)?;
                    cell = child;
                }
                Cell::Bytes(_) => return Err("byte root bypassed its typed read consumer".into()),
                Cell::Empty => return Err(Failure::Fault("constant reads uninitialized storage")),
            }
        }
        Ok((cell, ty, path.len()))
    }
    pub(super) fn read_frozen(
        &mut self,
        mut value: Constant,
        mut ty: crate::types::TypeId,
        path: &[u64],
        range: Option<super::Range>,
        purpose: crate::const_eval::static_reads::ReadPurpose,
    ) -> Result<Read<Constant>, Failure> {
        for (position, &index) in path.iter().enumerate() {
            self.charge(1)?;
            let reader = super::Reader::new(self.evaluator.facts());
            if matches!(reader.view(value)?, super::View::Image { .. }) {
                return self.read_image_path(value, ty, &path[position..], range, purpose);
            }
            ty = reader
                .field_types(ty, value)?
                .get(index)
                .ok_or("constant field exceeds its stored owner")?;
            value = self.stored_field(value, index)?;
        }
        if let Some(range) = range {
            if matches!(
                super::Reader::new(self.evaluator.facts()).view(value)?,
                super::View::Image { .. }
            ) {
                return self.read_image_path(value, ty, &[], Some(range), purpose);
            }
            self.charge(1)?;
            value = self
                .evaluator
                .sequence_range(value, range.from, range.length)?;
        }
        if purpose == crate::const_eval::static_reads::ReadPurpose::ReferenceValidation
            && matches!(
                super::Reader::new(self.evaluator.facts()).view(value)?,
                super::View::Image { .. }
            )
            && super::Reader::new(self.evaluator.facts())
                .array_image(value)?
                .is_some()
        {
            return self.read_image_path(value, ty, &[], None, purpose);
        }
        Ok(Read::Value(value))
    }

    pub(super) fn sequence_index(
        &mut self,
        frame: &Frame,
        address: Resolved,
        projection: ProjectionKind,
    ) -> Result<u64, Failure> {
        let length = match self.evaluator.facts().types.kind(address.ty) {
            Some(crate::types::Kind::Array { length, .. }) => length,
            Some(crate::types::Kind::Slice(_)) => address
                .address
                .length()
                .ok_or("constant slice place lacks length metadata")?,
            _ => return Err("constant indexed place is not an array or slice".into()),
        };
        let index = match projection {
            ProjectionKind::SliceIndex { index } => {
                let index = super::super::values::integer(Self::value(frame, index)?)?;
                u64::try_from(index)
                    .map_err(|_| Failure::Fault("constant index is out of bounds"))?
            }
            ProjectionKind::FromEnd { offset, minimum } => {
                if offset == 0 || length < minimum {
                    return Err(Failure::Fault("constant suffix exceeds its checked bound"));
                }
                length
                    .checked_sub(offset)
                    .ok_or(Failure::Fault("constant suffix exceeds its checked bound"))?
            }
            _ => return Err("constant index has no sequence projection".into()),
        };
        if index >= length {
            return Err(Failure::Fault("constant index is out of bounds"));
        }
        Ok(index)
    }
}
