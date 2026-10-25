//! Structural targets share candidate proofs without fabricating nominal identities.
use super::Candidates;
use crate::{
    resolve::{
        model::{DefId, ItemContext},
        names::NameId,
    },
    support::hash::{HashMap, HashSet},
    types::{ItemId, subject_head::Head},
};

/// Only an exclusion key. Applicability still consumes the full Self equations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::sema) enum Bucket {
    Nominal(ItemId, NameId),
    Structural(Head, NameId),
}

impl From<(ItemId, NameId)> for Bucket {
    fn from((item, name): (ItemId, NameId)) -> Self {
        Self::Nominal(item, name)
    }
}

impl Bucket {
    pub fn declared(head: Head, name: NameId) -> Option<Self> {
        match head {
            Head::Nominal(item) => Some(Self::Nominal(item, name)),
            Head::Primitive(_)
            | Head::Reference(_)
            | Head::Pointer(_)
            | Head::Tuple
            | Head::Array
            | Head::Slice
            | Head::Function
            | Head::Never => Some(Self::Structural(head, name)),
            _ => None,
        }
    }

    /// rustc admits function-pointer impl declarations but excludes them from lookup.
    pub fn lookup(head: Head, name: NameId) -> Option<Self> {
        if head == Head::Function {
            None
        } else {
            Self::declared(head, name)
        }
    }
}

type Key = (Head, NameId);

/// One optional crate owner; ordinary nominal maps keep their existing compact keys.
#[derive(Default)]
pub(super) struct Structural {
    names: HashSet<NameId>,
    members: HashMap<Key, HashMap<ItemContext, Vec<DefId>>>,
    indices: HashMap<(Head, NameId, ItemContext), Candidates>,
}

impl Structural {
    pub fn publish(&mut self, key: Key, context: ItemContext, member: DefId) {
        self.names.insert(key.1);
        let contexts = self.members.entry(key).or_default();
        // Keep a root presence bucket even for a solely contextual member.
        contexts.entry(ItemContext::ROOT).or_default();
        contexts.entry(context).or_default().push(member);
        self.indices.remove(&(key.0, key.1, context));
    }

    pub fn freeze(&mut self) {
        for members in self.members.values_mut().flat_map(HashMap::values_mut) {
            members.sort_unstable_by_key(|member| member.0);
        }
    }

    pub fn contains(&self, key: Key) -> bool {
        self.members.contains_key(&key)
    }
    pub fn named(&self, name: NameId) -> bool {
        self.names.contains(&name)
    }

    pub fn contextual(&self, key: Key) -> bool {
        self.members
            .get(&key)
            .is_some_and(|contexts| contexts.len() > 1)
    }

    pub fn members(&self, key: Key, context: ItemContext) -> Option<&[DefId]> {
        self.members.get(&key)?.get(&context).map(Vec::as_slice)
    }

    pub fn index(&self, key: Key, context: ItemContext) -> Option<&Candidates> {
        self.indices.get(&(key.0, key.1, context))
    }

    pub fn retain(
        &mut self,
        key: Key,
        context: ItemContext,
        index: Candidates,
    ) -> Result<(), &'static str> {
        if self.members(key, context).is_none() {
            return Err("structural method index changed its declared identity");
        }
        self.indices.insert((key.0, key.1, context), index);
        Ok(())
    }
}
