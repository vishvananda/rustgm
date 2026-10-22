//! Converted roots have one exclusive physical backing; ordinary roots remain typed.
mod access;
mod codec;
#[cfg(test)]
#[path = "../../../tests/invariants/byte_storage.rs"]
mod tests;
mod transport;
pub(in crate::const_eval) use transport::Snapshot;
mod capabilities;
use super::{Cell, Constant, Failure, LocationId, Machine, Reader, TypeId};
use std::collections::HashMap;
#[derive(Clone, Copy)]
pub(super) struct Id(u32);
pub(super) struct Backing {
    ty: TypeId,
    bytes: Vec<u8>,
    initialized: Vec<u64>,
}
#[derive(Default)]
pub(super) struct Storage {
    roots: Vec<Backing>,
    families: HashMap<TypeId, bool>,
    retained: usize,
}
impl Storage {
    pub(super) fn retained_slots(&self) -> usize {
        self.retained
            .saturating_add(self.families.len().saturating_mul(4))
    }
}
impl Backing {
    fn retained_slots(&self) -> usize {
        self.bytes
            .capacity()
            .saturating_add(self.initialized.capacity().saturating_mul(size_of::<u64>()))
            .saturating_add(4 * size_of::<Backing>())
            .div_ceil(size_of::<Constant>())
    }
    fn snapshot(self) -> Snapshot {
        Snapshot {
            bytes: self.bytes,
            initialized: self.initialized,
        }
    }
    fn initialized(&self, index: usize) -> bool {
        self.initialized[index / 64] & (1u64 << (index % 64)) != 0
    }
    fn set_initialized(&mut self, index: usize, initialized: bool) {
        let mask = 1u64 << (index % 64);
        let word = &mut self.initialized[index / 64];
        *word = (*word & !mask) | if initialized { mask } else { 0 };
    }
}
impl Machine<'_, '_> {
    pub(super) fn byte_root(&self, root: LocationId) -> Option<Id> {
        if self.memory.bytes.roots.is_empty() {
            return None;
        }
        let &cell = self.memory.roots.get(&root)?;
        match self.memory.cells[cell] {
            Cell::Bytes(id) => Some(id),
            _ => None,
        }
    }
    fn byte_backing(&self, id: Id) -> Result<&Backing, Failure> {
        self.memory
            .bytes
            .roots
            .get(id.0 as usize)
            .ok_or("missing constant byte backing".into())
    }
    fn empty_backing(&mut self, ty: TypeId) -> Result<Backing, Failure> {
        let size = usize::try_from(self.footprint(ty)?.size)
            .map_err(|_| "constant byte backing exceeds host size")?;
        self.empty_byte_buffer(ty, size)
    }
    fn empty_byte_buffer(&mut self, ty: TypeId, size: usize) -> Result<Backing, Failure> {
        let words = size.div_ceil(64);
        let retained = size
            .checked_add(words.saturating_mul(size_of::<u64>()))
            .and_then(|n| n.checked_add(4 * size_of::<Backing>()))
            .ok_or("constant byte backing retention overflow")?
            .div_ceil(size_of::<Constant>());
        self.charge(size.saturating_add(words))?;
        self.reserve_cells(retained)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(size)
            .map_err(|_| "constant byte backing allocation failed")?;
        bytes.resize(size, 0);
        let mut initialized = Vec::new();
        initialized
            .try_reserve_exact(words)
            .map_err(|_| "constant byte mask allocation failed")?;
        initialized.resize(words, 0);
        Ok(Backing {
            ty,
            bytes,
            initialized,
        })
    }
    fn convert_byte_root(&mut self, root: LocationId) -> Result<Id, Failure> {
        if let Some(id) = self.byte_root(root) {
            return Ok(id);
        }
        let &cell = self
            .memory
            .roots
            .get(&root)
            .ok_or("constant byte destination requires local storage")?;
        let ty = Reader::new(self.evaluator.facts()).location(root)?.ty;
        if !self.byte_family(ty)? {
            return Err(
                "constant byte backing with pointer or enum storage is not implemented".into(),
            );
        }
        let mut backing = self.empty_backing(ty)?;
        let previous = self.slots;
        self.slots = previous
            .checked_add(backing.retained_slots())
            .ok_or("byte conversion retention overflow")?;
        let encoded = self.encode_byte_cells(&mut backing, cell, ty);
        self.slots = previous;
        encoded?;
        let id = Id(u32::try_from(self.memory.bytes.roots.len())
            .map_err(|_| "constant byte backing identity exhausted")?);
        let retained = backing
            .bytes
            .capacity()
            .checked_add(
                backing
                    .initialized
                    .capacity()
                    .saturating_mul(size_of::<u64>()),
            )
            .and_then(|n| n.checked_add(4 * size_of::<Backing>()))
            .ok_or("constant byte backing retention overflow")?
            .div_ceil(size_of::<Constant>());
        self.reserve_cells(retained)?;
        self.memory
            .bytes
            .roots
            .try_reserve(1)
            .map_err(|_| "constant byte backing index allocation failed")?;
        let retained = self
            .memory
            .bytes
            .retained
            .checked_add(retained)
            .ok_or("constant byte backing retention overflow")?;
        self.memory.bytes.roots.push(backing);
        self.memory.bytes.retained = retained;
        // All addresses resolve from this root. Its former field cells are unreachable;
        // retained arena capacity remains accounted by Memory::retained_cells.
        self.memory.cells[cell] = Cell::Bytes(id);
        Ok(id)
    }
    pub(super) fn byte_read(
        &mut self,
        root: LocationId,
        path: &[u64],
    ) -> Result<Option<Constant>, Failure> {
        let Some(id) = self.byte_root(root) else {
            return Ok(None);
        };
        let ty = self.byte_backing(id)?.ty;
        let (selected, offset) = self.storage_offset(ty, path)?;
        self.decode_bytes(id, offset, selected).map(Some)
    }
}
