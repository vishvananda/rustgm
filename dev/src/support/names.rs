//! Canonical names occupy one byte arena; repeated ASCII lookups allocate nothing.
use crate::support::hash::BuildFxHasher;
use std::hash::BuildHasher;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NameId(pub u32);
#[derive(Clone, Copy)]
struct Entry {
    start: u32,
    end: u32,
    hash: u64,
}
/// Open-addressed slot: high hash bits beside `NameId + 1`; zero is vacant.
/// Probes reject most mismatches without touching the entry or byte arenas.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Slot(u64);
impl Slot {
    const fn new(hash: u64, id: u32) -> Self {
        Self((hash & TAG_MASK) | (id as u64 + 1))
    }
    const fn id(self) -> Option<NameId> {
        match (self.0 & !TAG_MASK).checked_sub(1) {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "the low half stores a 32-bit identity plus one"
            )]
            Some(id) => Some(NameId(id as u32)),
            None => None,
        }
    }
    const fn tag(self) -> u64 {
        self.0 & TAG_MASK
    }
}
const TAG_MASK: u64 = 0xffff_ffff_0000_0000;
#[derive(Default)]
pub struct Names {
    bytes: String,
    entries: Vec<Entry>,
    slots: Vec<Slot>,
    hash: BuildFxHasher,
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
        self.intern_plain(spelling)
    }
    /// Intern a spelling that has no raw prefix or lifetime quote.
    ///
    /// # Errors
    /// Rejects a name arena that cannot be indexed with 32-bit identities.
    #[inline]
    pub fn intern_plain(&mut self, spelling: &str) -> Result<NameId, &'static str> {
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
            self.slots.resize(16, Slot::default());
        }
        let hash = self.hash.hash_one(spelling);
        let tag = hash & TAG_MASK;
        let mask = self.slots.len() - 1;
        let mut slot = bucket(hash, self.slots.len());
        while let Some(id) = self.slots[slot].id() {
            if self.slots[slot].tag() == tag
                && let Some(entry) = self.entries.get(id.0 as usize)
                && self
                    .bytes
                    .as_bytes()
                    .get(entry.start as usize..entry.end as usize)
                    == Some(spelling.as_bytes())
            {
                return Ok(id);
            }
            slot = (slot + 1) & mask;
        }
        if self.entries.len() >= self.slots.len() / 2 {
            self.grow();
            slot = self.vacant(hash);
        }
        let index = u32::try_from(self.entries.len())
            .ok()
            .filter(|index| *index < u32::MAX)
            .ok_or("too many names")?;
        let start = u32::try_from(self.bytes.len()).map_err(|_| "name arena exceeds 32 bits")?;
        let end = self
            .bytes
            .len()
            .checked_add(spelling.len())
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("name arena exceeds 32 bits")?;
        self.bytes.push_str(spelling);
        self.entries.push(Entry { start, end, hash });
        self.slots[slot] = Slot::new(hash, index);
        Ok(NameId(index))
    }
    fn grow(&mut self) {
        self.slots = vec![Slot::default(); self.slots.len() * 2];
        for index in 0..self.entries.len() {
            let hash = self.entries[index].hash;
            let slot = self.vacant(hash);
            // Every entry was checked before insertion.
            self.slots[slot] = Slot::new(hash, u32::try_from(index).unwrap_or(u32::MAX - 1));
        }
    }
    fn vacant(&self, hash: u64) -> usize {
        let mut slot = bucket(hash, self.slots.len());
        while self.slots[slot].id().is_some() {
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
    // Multiplicative hashes mix every input byte into their high bits, while
    // the low bits only see low input bits: spellings sharing a prefix and a
    // constant byte position (`item_10`..`item_99`) would form one long probe
    // cluster. Select the slot from the top bits; the power-of-two capacity
    // keeps the index below 32 bits on every host.
    let bits = capacity.trailing_zeros();
    if bits == 0 {
        return 0;
    }
    usize::try_from(hash >> (64 - bits)).unwrap_or(0)
}
