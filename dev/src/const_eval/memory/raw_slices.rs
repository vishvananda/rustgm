//! Raw slice indexing selects a sized subobject before ordinary access checks.
use super::{
    Constant, Failure, Liveness, Machine, Origin, RawAccess, Reader, Resolved, StoragePath, TypeId,
};
use crate::{sema::values::WordTarget, types::Kind};
#[cfg(test)]
#[path = "../../../tests/invariants/raw_slices.rs"]
mod tests;
impl Machine<'_, '_> {
    pub(super) fn indexed_element(
        &mut self,
        place: Resolved,
        index: u64,
        result: TypeId,
    ) -> Result<Resolved, Failure> {
        if self.is_raw_slice(place) {
            return self.raw_slice_element(place, index, result);
        }
        self.project_raw_type(place)?;
        Ok(Resolved {
            address: self.evaluator.projection(place.address, index, result)?,
            ty: result,
            raw: place.raw,
            union: None,
        })
    }
    pub(super) fn is_raw_slice(&self, place: Resolved) -> bool {
        place.raw.is_some()
            && matches!(
                self.evaluator.facts().types.kind(place.ty),
                Some(Kind::Slice(_))
            )
    }
    pub(super) fn raw_slice_metadata(&mut self, place: Resolved) -> Result<(TypeId, u64), Failure> {
        self.charge(1)?;
        let access = place
            .raw
            .ok_or("constant raw slice lacks its access anchor")?;
        if access.ty != place.ty
            || access.location != place.address.location()
            || place.union.is_some()
        {
            return Err("constant raw slice differs from its checked access anchor".into());
        }
        let Some(Kind::Slice(element)) = self.evaluator.facts().types.kind(place.ty) else {
            return Err("constant raw indexed place is not a slice".into());
        };
        let length = place
            .address
            .length()
            .ok_or("constant raw slice lacks length metadata")?;
        Ok((element, length))
    }
    pub(super) fn raw_slice_element(
        &mut self,
        place: Resolved,
        index: u64,
        result: TypeId,
    ) -> Result<Resolved, Failure> {
        let (element, length) = self.raw_slice_metadata(place)?;
        if index >= length {
            return Err(Failure::Fault("constant index is out of bounds"));
        }
        if !Reader::new(self.evaluator.facts()).same_storage_type(element, result) {
            return Err("constant raw slice element differs from its checked type".into());
        }
        let stride = self.footprint(element)?.size;
        index
            .checked_mul(stride)
            .filter(|&offset| i64::try_from(offset).is_ok())
            .ok_or("constant raw slice element offset exceeds allocation size")?;
        let path = self.path_for(
            place.address,
            if index == 0 || stride == 0 {
                Liveness::AddressOnly
            } else {
                Liveness::Required
            },
            size_of::<StoragePath>().div_ceil(size_of::<Constant>()),
        )?;
        let retained = size_of::<StoragePath>()
            .div_ceil(size_of::<Constant>())
            .saturating_add(path.fields.len());
        self.reserve_cells(retained)?;
        let previous = self.slots;
        self.slots = previous
            .checked_add(retained)
            .ok_or("constant raw slice path retention overflow")?;
        let selected = (|| {
            self.check_slice_storage(&path, place.address, element)?;
            self.slice_data_element(place.address, element, index, result)
        })();
        self.slots = previous;
        let address = selected?;
        Ok(Resolved {
            address,
            ty: result,
            union: None,
            raw: Some(RawAccess {
                alignment: place
                    .raw
                    .ok_or("constant raw slice anchor disappeared")?
                    .alignment,
                location: address.location(),
                ty: result,
            }),
        })
    }
    fn check_slice_storage(
        &mut self,
        path: &StoragePath,
        address: Constant,
        element: TypeId,
    ) -> Result<(), Failure> {
        let reader = Reader::new(self.evaluator.facts());
        let root = reader.location(path.root)?.ty;
        let declared = reader
            .location(
                address
                    .location()
                    .ok_or("constant raw slice lacks storage")?,
            )?
            .ty;
        self.footprint(root)?;
        let (actual, _) = self.storage_offset(root, &path.fields)?;
        let reader = Reader::new(self.evaluator.facts());
        if let Some(range) = path.range {
            let Some(Kind::Array {
                element: stored,
                length,
            }) = self.evaluator.facts().types.kind(actual)
            else {
                return Err("constant raw slice range lacks original array storage".into());
            };
            let declared_sequence = match self.evaluator.facts().types.kind(declared) {
                Some(Kind::Slice(stored)) => reader.same_storage_type(stored, element),
                Some(Kind::Array {
                    element: stored,
                    length,
                }) => length == range.length && reader.same_storage_type(stored, element),
                _ => false,
            };
            if !reader.same_storage_type(stored, element)
                || range
                    .from
                    .checked_add(range.length)
                    .is_none_or(|end| end > length)
                || !declared_sequence
            {
                return Err(
                    "constant raw slice range differs from its stored element or bounds".into(),
                );
            }
        } else if !reader.same_storage_type(actual, declared) {
            return Err("constant raw slice data differs from its declared storage".into());
        }
        Ok(())
    }
    fn slice_data_element(
        &mut self,
        address: Constant,
        element: TypeId,
        index: u64,
        result: TypeId,
    ) -> Result<Constant, Failure> {
        let reader = Reader::new(self.evaluator.facts());
        let location = reader.location(
            address
                .location()
                .ok_or("constant raw slice lacks data provenance")?,
        )?;
        match self.evaluator.facts().types.kind(location.ty) {
            Some(Kind::Array {
                element: stored,
                length,
            }) if reader.same_storage_type(stored, element) => {
                if index >= length {
                    return Err(Failure::Fault(
                        "constant raw slice element is outside backing bounds",
                    ));
                }
                self.evaluator.projection(address, index, result)
            }
            Some(Kind::Slice(stored)) if reader.same_storage_type(stored, element) => {
                let Origin::Subsequence { length, .. } = location.origin else {
                    return Err("constant raw slice lacks a normalized range".into());
                };
                if index >= length {
                    return Err(Failure::Fault(
                        "constant raw slice element is outside backing bounds",
                    ));
                }
                self.evaluator.projection(address, index, result)
            }
            _ if reader.same_storage_type(location.ty, element) => {
                self.offset_slice_element(address, index, result)
            }
            _ => Err("constant raw slice element reinterpretation is not implemented".into()),
        }
    }
    fn offset_slice_element(
        &mut self,
        address: Constant,
        index: u64,
        result: TypeId,
    ) -> Result<Constant, Failure> {
        let reader = Reader::new(self.evaluator.facts());
        let location = reader.location(
            address
                .location()
                .ok_or("constant raw slice lacks data provenance")?,
        )?;
        if let Origin::Projection { .. } = location.origin {
            let (parent, start) = address.projection_parent(reader)?;
            let owner = reader
                .location(
                    parent
                        .location()
                        .ok_or("constant raw slice lacks its array parent")?,
                )?
                .ty;
            if let Some(Kind::Array { element, length }) = self.evaluator.facts().types.kind(owner)
                && reader.same_storage_type(element, result)
            {
                let selected =
                    start
                        .checked_add(index)
                        .filter(|&n| n < length)
                        .ok_or(Failure::Fault(
                            "constant raw slice element is outside backing bounds",
                        ))?;
                return self.evaluator.projection(parent, selected, result);
            }
        }
        if index != 0 {
            return Err("constant raw slice offset requires contiguous array storage".into());
        }
        let (WordTarget::Value(value), _) = self.evaluator.pointer_words(address)? else {
            return Err("constant raw slice data is not a pointer word".into());
        };
        Ok(value)
    }
}
