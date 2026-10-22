//! Pending headers are scheduling dependencies, never implementation evidence.
mod collections;
mod progression_heads;
use super::{DefId, Head, Identity, Subject};
use crate::support::hash::{HashMap, HashSet};
use crate::types::infer::{Context, Error};
#[cfg(test)]
#[path = "../../../../tests/invariants/pending_headers.rs"]
mod tests;

#[derive(Clone, Copy)]
pub(in crate::sema) struct Header {
    pub owner: DefId,
    pub item: Identity,
    /// Unknown aliases, projections and generic subjects must stay possible.
    pub head: Option<Head>,
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::sema) enum Candidates {
    Ready(Vec<DefId>),
    Pending(Vec<DefId>),
}

#[derive(Default)]
pub(super) struct Pending {
    collections: Option<Box<collections::Heads>>,
    owners: HashMap<DefId, (Identity, Option<Head>)>,
    traits: HashMap<Identity, HashSet<DefId>>,
    heads: HashMap<(Identity, Option<Head>), HashSet<DefId>>,
}
impl Pending {
    pub(super) fn new(headers: Vec<Header>) -> Result<Self, &'static str> {
        let mut pending = Self::default();
        for Header { owner, item, head } in headers {
            if pending.owners.insert(owner, (item, head)).is_some() {
                return Err("implementation candidate was reserved twice");
            }
            pending.traits.entry(item).or_default().insert(owner);
            pending.heads.entry((item, head)).or_default().insert(owner);
        }
        Ok(pending)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.owners.is_empty()
    }

    pub(super) fn validate(
        &self,
        owner: DefId,
        item: impl Into<Identity>,
        head: Option<Head>,
    ) -> Result<(), &'static str> {
        let &(reserved_item, reserved_head) = self
            .owners
            .get(&owner)
            .ok_or("implementation header was not reserved")?;
        if item.into() != reserved_item
            || reserved_head.is_some_and(|reserved| Some(reserved) != head)
        {
            return Err("implementation header disagrees with its reserved subject");
        }
        Ok(())
    }

    pub(super) fn complete(&mut self, owner: DefId) {
        if let Some((item, head)) = self.owners.remove(&owner) {
            Self::remove(&mut self.traits, item, owner);
            Self::remove(&mut self.heads, (item, head), owner);
        }
    }

    fn remove<K: Eq + std::hash::Hash>(
        index: &mut HashMap<K, HashSet<DefId>>,
        key: K,
        owner: DefId,
    ) {
        if let std::collections::hash_map::Entry::Occupied(mut entry) = index.entry(key) {
            entry.get_mut().remove(&owner);
            if entry.get().is_empty() {
                entry.remove();
            }
        }
    }

    pub(super) fn may_have_subject(&self, item: impl Into<Identity>, subject: Subject) -> bool {
        let item = item.into();
        self.heads.contains_key(&(item, None))
            || matches!(subject, Subject::Rigid(Head::Projection(_)))
                && self.traits.contains_key(&item)
            || subject
                .heads()
                .any(|head| self.heads.contains_key(&(item, Some(head))))
    }

    pub(super) fn candidates(
        &self,
        context: &mut Context,
        item: impl Into<Identity>,
        subject: Option<Subject>,
    ) -> Result<Vec<DefId>, Error> {
        let item = item.into();
        let mut result = Vec::new();
        match subject {
            None | Some(Subject::Rigid(Head::Projection(_))) => {
                Self::append(context, self.traits.get(&item), &mut result)?;
            }
            Some(subject) => {
                Self::append(context, self.heads.get(&(item, None)), &mut result)?;
                for head in subject.heads() {
                    Self::append(context, self.heads.get(&(item, Some(head))), &mut result)?;
                }
            }
        }
        result.sort_unstable_by_key(|id| id.0);
        Ok(result)
    }

    fn append(
        context: &mut Context,
        bucket: Option<&HashSet<DefId>>,
        result: &mut Vec<DefId>,
    ) -> Result<(), Error> {
        context.account_work(1 + bucket.map_or(0, HashSet::len))?;
        result.extend(bucket.into_iter().flatten().copied());
        Ok(())
    }
}
