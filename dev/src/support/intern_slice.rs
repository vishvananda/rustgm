//! Canonical typed slices in flat storage. Collisions compare the original IDs.
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};

struct Entry {
    start: u32,
    end: u32,
    next: Option<u32>,
}

pub(crate) struct Slices<T> {
    values: Vec<T>,
    entries: Vec<Entry>,
    index: HashMap<u64, u32>,
}

fn fingerprint<T: Hash>(values: &[T]) -> u64 {
    let mut hasher = DefaultHasher::new();
    values.hash(&mut hasher);
    hasher.finish()
}

impl<T: Hash> Default for Slices<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            entries: vec![Entry {
                start: 0,
                end: 0,
                next: None,
            }],
            index: HashMap::from([(fingerprint::<T>(&[]), 0)]),
        }
    }
}

impl<T: Copy + Eq + Hash> Slices<T> {
    pub(crate) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.vector(&self.values);
        total.vector(&self.entries);
        total.map(&self.index);
        total
    }

    pub fn get(&self, id: u32) -> Option<&[T]> {
        let entry = self.entries.get(id as usize)?;
        self.values.get(entry.start as usize..entry.end as usize)
    }

    pub fn edge_count(&self) -> usize {
        self.values.len()
    }

    pub fn find(&self, values: &[T]) -> Option<u32> {
        self.find_from(values, self.index.get(&fingerprint(values)).copied())
    }

    fn find_from(&self, values: &[T], mut next: Option<u32>) -> Option<u32> {
        while let Some(id) = next {
            if self.get(id) == Some(values) {
                return Some(id);
            }
            next = self.entries[id as usize].next;
        }
        None
    }

    pub fn intern(&mut self, values: &[T]) -> Result<u32, &'static str> {
        let hash = fingerprint(values);
        let head = self.index.get(&hash).copied();
        if let Some(id) = self.find_from(values, head) {
            return Ok(id);
        }
        let id = u32::try_from(self.entries.len()).map_err(|_| "too many canonical lists")?;
        let start = u32::try_from(self.values.len()).map_err(|_| "too many canonical edges")?;
        let end = self
            .values
            .len()
            .checked_add(values.len())
            .and_then(|size| u32::try_from(size).ok())
            .ok_or("too many canonical edges")?;
        self.values.extend_from_slice(values);
        self.entries.push(Entry {
            start,
            end,
            next: head,
        });
        self.index.insert(hash, id);
        Ok(id)
    }
}
