//! Small exact maps for per-operation inference memos.
use crate::support::hash::HashMap;

/// An exact map that stays a short vector while small: instantiations and
/// binder folds usually visit a few parameters and template nodes.
pub(super) enum Memo<K, V> {
    Linear(Vec<(K, V)>),
    Hashed(HashMap<K, V>),
}
impl<K, V> Default for Memo<K, V> {
    fn default() -> Self {
        Self::Linear(Vec::new())
    }
}
impl<K: Copy + Eq + std::hash::Hash, V: Copy> Memo<K, V> {
    const LINEAR: usize = 16;
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity > Self::LINEAR {
            Self::Hashed(HashMap::with_capacity_and_hasher(
                capacity,
                crate::support::hash::BuildFxHasher::default(),
            ))
        } else {
            Self::Linear(Vec::with_capacity(capacity))
        }
    }
    #[inline]
    pub fn get(&self, key: &K) -> Option<V> {
        match self {
            Self::Linear(entries) => entries
                .iter()
                .find(|(entry, _)| entry == key)
                .map(|&(_, value)| value),
            Self::Hashed(entries) => entries.get(key).copied(),
        }
    }
    /// Empty the map, keeping a small linear allocation for reuse.
    pub fn clear(&mut self) {
        match self {
            Self::Linear(entries) if entries.capacity() <= 4 * Self::LINEAR => entries.clear(),
            _ => *self = Self::default(),
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Linear(entries) => entries.is_empty(),
            Self::Hashed(entries) => entries.is_empty(),
        }
    }
    /// Insert or replace, returning the previous value.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self {
            Self::Linear(entries) => {
                if let Some(entry) = entries.iter_mut().find(|(entry, _)| *entry == key) {
                    return Some(std::mem::replace(&mut entry.1, value));
                }
                if entries.len() < Self::LINEAR {
                    entries.push((key, value));
                    return None;
                }
                let mut hashed = HashMap::with_capacity_and_hasher(
                    entries.len() * 2,
                    crate::support::hash::BuildFxHasher::default(),
                );
                hashed.extend(entries.drain(..));
                hashed.insert(key, value);
                *self = Self::Hashed(hashed);
                None
            }
            Self::Hashed(entries) => entries.insert(key, value),
        }
    }
}
