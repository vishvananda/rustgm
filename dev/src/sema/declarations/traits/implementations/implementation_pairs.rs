//! Each actual coherence pair is checked once while demanded owners advance.
use super::DefId;
use crate::{support::hash::HashMap, types::parameters::predicates::Predicate};
type Pair = (Predicate, DefId, Predicate, DefId);
#[derive(Default)]
pub(super) struct Pairs {
    values: Vec<Option<Pair>>,
    owners: HashMap<DefId, Vec<usize>>,
}
impl Pairs {
    pub(super) fn append(&mut self, pairs: Vec<Pair>) {
        for pair @ (_, first, _, second) in pairs {
            let index = self.values.len();
            self.owners.entry(first).or_default().push(index);
            self.owners.entry(second).or_default().push(index);
            self.values.push(Some(pair));
        }
    }
    pub(super) fn peers(&self, owner: DefId) -> Vec<DefId> {
        self.owners
            .get(&owner)
            .into_iter()
            .flatten()
            .filter_map(|&index| {
                self.values[index].map(|(_, a, _, b)| if a == owner { b } else { a })
            })
            .collect()
    }
    pub(super) fn count(&self, owner: Option<DefId>) -> usize {
        owner.map_or(self.values.len(), |owner| {
            self.owners.get(&owner).map_or(0, Vec::len)
        })
    }
    pub(super) fn take(&mut self, owner: Option<DefId>) -> Vec<Pair> {
        if let Some(owner) = owner {
            self.owners
                .get(&owner)
                .into_iter()
                .flatten()
                .filter_map(|&index| self.values[index].take())
                .collect()
        } else {
            self.values.iter_mut().filter_map(Option::take).collect()
        }
    }
}
