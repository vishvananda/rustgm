//! Reinterpreted typed accesses share the same exclusive physical backing.
use super::{Constant, Failure, Id, Machine, Read, Reader, TypeId};
use crate::{const_eval::static_reads::ReadPurpose, sema::values::LocationId};

impl Machine<'_, '_> {
    pub(super) fn prepare_byte_access(
        &mut self,
        root: LocationId,
        leaf: TypeId,
        purpose: ReadPurpose,
    ) -> Result<Option<Id>, Failure> {
        let location = Reader::new(self.evaluator.facts()).location(root)?;
        if let crate::sema::values::Origin::Static(id) = location.origin {
            self.check_static_read(id)?;
            self.check_static_value_read(location.ty, leaf, purpose)?;
        }
        if let Some(id) = self.byte_root(root) {
            return Ok(Some(id));
        }
        if self.memory.roots.contains_key(&root) {
            return self.convert_byte_root(root).map(Some);
        }
        let Some(value) = self.read_storage(root, &[], None, location.ty, purpose)? else {
            return Ok(None);
        };
        if !self.byte_family(location.ty)? {
            return Err("constant raw byte access requires a supported backing family".into());
        }
        let mut backing = self.empty_backing(location.ty)?;
        let previous = self.slots;
        self.slots = previous
            .checked_add(backing.retained_slots())
            .ok_or("immutable byte backing retention overflow")?;
        let encoded = self.encode_bytes(&mut backing, 0, location.ty, value);
        self.slots = previous;
        encoded?;
        self.reserve_cells(8)?;
        self.install_byte_root(root, backing, None).map(Some)
    }
    pub(in crate::const_eval) fn read_byte_access(
        &mut self,
        root: LocationId,
        offset: u64,
        ty: TypeId,
        purpose: ReadPurpose,
    ) -> Result<Option<Constant>, Failure> {
        if !self.byte_family(ty)? {
            return Err("constant raw byte read requires a supported target family".into());
        }
        let Some(id) = self.prepare_byte_access(root, ty, purpose)? else {
            return Ok(None);
        };
        match self.decode_bytes(id, offset, ty, purpose)? {
            Read::Value(value) => Ok(Some(value)),
            Read::Pending => Ok(None),
        }
    }
    pub(in crate::const_eval) fn snapshot_byte_access(
        &mut self,
        root: LocationId,
        offset: u64,
        bytes: u64,
        ty: TypeId,
    ) -> Result<Option<super::Snapshot>, Failure> {
        if self
            .prepare_byte_access(root, ty, ReadPurpose::Value)?
            .is_none()
        {
            return Ok(None);
        }
        self.snapshot_byte_root(root, offset, bytes)
    }
    pub(in crate::const_eval) fn write_byte_access(
        &mut self,
        root: LocationId,
        offset: u64,
        ty: TypeId,
        value: Constant,
    ) -> Result<(), Failure> {
        if !self.byte_family(ty)? {
            return Err("constant raw byte write requires a supported target family".into());
        }
        self.convert_byte_root(root)?;
        self.write_byte_value(root, offset, ty, value)
    }
}
