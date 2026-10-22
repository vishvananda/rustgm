//! Resumable declaration bodies retain only their own slots, independent of crate size.
use crate::support::hash::HashMap;
use crate::{
    resolve::unit::{FileId, NodeKey},
    types::infer::TermId,
};
use std::ops::{Index, IndexMut};

pub(super) enum Terms {
    Dense(Vec<Vec<Option<TermId>>>),
    Sparse(HashMap<NodeKey, TermId>),
}
impl Default for Terms {
    fn default() -> Self {
        Self::Dense(Vec::new())
    }
}
impl Terms {
    pub fn capacity(&self) -> usize {
        match self {
            Self::Dense(files) => files.capacity() + files.iter().map(Vec::capacity).sum::<usize>(),
            Self::Sparse(values) => values.capacity(),
        }
    }
    pub fn sparse() -> Self {
        Self::Sparse(HashMap::default())
    }
    /// Keep the suspended owner's entries, without scanning unrelated file slots.
    pub fn retain_sparse(&mut self, keys: impl Iterator<Item = NodeKey>) {
        if matches!(self, Self::Dense(_)) {
            let values = keys
                .filter_map(|key| self.get(key).map(|value| (key, value)))
                .collect();
            *self = Self::Sparse(values);
        }
    }
    pub fn files(&mut self, count: usize) {
        if let Self::Dense(files) = self {
            files.resize_with(count, Vec::new);
        }
    }
    pub fn prepare(&mut self, file: FileId, count: usize) {
        if let Self::Dense(files) = self {
            files[file.0 as usize].resize(count, None);
        }
    }
    pub fn get(&self, key: NodeKey) -> Option<TermId> {
        match self {
            Self::Dense(files) => files[key.file.0 as usize][key.node.0 as usize],
            Self::Sparse(values) => values.get(&key).copied(),
        }
    }
    pub fn insert(&mut self, key: NodeKey, term: TermId) -> Option<TermId> {
        match self {
            Self::Dense(files) => files[key.file.0 as usize][key.node.0 as usize].replace(term),
            Self::Sparse(values) => values.insert(key, term),
        }
    }
    pub fn remove(&mut self, key: NodeKey) {
        match self {
            Self::Dense(files) => files[key.file.0 as usize][key.node.0 as usize] = None,
            Self::Sparse(values) => {
                values.remove(&key);
            }
        }
    }
}

enum Storage<T> {
    Dense(Vec<T>),
    Sparse(HashMap<usize, T>),
}
pub(super) struct Slots<T> {
    storage: Storage<T>,
    empty: T,
}
impl<T: Default> Default for Slots<T> {
    fn default() -> Self {
        Self {
            storage: Storage::Dense(Vec::new()),
            empty: T::default(),
        }
    }
}
impl<T: Default + Copy> Slots<T> {
    pub fn capacity(&self) -> usize {
        match &self.storage {
            Storage::Dense(values) => values.capacity(),
            Storage::Sparse(values) => values.capacity(),
        }
    }
    pub fn sparse() -> Self {
        Self {
            storage: Storage::Sparse(HashMap::default()),
            empty: T::default(),
        }
    }
    /// A continuation needs only its own touched bindings and expression states.
    pub fn retain_sparse(&mut self, keys: impl Iterator<Item = usize>) {
        if matches!(self.storage, Storage::Dense(_)) {
            let values = keys.map(|key| (key, self[key])).collect();
            self.storage = Storage::Sparse(values);
        }
    }
    pub fn resize(&mut self, count: usize, value: T) {
        if let Storage::Dense(values) = &mut self.storage {
            values.resize(count, value);
        }
    }
    pub fn remove(&mut self, index: usize) -> T {
        match &mut self.storage {
            Storage::Dense(values) => std::mem::take(&mut values[index]),
            Storage::Sparse(values) => values.remove(&index).unwrap_or_default(),
        }
    }
}
impl<T> Index<usize> for Slots<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        match &self.storage {
            Storage::Dense(values) => &values[index],
            Storage::Sparse(values) => values.get(&index).unwrap_or(&self.empty),
        }
    }
}
impl<T: Default> IndexMut<usize> for Slots<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        match &mut self.storage {
            Storage::Dense(values) => &mut values[index],
            Storage::Sparse(values) => values.entry(index).or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FileId, NodeKey, Slots, Storage, TermId, Terms};
    use crate::syntax::ast::NodeId;

    #[test]
    fn parked_body_slots_do_not_scale_with_crate_or_definition_indices() -> Result<(), &'static str>
    {
        for count in [1_u32, 16, 128] {
            let mut parked = Vec::new();
            for index in 0..count {
                let file = FileId(100_000 + index);
                let key = NodeKey {
                    file,
                    node: NodeId(10_000_000 + index),
                };
                let mut terms = Terms::sparse();
                terms.files(1_000_000);
                terms.prepare(file, 100_000_000);
                assert_eq!(terms.insert(key, TermId(index)), None);
                let mut locals = Slots::<Option<TermId>>::sparse();
                locals.resize(100_000_000, None);
                locals[index as usize + 50_000_000] = Some(TermId(index));
                parked.push((key, terms, locals));
            }
            for (index, (key, mut terms, mut locals)) in parked.into_iter().enumerate() {
                let Terms::Sparse(values) = &terms else {
                    return Err("resumable slots unexpectedly became dense");
                };
                let Storage::Sparse(bindings) = &locals.storage else {
                    return Err("resumable slots unexpectedly became dense");
                };
                assert_eq!(values.len(), 1);
                assert_eq!(bindings.len(), 1);
                assert!(values.capacity() <= 8 && bindings.capacity() <= 8);
                assert_eq!(terms.get(key), locals.remove(index + 50_000_000));
                terms.remove(key);
                assert_eq!(terms.get(key), None);
                assert_eq!(locals[index + 50_000_000], None);
            }
        }
        Ok(())
    }
}
