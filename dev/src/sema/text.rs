//! Selected text literals and demanded constants share one canonical byte arena.
use crate::support::hash::{BuildFxHasher, HashMap};
use std::hash::{BuildHasher, Hash, Hasher};

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
    hash: BuildFxHasher,
}
impl Texts {
    /// # Errors
    /// Rejects byte or identity arenas exceeding compact offsets.
    pub fn intern(&mut self, bytes: &[u8]) -> Result<Id, &'static str> {
        self.intern_suffixed(bytes, None)
    }
    /// Intern decoded C string content with exactly one implicit trailing NUL.
    /// The original decoded bytes need no temporary owned copy.
    ///
    /// # Errors
    /// Rejects byte or identity arenas exceeding compact offsets.
    pub fn intern_c_string(&mut self, bytes: &[u8]) -> Result<Id, &'static str> {
        self.intern_suffixed(bytes, Some(0))
    }
    fn intern_suffixed(&mut self, bytes: &[u8], suffix: Option<u8>) -> Result<Id, &'static str> {
        let length = bytes
            .len()
            .checked_add(usize::from(suffix.is_some()))
            .ok_or("text arena exceeds host size")?;
        let hash = suffix.map_or_else(
            || self.hash.hash_one(bytes),
            |last| {
                // FxHasher consumes complete eight-byte words. Keep its final word
                // intact across the borrowed content/suffix boundary so this has
                // precisely the ordinary slice hash of the concatenated bytes.
                let split = bytes.len() / 8 * 8;
                let mut tail = [0; 8];
                let rest = &bytes[split..];
                tail[..rest.len()].copy_from_slice(rest);
                tail[rest.len()] = last;
                let mut state = self.hash.build_hasher();
                length.hash(&mut state);
                state.write(&bytes[..split]);
                state.write(&tail[..=rest.len()]);
                state.finish()
            },
        );
        let head = self.buckets.get(&hash).copied();
        let mut current = head;
        while let Some(id) = current {
            let stored = self.get(id)?;
            let equal = suffix.map_or_else(
                || stored == bytes,
                |last| {
                    stored
                        .split_last()
                        .is_some_and(|(byte, head)| *byte == last && head == bytes)
                },
            );
            if equal {
                return Ok(id);
            }
            current = self.entries[id.0 as usize].next;
        }
        let id = Id(u32::try_from(self.entries.len()).map_err(|_| "too many constant texts")?);
        let start = u32::try_from(self.bytes.len()).map_err(|_| "text arena exceeds 32 bits")?;
        let end = length
            .checked_add(self.bytes.len())
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("text arena exceeds 32 bits")?;
        self.bytes.extend_from_slice(bytes);
        if let Some(last) = suffix {
            self.bytes.push(last);
        }
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
