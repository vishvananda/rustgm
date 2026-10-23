//! Logical raw selections retain physical allocation coordinates independently of type.
use super::{Constant, Failure, Machine, Origin, RawAlignment, Reader, Resolved, Terminal, TypeId};
use crate::sema::values::LocationId;
#[derive(Clone, Copy)]
pub(in crate::const_eval) struct BytePlace {
    pub root: LocationId,
    pub offset: u64,
    pub ty: TypeId,
}
impl Machine<'_, '_> {
    pub(in crate::const_eval) fn byte_place(
        &mut self,
        place: Resolved,
    ) -> Result<Option<BytePlace>, Failure> {
        if place.union.is_some() {
            return Ok(None);
        }
        if !self.byte_projecting(place)? {
            return Ok(None);
        }
        let path = self.path(place.address)?;
        if let Some(Terminal::Bytes { offset, ty }) = path.terminal {
            if place.raw.is_none()
                && !Reader::new(self.evaluator.facts()).same_storage_type(ty, place.ty)
            {
                return Err("constant byte view differs from its checked place type".into());
            }
            return Ok(Some(BytePlace {
                root: path.root,
                offset,
                ty: place.ty,
            }));
        }
        if place.raw.is_none() {
            return Ok(None);
        }
        let root_ty = Reader::new(self.evaluator.facts()).location(path.root)?.ty;
        let (stored, offset) = self.storage_offset(root_ty, &path.fields)?;
        if let Some(range) = path.range()? {
            if self.evaluator.facts().types.is_sized(place.ty) != Some(true) {
                return Err("constant raw whole-range byte access is not implemented".into());
            }
            self.footprint(stored)?;
            let reader = Reader::new(self.evaluator.facts());
            let selected = reader
                .location(
                    place
                        .address
                        .location()
                        .ok_or("constant sequence lacks provenance")?,
                )?
                .ty;
            let displacement =
                reader.sequence_displacement(stored, range.from, range.length, selected)?;
            return Ok(Some(BytePlace {
                root: path.root,
                offset: offset
                    .checked_add(displacement)
                    .ok_or("constant sequence byte offset overflow")?,
                ty: place.ty,
            }));
        }
        if Reader::new(self.evaluator.facts()).same_storage_type(stored, place.ty) {
            return Ok(None);
        }
        Ok(Some(BytePlace {
            root: path.root,
            offset,
            ty: place.ty,
        }))
    }
    pub(in crate::const_eval) fn byte_address(
        &mut self,
        place: Resolved,
        displacement: u64,
        ty: TypeId,
    ) -> Result<Constant, Failure> {
        if displacement != 0 {
            self.path(place.address)?;
        }
        self.charge(1)?;
        self.reserve_cells(4)?;
        let (address, work) = self.evaluator.byte_projection(
            place.address,
            displacement,
            ty,
            self.limits.steps.saturating_sub(self.work.steps),
        )?;
        self.charge(work)?;
        Ok(address)
    }
    pub(in crate::const_eval) fn check_byte_access(
        &mut self,
        place: Resolved,
        selected: BytePlace,
        reference: bool,
    ) -> Result<(), Failure> {
        let root_ty = Reader::new(self.evaluator.facts())
            .location(selected.root)?
            .ty;
        let root = self.footprint(root_ty)?;
        let leaf = self.footprint(selected.ty)?;
        if selected
            .offset
            .checked_add(leaf.size)
            .is_none_or(|end| end > root.size)
        {
            return Err("constant byte access is outside allocation bounds".into());
        }
        let anchor = if let Some(raw) = place.raw {
            let anchor = raw
                .location
                .ok_or("constant raw byte access lacks provenance")?;
            let (address, work) = self.evaluator.word_address(
                Constant::pointer(anchor),
                self.limits.steps.saturating_sub(self.work.steps),
            )?;
            self.charge(work)?;
            if address.root() != selected.root {
                return Err("constant raw byte anchor names a different allocation".into());
            }
            Some((raw, address.offset()))
        } else {
            None
        };
        let (offset, align) = if reference {
            (selected.offset, leaf.align)
        } else if let Some((raw, offset)) = anchor {
            if raw.alignment == RawAlignment::Unaligned {
                return Ok(());
            }
            (offset, self.footprint(raw.ty)?.align)
        } else {
            (selected.offset, leaf.align)
        };
        if root.align < align || !offset.is_multiple_of(u64::from(align)) {
            return Err("constant raw byte access is unaligned".into());
        }
        Ok(())
    }
    pub(in crate::const_eval) fn checked_byte_reference(
        &mut self,
        place: Resolved,
        selected: BytePlace,
        mutable: bool,
    ) -> Result<Option<Constant>, Failure> {
        let location = Reader::new(self.evaluator.facts()).location(selected.root)?;
        if mutable {
            self.check_raw_write_authority(place.address, selected.root)?;
        }
        if let crate::sema::values::Origin::Static(id) = location.origin {
            self.check_static_read(id)?;
            if self.active_initializers.contains(&id) {
                return Err(
                    "constant raw references to cyclic static storage are not implemented".into(),
                );
            }
        }
        self.check_byte_access(place, selected, true)?;
        let Some(_value) = self.read_byte_access(
            selected.root,
            selected.offset,
            selected.ty,
            crate::const_eval::static_reads::ReadPurpose::ReferenceValidation,
        )?
        else {
            return Ok(None);
        };
        let retained = Reader::new(self.evaluator.facts()).location(
            place
                .address
                .location()
                .ok_or("constant byte reference lacks provenance")?,
        )?;
        let address = if retained.ty == place.ty
            && matches!(
                retained.origin,
                crate::sema::values::Origin::ByteOffset { .. }
            ) {
            // A validated normalized view already has the exact checked identity.
            place.address
        } else {
            self.byte_address(place, 0, place.ty)?
        };
        Ok(Some(if mutable {
            address
        } else {
            address.restrict_shared()
        }))
    }
    pub(in crate::const_eval) fn byte_projecting(&self, place: Resolved) -> Result<bool, Failure> {
        let location = Reader::new(self.evaluator.facts()).location(
            place
                .address
                .location()
                .ok_or("constant pointer has no allocation provenance")?,
        )?;
        Ok(matches!(location.origin, Origin::ByteOffset { .. })
            || place.raw.is_some()
                && !Reader::new(self.evaluator.facts()).same_storage_type(location.ty, place.ty))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/byte_access.rs"]
mod tests;
