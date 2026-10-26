//! Converted roots have one exclusive physical backing; ordinary roots remain typed.
mod access;
mod codec;
mod continuations;
mod enumerations;
mod fill;
mod images;
mod intervals;
mod niches;
mod nonnull_values;
mod products;
mod relocations;
mod slice_words;
mod sources;
mod subobjects;
#[cfg(test)]
#[path = "../../../tests/invariants/byte_storage.rs"]
mod tests;
mod transport;
mod unions;
mod value_reinterpretation;
mod words;
pub(in crate::const_eval) use transport::Snapshot;
mod capabilities;
mod raw;
use super::{Cell, Constant, Failure, LocationId, Machine, Read, Reader, TypeId};
use crate::support::hash::HashMap;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Id(u32);
pub(super) struct Backing {
    ty: TypeId,
    pending_reads: u32,
    bytes: Vec<u8>,
    initialized: Vec<u64>,
    fragments: Option<Box<relocations::Fragments>>,
}
#[derive(Default)]
pub(super) struct Storage {
    #[cfg(test)]
    decoded: usize,
    #[cfg(test)]
    checked_duties: usize,
    #[cfg(test)]
    paid_duties: usize,
    roots: Vec<Backing>,
    families: HashMap<TypeId, bool>,
    retained: usize,
    fragment_slots: usize,
    words: Option<Box<relocations::Words>>,
    paths: Option<Box<subobjects::Paths>>,
    reads: Option<Box<continuations::Reads>>,
    enumerations: Option<Box<enumerations::Enumerations>>,
}
impl Storage {
    pub(super) fn retained_slots(&self) -> usize {
        self.retained
            .saturating_add(self.fragment_slots)
            .saturating_add(
                self.reads
                    .as_ref()
                    .map_or(0, |reads| reads.retained_slots()),
            )
            .saturating_add(
                self.paths
                    .as_ref()
                    .map_or(0, |paths| paths.retained_slots()),
            )
            .saturating_add(
                self.words
                    .as_ref()
                    .map_or(0, |words| words.retained_slots()),
            )
            .saturating_add(self.families.len().saturating_mul(4))
            .saturating_add(
                self.enumerations
                    .as_ref()
                    .map_or(0, |enums| enums.retained_slots()),
            )
    }
}
impl Backing {
    fn retained_slots(&self) -> usize {
        self.bytes
            .capacity()
            .saturating_add(self.initialized.capacity().saturating_mul(size_of::<u64>()))
            .saturating_add(self.fragments.as_ref().map_or(0, |fragments| {
                relocations::fragment_slots(fragments).saturating_mul(size_of::<Constant>())
            }))
            .saturating_add(4 * size_of::<Backing>())
            .div_ceil(size_of::<Constant>())
    }
    fn snapshot(self) -> Snapshot {
        Snapshot {
            bytes: self.bytes,
            initialized: self.initialized,
            fragments: self.fragments,
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
    pub(super) fn has_byte_roots(&self) -> bool {
        !self.memory.bytes.roots.is_empty()
    }
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
            pending_reads: 0,
            bytes,
            initialized,
            fragments: None,
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
        self.install_byte_root(root, backing, Some(cell))
    }
    fn install_byte_root(
        &mut self,
        root: LocationId,
        backing: Backing,
        cell: Option<usize>,
    ) -> Result<Id, Failure> {
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
        let fragment_slots = backing
            .fragments
            .as_ref()
            .map_or(0, |tree| relocations::fragment_slots(tree));
        self.reserve_cells(fragment_slots)?;
        self.memory.bytes.fragment_slots = self
            .memory
            .bytes
            .fragment_slots
            .checked_add(fragment_slots)
            .ok_or("symbolic backing retention overflow")?;
        self.memory.bytes.roots.push(backing);
        self.memory.bytes.retained = retained;
        // All addresses resolve from this root. Its former field cells are unreachable;
        // retained arena capacity remains accounted by Memory::retained_cells.
        if let Some(cell) = cell {
            self.memory.cells[cell] = Cell::Bytes(id);
        } else {
            self.memory
                .roots
                .try_reserve(1)
                .map_err(|_| "constant immutable byte root index allocation failed")?;
            self.memory
                .cells
                .try_reserve(1)
                .map_err(|_| "constant immutable byte root allocation failed")?;
            self.memory.roots.insert(root, self.memory.cells.len());
            self.memory.cells.push(Cell::Bytes(id));
        }
        Ok(id)
    }
    pub(super) fn byte_read(
        &mut self,
        root: LocationId,
        path: &[u64],
        purpose: crate::const_eval::static_reads::ReadPurpose,
    ) -> Result<Option<Read<Constant>>, Failure> {
        let Some(id) = self.byte_root(root) else {
            return Ok(None);
        };
        let ty = self.byte_backing(id)?.ty;
        let (selected, offset) = self.storage_offset(ty, path)?;
        self.decode_bytes(id, offset, selected, purpose).map(Some)
    }
}
