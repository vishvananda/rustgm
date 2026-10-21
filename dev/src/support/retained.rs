//! Saturating logical payload charges for retained owned collection storage.
//!
//! These charges cover vector slots and hash-table entry capacity. Allocator
//! headers, hash control bytes and shared immutable `Arc`/`Rc` pointees are excluded;
//! this is a deterministic work/storage policy, not a heap-size measurement.
use std::collections::HashMap;

pub(crate) fn vector<T>(values: &Vec<T>) -> usize {
    values.capacity().saturating_mul(size_of::<T>())
}
pub(crate) fn map<K, V, S>(values: &HashMap<K, V, S>) -> usize {
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
