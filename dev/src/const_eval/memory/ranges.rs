//! Terminal sequence views alias their original cells; value reads create immutable snapshots.
use super::{Cell, Constant, Failure, LocationId, Machine, ProjectionKind, Resolved};
use crate::{
    sema::values::{Origin, Reader},
    types::Kind,
};
#[derive(Clone, Copy)]
pub(in crate::const_eval) struct Range {
    pub from: u64,
    pub length: u64,
}
pub(in crate::const_eval) struct StoragePath {
    pub root: LocationId,
    pub fields: Vec<u64>,
    pub terminal: Option<Terminal>,
}
#[derive(Clone, Copy)]
pub(in crate::const_eval) enum Terminal {
    Range(Range),
    Bytes {
        offset: u64,
        ty: crate::types::TypeId,
    },
}
impl StoragePath {
    pub(in crate::const_eval) fn range(&self) -> Result<Option<Range>, Failure> {
        match self.terminal {
            None => Ok(None),
            Some(Terminal::Range(range)) => Ok(Some(range)),
            Some(Terminal::Bytes { .. }) => {
                Err("constant byte address reached a range-only consumer".into())
            }
        }
    }
    pub(in crate::const_eval) fn fields_only(self) -> Result<(LocationId, Vec<u64>), Failure> {
        match self.terminal {
            None => Ok((self.root, self.fields)),
            Some(Terminal::Range(_)) => {
                Err("constant raw whole-range access is not implemented".into())
            }
            Some(Terminal::Bytes { .. }) => {
                Err("constant byte address reached a fields-only consumer".into())
            }
        }
    }
}
impl Machine<'_, '_> {
    pub(super) fn range_projection(
        &mut self,
        address: Resolved,
        projection: crate::body::ir::Projection,
    ) -> Result<Constant, Failure> {
        let bound = self.sequence_length(address)?;
        let (from, length) = match projection.kind {
            ProjectionKind::Subslice { prefix, suffix } => (
                prefix,
                bound
                    .checked_sub(prefix)
                    .and_then(|n| n.checked_sub(suffix))
                    .ok_or(Failure::Fault("constant range is out of bounds"))?,
            ),
            ProjectionKind::Subarray { start, end } if end <= bound => (
                start,
                end.checked_sub(start).ok_or("constant range is reversed")?,
            ),
            _ => return Err("constant range lacks checked sequence bounds".into()),
        };
        self.charge(1)?;
        self.evaluator
            .subsequence(address.address, from, length, projection.ty)
    }
    pub(in crate::const_eval) fn sequence_length(
        &mut self,
        place: Resolved,
    ) -> Result<u64, Failure> {
        if self.is_raw_slice(place) {
            // The pointer value was already read. Its metadata does not read
            // the pointee, form a reference or perform address arithmetic.
            return self.raw_slice_metadata(place).map(|(_, length)| length);
        }
        if let Some(Kind::Array { length, .. }) = self.evaluator.facts().types.kind(place.ty)
            && (place.raw.is_some() || self.byte_projecting(place)?)
        {
            // Raw place evaluation already read the pointer value. A checked
            // array length is type metadata, independently of selected contents.
            self.charge(1)?;
            return Ok(length);
        }
        let path = self.path(place.address)?;
        // Metadata does not read bytes or demand a static initializer. Local storage
        // must nevertheless have an initialized container at the selected path.
        self.check_length_storage(path.root, &path.fields)?;
        if let Some(access) = place.raw {
            if path.range()?.is_some() {
                return Err("constant raw whole-range access is not implemented".into());
            }
            self.check_raw_read(place, access, path.root, &path.fields)?;
        }
        let length = match self.evaluator.facts().types.kind(place.ty) {
            Some(Kind::Array { length, .. }) => length,
            Some(Kind::Slice(_)) => place
                .address
                .length()
                .ok_or("constant slice place lacks length metadata")?,
            _ => return Err("constant length requires an array or slice place".into()),
        };
        if let Some(range) = path.range()?
            && range.length != length
        {
            return Err("constant range length differs from its checked place".into());
        }
        Ok(length)
    }
    fn check_length_storage(&mut self, root: LocationId, path: &[u64]) -> Result<(), Failure> {
        let Some(&mut_cell) = self.memory.roots.get(&root) else {
            let location = Reader::new(self.evaluator.facts()).location(root)?;
            if location.value.is_none() && !matches!(location.origin, Origin::Static(_)) {
                return Err("constant length reads uninitialized storage".into());
            }
            return Ok(());
        };
        if self.byte_root(root).is_some() {
            self.check_byte_length(root, path)?;
            return Ok(());
        }
        let mut cell = mut_cell;
        for (position, &index) in path.iter().enumerate() {
            self.charge(1)?;
            match &self.memory.cells[cell] {
                Cell::Frozen(value) => {
                    let mut value = *value;
                    self.charge(path.len() - position)?;
                    for &index in &path[position..] {
                        value = self.stored_field(value, index)?;
                    }
                    return Ok(());
                }
                Cell::Fields { values, .. } => {
                    cell = *values
                        .get(
                            usize::try_from(index)
                                .map_err(|_| "constant index exceeds host size")?,
                        )
                        .ok_or("constant index is out of bounds")?;
                }
                Cell::Bytes(_) => return Err("byte root bypassed length metadata consumer".into()),
                Cell::Empty => return Err("constant length reads uninitialized storage".into()),
            }
        }
        if matches!(self.memory.cells[cell], Cell::Empty) {
            return Err("constant length reads uninitialized storage".into());
        }
        Ok(())
    }
    pub(super) fn read_range(
        &mut self,
        root: LocationId,
        path: &[u64],
        range: Range,
        purpose: crate::const_eval::static_reads::ReadPurpose,
    ) -> Result<super::Read<Constant>, Failure> {
        if let Some(value) = self.read_byte_range(root, path, range, purpose)? {
            return Ok(value);
        }
        let (cell, ty, rest) = self.read_cell_selection(root, path)?;
        match &self.memory.cells[cell] {
            Cell::Frozen(value) => {
                let value = *value;
                self.read_frozen(value, ty, &path[rest..], Some(range), purpose)
            }
            Cell::Bytes(_) => Err("byte root bypassed its range snapshot consumer".into()),
            Cell::Empty => Err("constant range reads uninitialized storage".into()),
            Cell::Fields { values, .. } => {
                let start =
                    usize::try_from(range.from).map_err(|_| "constant range exceeds host size")?;
                let length = usize::try_from(range.length)
                    .map_err(|_| "constant range exceeds host size")?;
                let end = start
                    .checked_add(length)
                    .ok_or("constant range size overflow")?;
                if values.get(start..end).is_none() {
                    return Err("constant range is out of bounds".into());
                }
                self.charge(length)?;
                self.reserve_cells(length.saturating_mul(6))?;
                let Cell::Fields { values, .. } = &self.memory.cells[cell] else {
                    return Err("constant range container changed".into());
                };
                let selected = values[start..end].to_vec();
                let mut fields = Vec::with_capacity(length);
                for cell in selected {
                    let super::Read::Value(value) = self.freeze(cell, purpose)? else {
                        return Ok(super::Read::Pending);
                    };
                    fields.push(value);
                }
                self.evaluator.fields(&fields, None).map(super::Read::Value)
            }
        }
    }
    pub(in crate::const_eval) fn write_range(
        &mut self,
        root: LocationId,
        path: &[u64],
        range: Range,
        value: Constant,
    ) -> Result<(), Failure> {
        if self.write_byte_range(root, path, range, value)? {
            return Ok(());
        }
        let cell = self.cell(root, path)?;
        self.thaw(cell)?;
        let start = usize::try_from(range.from).map_err(|_| "constant range exceeds host size")?;
        let length =
            usize::try_from(range.length).map_err(|_| "constant range exceeds host size")?;
        self.charge(length)?;
        self.reserve_cells(length.saturating_mul(2))?;
        let Cell::Fields { values, .. } = &self.memory.cells[cell] else {
            return Err("constant range writes uninitialized storage".into());
        };
        let end = start
            .checked_add(length)
            .ok_or("constant range size overflow")?;
        let selected = values
            .get(start..end)
            .ok_or("constant range is out of bounds")?
            .to_vec();
        let actual = match Reader::new(self.evaluator.facts()).view(value)? {
            crate::sema::values::View::Product {
                length,
                variant: None,
            }
            | crate::sema::values::View::Repeat { length, .. }
            | crate::sema::values::View::ByteRange { length, .. } => length,
            crate::sema::values::View::Bytes(text) => {
                Reader::new(self.evaluator.facts()).bytes(text)?.len() as u64
            }
            _ => return Err("constant range assignment requires a sequence value".into()),
        };
        if actual != range.length {
            return Err("constant range assignment length differs from its storage".into());
        }
        for (index, cell) in selected.into_iter().enumerate() {
            self.memory.cells[cell] = Cell::Frozen(self.stored_field(value, index as u64)?);
        }
        Ok(())
    }
}
