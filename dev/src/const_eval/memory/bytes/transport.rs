//! Raw snapshots preserve byte initialization and publish one destination update.
use super::{Constant, Failure, Machine, Reader, TypeId};
use crate::sema::values::LocationId;
pub(in crate::const_eval) struct Snapshot {
    pub bytes: Vec<u8>,
    pub initialized: Vec<u64>,
    pub(super) fragments: Option<Box<super::relocations::Fragments>>,
}
impl Snapshot {
    pub(in crate::const_eval) fn retained_slots(&self) -> usize {
        self.bytes
            .capacity()
            .saturating_add(self.initialized.capacity().saturating_mul(size_of::<u64>()))
            .saturating_add(self.fragments.as_ref().map_or(0, |fragments| {
                super::relocations::fragment_slots(fragments).saturating_mul(size_of::<Constant>())
            }))
            .saturating_add(4 * size_of::<Self>())
            .div_ceil(size_of::<Constant>())
    }
}
impl Machine<'_, '_> {
    pub(in crate::const_eval) fn has_byte_root(&self, root: LocationId) -> bool {
        !self.memory.bytes.roots.is_empty() && self.byte_root(root).is_some()
    }
    pub(in crate::const_eval) fn snapshot_byte_root(
        &mut self,
        root: LocationId,
        offset: u64,
        length: u64,
    ) -> Result<Option<Snapshot>, Failure> {
        let Some(id) = self.byte_root(root) else {
            return Ok(None);
        };
        let ty = self.byte_backing(id)?.ty;
        let size = usize::try_from(length).map_err(|_| "raw byte span exceeds host size")?;
        let mut buffer = self.empty_byte_buffer(ty, size)?;
        let start = usize::try_from(offset).map_err(|_| "raw byte offset exceeds host size")?;
        let end = start
            .checked_add(size)
            .filter(|&n| n <= self.byte_backing(id).map_or(0, |b| b.bytes.len()))
            .ok_or("raw byte snapshot exceeds backing bounds")?;
        self.charge(size)?;
        let source = self.byte_backing(id)?;
        buffer.bytes.copy_from_slice(&source.bytes[start..end]);
        for index in 0..size {
            buffer.set_initialized(index, source.initialized(start + index));
        }
        let previous = self.slots;
        self.slots = previous
            .checked_add(buffer.retained_slots())
            .ok_or("symbolic snapshot retention overflow")?;
        let fragments = (|| {
            let fragments = self.root_fragments(id, offset, length)?;
            self.buffer_fragments(&mut buffer, 0, length, &fragments)
        })();
        self.slots = previous;
        fragments?;
        Ok(Some(buffer.snapshot()))
    }
    pub(in crate::const_eval) fn snapshot_image_bytes(
        &mut self,
        value: Constant,
        ty: TypeId,
        offset: u64,
        length: u64,
    ) -> Result<Snapshot, Failure> {
        let size = usize::try_from(length).map_err(|_| "raw image span exceeds host size")?;
        let mut buffer = self.empty_byte_buffer(ty, size)?;
        self.charge(size)?;
        let image = Reader::new(self.evaluator.facts()).image(value)?;
        let start = usize::try_from(offset).map_err(|_| "raw image offset exceeds host size")?;
        let end = start
            .checked_add(size)
            .filter(|&end| end <= image.bytes.len())
            .ok_or("raw image snapshot exceeds checked storage")?;
        if image.ty != ty {
            return Err("raw image snapshot exceeds checked storage".into());
        }
        buffer.bytes.copy_from_slice(&image.bytes[start..end]);
        for index in 0..size {
            buffer.set_initialized(index, image.initialized(start + index));
        }
        let previous = self.slots;
        self.slots = previous
            .checked_add(buffer.retained_slots())
            .ok_or("symbolic image buffer retention overflow")?;
        let result = (|| {
            let fragments = self.image_fragments(value, offset, length)?;
            self.buffer_fragments(&mut buffer, 0, length, &fragments)
        })();
        self.slots = previous;
        result?;
        Ok(buffer.snapshot())
    }
    pub(in crate::const_eval) fn snapshot_typed_bytes(
        &mut self,
        ty: TypeId,
        value: Constant,
        count: u64,
        bytes: u64,
    ) -> Result<Snapshot, Failure> {
        let size = usize::try_from(bytes).map_err(|_| "typed byte span exceeds host size")?;
        let mut buffer = self.empty_byte_buffer(ty, size)?;
        let stride = self.footprint(ty)?.size;
        let previous = self.slots;
        self.slots = previous
            .checked_add(buffer.retained_slots())
            .ok_or("typed byte snapshot retention overflow")?;
        let encoded = (|| {
            if let crate::sema::values::View::Repeat {
                value: fill,
                length,
            } = Reader::new(self.evaluator.facts()).view(value)?
            {
                if length != count {
                    return Err("typed byte snapshot repetition differs from its span".into());
                }
                if count != 0 && stride != 0 {
                    self.encode_bytes(&mut buffer, 0, ty, fill)?;
                }
                self.repeat_byte_span(&mut buffer, 0, stride, count)?;
            } else {
                for index in 0..count {
                    self.charge(1)?;
                    let field = self.stored_field(value, index)?;
                    self.encode_bytes(
                        &mut buffer,
                        index
                            .checked_mul(stride)
                            .ok_or("typed byte snapshot offset overflow")?,
                        ty,
                        field,
                    )?;
                }
            }
            Ok::<_, Failure>(())
        })();
        self.slots = previous;
        encoded?;
        Ok(buffer.snapshot())
    }
    pub(in crate::const_eval) fn commit_byte_snapshot(
        &mut self,
        root: LocationId,
        offset: u64,
        snapshot: &Snapshot,
    ) -> Result<(), Failure> {
        let id = self.convert_byte_root(root)?;
        if self.byte_backing(id)?.pending_reads != 0 {
            return Err("constant byte storage mutated while a typed read was pending".into());
        }
        let start =
            usize::try_from(offset).map_err(|_| "byte destination offset exceeds host size")?;
        let end = start
            .checked_add(snapshot.bytes.len())
            .filter(|&n| n <= self.byte_backing(id).map_or(0, |b| b.bytes.len()))
            .ok_or("byte destination exceeds backing bounds")?;
        if snapshot.initialized.len() != snapshot.bytes.len().div_ceil(64) {
            return Err("malformed raw byte initialization mask".into());
        }
        let fragments = snapshot.fragments.as_ref().map_or_else(
            || Ok(Vec::new()),
            |tree| self.clip_fragments(tree, 0, snapshot.bytes.len() as u64),
        )?;
        self.commit_root_fragments(id, offset, snapshot.bytes.len() as u64, &fragments)?;
        self.charge(snapshot.bytes.len())?;
        let destination = self
            .memory
            .bytes
            .roots
            .get_mut(id.0 as usize)
            .ok_or("missing byte destination")?;
        destination.bytes[start..end].copy_from_slice(&snapshot.bytes);
        for index in 0..snapshot.bytes.len() {
            destination.set_initialized(
                start + index,
                snapshot.initialized[index / 64] & (1u64 << (index % 64)) != 0,
            );
        }
        Ok(())
    }
}
