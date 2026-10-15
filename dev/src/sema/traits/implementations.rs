//! Prepared impl headers stay unavailable to selection until conformance succeeds.
pub(in crate::sema) mod overlap;
use super::heads::Head;
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    types::{ItemId, Kind, Store, parameters::predicates::Predicate, substitute},
};
use std::collections::HashMap;
#[cfg(test)]
mod tests;

pub struct Implementation {
    pub owner: DefId,
    pub source: NodeKey,
    pub predicate: Predicate,
    /// Trait declaration identity paired with an override, or its checked default.
    pub methods: Vec<(DefId, Option<DefId>)>,
}
struct Entry {
    value: Implementation,
    validated: bool,
}
#[derive(Default)]
pub(in crate::sema) struct Implementations {
    entries: Vec<Entry>,
    owners: HashMap<DefId, usize>,
    traits: HashMap<ItemId, Vec<usize>>,
    heads: HashMap<(ItemId, Option<Head>), Vec<usize>>,
    open: HashMap<(ItemId, Option<Head>), Vec<usize>>,
    closed: HashMap<(ItemId, Predicate), Vec<usize>>,
    shapes: Option<Box<substitute::Bound>>,
    pairs: Vec<(usize, usize)>,
}
impl Implementations {
    pub fn prepare(
        &mut self,
        store: &mut Store,
        item: ItemId,
        value: Implementation,
    ) -> Result<(), &'static str> {
        if self.owners.contains_key(&value.owner) {
            return Err("trait implementation was prepared twice");
        }
        let index = self.entries.len();
        let kind = store
            .kind(value.predicate.ty)
            .ok_or("invalid implementation subject")?;
        let head = if matches!(kind, Kind::Parameter(_)) {
            None
        } else {
            Some(Head::canonical(kind))
        };
        let shape = if value.predicate.contains_parameters(store)? {
            None
        } else {
            Some(
                self.shapes
                    .get_or_insert_with(|| Box::new(substitute::Bound::shape()))
                    .predicate(store, value.predicate)?,
            )
        };
        let (first, second, third) = if let Some(shape) = shape {
            (
                self.closed
                    .get(&(item, shape))
                    .map_or(&[][..], Vec::as_slice),
                self.open.get(&(item, head)).map_or(&[][..], Vec::as_slice),
                self.open.get(&(item, None)).map_or(&[][..], Vec::as_slice),
            )
        } else if head.is_none() {
            (
                self.traits.get(&item).map_or(&[][..], Vec::as_slice),
                &[][..],
                &[][..],
            )
        } else {
            (
                self.heads.get(&(item, head)).map_or(&[][..], Vec::as_slice),
                self.heads.get(&(item, None)).map_or(&[][..], Vec::as_slice),
                &[][..],
            )
        };
        let count = first
            .len()
            .saturating_add(second.len())
            .saturating_add(third.len());
        if self.pairs.len().saturating_add(count) > 1_000_000 {
            return Err("trait implementation overlap pair budget exceeded");
        }
        self.pairs.extend(
            first
                .iter()
                .chain(second)
                .chain(third)
                .map(|&other| (other, index)),
        );
        self.owners.insert(value.owner, index);
        self.traits.entry(item).or_default().push(index);
        self.heads.entry((item, head)).or_default().push(index);
        if let Some(shape) = shape {
            self.closed.entry((item, shape)).or_default().push(index);
        } else {
            self.open.entry((item, head)).or_default().push(index);
        }
        self.entries.push(Entry {
            value,
            validated: false,
        });
        Ok(())
    }
    pub fn headers(&self) -> impl Iterator<Item = &Implementation> {
        self.entries.iter().map(|entry| &entry.value)
    }
    pub fn header(&self, owner: DefId) -> Option<&Implementation> {
        self.owners
            .get(&owner)
            .map(|&index| &self.entries[index].value)
    }
    pub fn take_overlap_pairs(&mut self) -> Vec<(Predicate, DefId, Predicate, DefId)> {
        std::mem::take(&mut self.pairs)
            .into_iter()
            .map(|(a, b)| {
                let a = &self.entries[a].value;
                let b = &self.entries[b].value;
                (a.predicate, a.owner, b.predicate, b.owner)
            })
            .collect()
    }
    pub fn validated(&self) -> impl Iterator<Item = &Implementation> {
        self.entries
            .iter()
            .filter(|entry| entry.validated)
            .map(|entry| &entry.value)
    }
    pub fn is_validated(&self, owner: DefId) -> bool {
        self.owners
            .get(&owner)
            .is_some_and(|&index| self.entries[index].validated)
    }
    pub fn complete(
        &mut self,
        owner: DefId,
        methods: Vec<(DefId, Option<DefId>)>,
    ) -> Result<(), &'static str> {
        let index = *self
            .owners
            .get(&owner)
            .ok_or("missing prepared trait implementation")?;
        let entry = &mut self.entries[index];
        if entry.validated {
            return Err("trait implementation was validated twice");
        }
        entry.value.methods = methods;
        entry.validated = true;
        Ok(())
    }
}
