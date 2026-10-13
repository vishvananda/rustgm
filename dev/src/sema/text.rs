//! Demanded text uses one canonical byte arena; ordinary literals retain lexer storage.
use std::{
    collections::{HashMap, hash_map::RandomState},
    hash::BuildHasher,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub u32);
struct Entry {
    start: u32,
    end: u32,
    next: Option<Id>,
}
#[derive(Default)]
pub struct Texts {
    bytes: Vec<u8>,
    entries: Vec<Entry>,
    buckets: HashMap<u64, Id>,
    hash: RandomState,
}
impl Texts {
    /// # Errors
    /// Rejects byte or identity arenas exceeding compact offsets.
    pub fn intern(&mut self, bytes: &[u8]) -> Result<Id, &'static str> {
        let hash = self.hash.hash_one(bytes);
        let head = self.buckets.get(&hash).copied();
        let mut current = head;
        while let Some(id) = current {
            if self.get(id)? == bytes {
                return Ok(id);
            }
            current = self.entries[id.0 as usize].next;
        }
        let id = Id(u32::try_from(self.entries.len()).map_err(|_| "too many constant texts")?);
        let start = u32::try_from(self.bytes.len()).map_err(|_| "text arena exceeds 32 bits")?;
        let end = bytes
            .len()
            .checked_add(self.bytes.len())
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("text arena exceeds 32 bits")?;
        self.bytes.extend_from_slice(bytes);
        self.entries.push(Entry {
            start,
            end,
            next: head,
        });
        self.buckets.insert(hash, id);
        Ok(id)
    }
    /// # Errors
    /// Rejects invalid text identities.
    pub fn get(&self, id: Id) -> Result<&[u8], &'static str> {
        let entry = self
            .entries
            .get(id.0 as usize)
            .ok_or("missing constant text")?;
        self.bytes
            .get(entry.start as usize..entry.end as usize)
            .ok_or("invalid constant text range")
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
