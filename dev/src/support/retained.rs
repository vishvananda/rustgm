//! Saturating logical payload charges for retained owned collection storage.
//!
//! These charges cover vector slots and hash-table entry capacity. Allocator
//! headers, hash control bytes and shared immutable `Arc`/`Rc` pointees are excluded;
//! this is a deterministic work/storage policy, not a heap-size measurement.

pub(crate) fn vector<T>(values: &Vec<T>) -> usize {
    values.capacity().saturating_mul(size_of::<T>())
}
pub(crate) fn map<K, V, S>(values: &std::collections::HashMap<K, V, S>) -> usize {
    values.capacity().saturating_mul(size_of::<(K, V)>())
}
/// Reserve a conservative entry-slot charge before constructing a new map.
/// Four slots per entry covers minimum capacity and geometric growth in the
/// pinned standard library without allocating a throwaway table to inspect it.
pub(crate) const fn new_map<K, V>(entries: usize) -> usize {
    entries
        .saturating_mul(4)
        .saturating_mul(size_of::<(K, V)>())
}
pub(crate) fn sum(parts: impl IntoIterator<Item = usize>) -> usize {
    parts.into_iter().fold(0, usize::saturating_add)
}

/// Allocation/copy work before a collection changes capacity. No values or
/// semantic lookup results are exposed by this cost-only record.
#[derive(Clone, Copy, Default)]
pub(crate) struct Growth {
    pub bytes: usize,
    pub work: usize,
}
impl Growth {
    pub fn include(&mut self, other: Self) {
        self.bytes = self.bytes.saturating_add(other.bytes);
        self.work = self.work.saturating_add(other.work);
    }
    pub fn vector<T>(length: usize, capacity: usize, additional: usize) -> Self {
        let element = size_of::<T>();
        let needed = length.saturating_add(additional);
        if element == 0 || needed <= capacity {
            return Self::default();
        }
        // Byte vectors can start with eight slots; other nonzero elements have
        // a minimum no greater than four in the pinned standard library.
        let minimum = if element == 1 { 8 } else { 4 };
        Self {
            bytes: needed
                .saturating_mul(4)
                .max(minimum)
                .saturating_mul(element),
            work: capacity,
        }
    }
    pub fn map<K, V>(length: usize, capacity: usize, additional: usize) -> Self {
        let needed = length.saturating_add(additional);
        if needed <= capacity {
            return Self::default();
        }
        Self {
            bytes: new_map::<K, V>(needed),
            work: length.saturating_add(needed.saturating_mul(4)),
        }
    }
}

/// Capacity accounting visits collections, not their elements or allocator metadata.
#[derive(Clone, Copy, Default)]
pub(crate) struct Allocation {
    pub bytes: usize,
    pub visits: usize,
}
impl Allocation {
    pub fn add(&mut self, bytes: usize) {
        self.bytes = self.bytes.saturating_add(bytes);
        self.visits = self.visits.saturating_add(1);
    }
    pub fn include(&mut self, other: Self) {
        self.bytes = self.bytes.saturating_add(other.bytes);
        self.visits = self.visits.saturating_add(other.visits);
    }
    pub fn vector<T>(&mut self, values: &Vec<T>) {
        self.add(vector(values));
    }
    pub fn map<K, V, S>(&mut self, values: &std::collections::HashMap<K, V, S>) {
        self.add(map(values));
    }
    pub fn set<T, S>(&mut self, values: &std::collections::HashSet<T, S>) {
        self.add(values.capacity().saturating_mul(size_of::<T>()));
    }
    pub fn deque<T>(&mut self, values: &std::collections::VecDeque<T>) {
        self.add(values.capacity().saturating_mul(size_of::<T>()));
    }
    pub fn boxed<T>(&mut self, value: &T) {
        self.add(size_of_val(value));
    }
}
