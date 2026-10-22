//! Indexed reads preserve compressed immutable products and original storage identity.
use super::{Cell, Constant, Failure, Frame, LocationId, Machine, ProjectionKind, Resolved};

impl Machine<'_, '_> {
    pub(super) fn read_cell(
        &mut self,
        root: LocationId,
        path: &[u64],
    ) -> Result<Constant, Failure> {
        if let Some(value) = self.byte_read(root, path)? {
            return Ok(value);
        }
        let mut cell = *self
            .memory
            .roots
            .get(&root)
            .ok_or("constant read lacks local storage")?;
        for (position, &index) in path.iter().enumerate() {
            self.charge(1)?;
            match &self.memory.cells[cell] {
                Cell::Frozen(value) => {
                    let mut value = *value;
                    self.charge(path.len() - position)?;
                    for &index in &path[position..] {
                        value = self.evaluator.field(value, index)?;
                    }
                    return Ok(value);
                }
                Cell::Fields { values, .. } => {
                    cell = usize::try_from(index)
                        .ok()
                        .and_then(|index| values.get(index))
                        .copied()
                        .ok_or(Failure::Fault("constant index is out of bounds"))?;
                }
                Cell::Bytes(_) => return Err("byte root bypassed its typed read consumer".into()),
                Cell::Empty => {
                    return Err(Failure::Fault("constant reads uninitialized storage"));
                }
            }
        }
        self.freeze(cell)
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
