//! Canonical names occupy one byte arena; repeated ASCII lookups allocate nothing.
use std::{collections::hash_map::RandomState, hash::BuildHasher};
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NameId(pub u32);
#[derive(Clone, Copy)]
struct Entry {
    start: u32,
    end: u32,
    hash: u64,
}
#[derive(Default)]
pub struct Names {
    bytes: String,
    entries: Vec<Entry>,
    slots: Vec<Option<NameId>>,
    hash: RandomState,
    scratch: String,
}
impl Names {
    /// Intern the semantic spelling (raw prefixes and lifetime quotes excluded).
    ///
    /// # Errors
    /// Rejects a name arena that cannot be indexed with 32-bit identities.
    pub fn intern(&mut self, spelling: &str) -> Result<NameId, &'static str> {
        let spelling = spelling.strip_prefix('\'').unwrap_or(spelling);
        let spelling = spelling.strip_prefix("r#").unwrap_or(spelling);
        if spelling.is_ascii() {
            return self.canonical(spelling);
        }
        let mut scratch = std::mem::take(&mut self.scratch);
        scratch.clear();
        scratch.extend(spelling.nfc());
        let result = self.canonical(&scratch);
        self.scratch = scratch;
        result
    }
    /// Exact linkage strings share the existing arena without identifier normalization.
    pub(crate) fn intern_exact(&mut self, spelling: &str) -> Result<NameId, &'static str> {
        self.canonical(spelling)
    }
    fn canonical(&mut self, spelling: &str) -> Result<NameId, &'static str> {
        if self.slots.is_empty() {
            self.slots.resize(16, None);
        }
        let hash = self.hash.hash_one(spelling);
        let mut slot = bucket(hash, self.slots.len());
        while let Some(id) = self.slots[slot] {
            if self.entries[id.0 as usize].hash == hash && self.get(id) == spelling {
                return Ok(id);
            }
            slot = (slot + 1) & (self.slots.len() - 1);
        }
        if self.entries.len() >= self.slots.len() / 2 {
            self.grow();
            slot = self.vacant(hash);
        }
        let id = NameId(u32::try_from(self.entries.len()).map_err(|_| "too many names")?);
        let start = u32::try_from(self.bytes.len()).map_err(|_| "name arena exceeds 32 bits")?;
        let end = self
            .bytes
            .len()
            .checked_add(spelling.len())
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("name arena exceeds 32 bits")?;
        self.bytes.push_str(spelling);
        self.entries.push(Entry { start, end, hash });
        self.slots[slot] = Some(id);
        Ok(id)
    }
    fn grow(&mut self) {
        self.slots = vec![None; self.slots.len() * 2];
        for index in 0..self.entries.len() {
            let slot = self.vacant(self.entries[index].hash);
            // Every entry was checked before insertion.
            self.slots[slot] = Some(NameId(u32::try_from(index).unwrap_or(u32::MAX)));
        }
    }
    fn vacant(&self, hash: u64) -> usize {
        let mut slot = bucket(hash, self.slots.len());
        while self.slots[slot].is_some() {
            slot = (slot + 1) & (self.slots.len() - 1);
        }
        slot
    }
    #[must_use]
    pub fn get(&self, id: NameId) -> &str {
        let entry = self.entries[id.0 as usize];
        &self.bytes[entry.start as usize..entry.end as usize]
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }
}
fn bucket(hash: u64, capacity: usize) -> usize {
    // Mask before conversion so 32-bit hosts select the same low hash bits.
    usize::try_from(hash & u64::try_from(capacity - 1).unwrap_or(u64::MAX)).unwrap_or(0)
}
