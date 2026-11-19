//! Ordered candidate sets from the validated registry, with bounded reuse for wide lists.
use super::{DefId, Identity, ItemId, ShapeKey, Subject};
use crate::support::hash::FxHasher;
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Clone, Debug)]
pub(in crate::sema) struct Owners(Storage);

#[derive(Clone, Debug)]
enum Storage {
    Inline(Inline),
    Shared(Arc<Vec<DefId>>),
}
#[derive(Clone, Debug)]
struct Inline {
    ids: [DefId; 4],
    len: u8,
}
impl Default for Inline {
    fn default() -> Self {
        Self {
            ids: [DefId(0); 4],
            len: 0,
        }
    }
}
impl Default for Owners {
    fn default() -> Self {
        Self(Storage::Inline(Inline::default()))
    }
}
impl PartialEq for Owners {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl Eq for Owners {}
impl Owners {
    pub fn as_slice(&self) -> &[DefId] {
        match &self.0 {
            Storage::Inline(small) => &small.ids[..usize::from(small.len)],
            Storage::Shared(ids) => ids,
        }
    }

    pub(super) const fn share_width(len: usize) -> bool {
        len >= 16
    }
}

#[derive(Default)]
pub(super) struct Collected {
    small: Inline,
    wide: Vec<DefId>,
}
impl Collected {
    pub(super) fn push(&mut self, owner: DefId) {
        if self.wide.is_empty() {
            if let Some(slot) = self.small.ids.get_mut(usize::from(self.small.len)) {
                *slot = owner;
                self.small.len += 1;
                return;
            }
            self.wide.extend(self.small.ids);
        }
        self.wide.push(owner);
    }

    pub(super) fn sorted(mut self) -> Owners {
        if self.wide.is_empty() {
            self.small.ids[..usize::from(self.small.len)].sort_unstable_by_key(|id| id.0);
            return Owners(Storage::Inline(self.small));
        }
        self.wide.sort_unstable_by_key(|id| id.0);
        Owners(Storage::Shared(Arc::new(self.wide)))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Lookup {
    pub(super) item: Identity,
    pub(super) subject: Subject,
    pub(super) shape: Option<ShapeKey>,
    pub(super) argument: Option<ItemId>,
}

/// Cache entries belong to one validated registry view. Replacing a slot or
/// publishing a header leaves the ordered list held by an existing frame intact.
pub(super) struct Cache {
    slots: [Option<(Lookup, Arc<Vec<DefId>>)>; 256],
}
impl Cache {
    fn slot(key: Lookup) -> usize {
        let mut hasher = FxHasher::default();
        key.hash(&mut hasher);
        usize::from(hasher.finish().to_be_bytes()[0])
    }

    pub(super) fn get(&self, key: Lookup) -> Option<Owners> {
        let (recorded, owners) = self.slots[Self::slot(key)].as_ref()?;
        (*recorded == key).then(|| Owners(Storage::Shared(Arc::clone(owners))))
    }
}

pub(super) fn retain(cache: &mut Option<Box<Cache>>, key: Lookup, owners: &Owners) {
    // Bound the pool to 256 lists of at most 512 owner slots each. Large or
    // common short lists stay local to their search.
    let Storage::Shared(owners) = &owners.0 else {
        return;
    };
    if !Owners::share_width(owners.len()) || owners.capacity() > 512 {
        return;
    }
    let cache = cache.get_or_insert_with(|| {
        Box::new(Cache {
            slots: std::array::from_fn(|_| None),
        })
    });
    cache.slots[Cache::slot(key)] = Some((key, Arc::clone(owners)));
}
