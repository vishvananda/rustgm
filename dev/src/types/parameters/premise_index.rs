//! Immutable clause lookup structure is shared independently of each query's proof.
use super::{
    ParameterId,
    clauses::{Clause, QuantifiedId},
    predicates::{Bound, Predicate},
};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{ItemId, Store, infer, subject_head::Head};
use std::sync::OnceLock;

#[derive(Clone, Default)]
pub(crate) struct ClauseIndex {
    pub candidates: HashMap<(ItemId, Head), Vec<Predicate>>,
    pub seen: HashSet<Predicate>,
    parameter_traits: OnceLock<HashMap<ParameterId, Vec<ItemId>>>,
    pub quantified: Option<Box<QuantifiedIndex>>,
}
#[derive(Clone, Default)]
pub(crate) struct QuantifiedIndex {
    pub candidates: HashMap<(ItemId, Head), Vec<QuantifiedId>>,
    pub seen: HashSet<QuantifiedId>,
}
/// The immutable index can gain its parameter lookup cache once.
#[derive(Default)]
pub(crate) struct Retained {
    base: bool,
    parameters: bool,
}
impl ClauseIndex {
    pub(crate) fn retain_allocation(
        &self,
        observer: &mut Retained,
    ) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        if !observer.base {
            observer.base = true;
            total.boxed(self);
            total.map(&self.candidates);
            total.set(&self.seen);
            for values in self.candidates.values() {
                total.vector(values);
            }
            if let Some(q) = &self.quantified {
                total.boxed(q.as_ref());
                total.map(&q.candidates);
                total.set(&q.seen);
                for values in q.candidates.values() {
                    total.vector(values);
                }
            }
        }
        total.visits = total.visits.saturating_add(1);
        if !observer.parameters
            && let Some(index) = self.parameter_traits.get()
        {
            observer.parameters = true;
            total.map(index);
            for values in index.values() {
                total.vector(values);
            }
        }
        total
    }
}
impl ClauseIndex {
    pub fn parameter_traits(&self, parameter: ParameterId) -> &[ItemId] {
        self.parameter_traits
            .get_or_init(|| {
                let mut index: HashMap<ParameterId, Vec<ItemId>> = HashMap::default();
                for &(item, head) in self.candidates.keys() {
                    if let Head::Parameter(parameter) = head {
                        index.entry(parameter).or_default().push(item);
                    }
                }
                if let Some(quantified) = &self.quantified {
                    for &(item, head) in quantified.candidates.keys() {
                        if let Head::Parameter(parameter) = head
                            && !self.candidates.contains_key(&(item, head))
                        {
                            index.entry(parameter).or_default().push(item);
                        }
                    }
                }
                index
            })
            .get(&parameter)
            .map_or(&[], Vec::as_slice)
    }
    pub fn insert_clause(&mut self, store: &Store, clause: Clause) -> Result<(), infer::Error> {
        let predicate = match clause {
            Clause::Predicate(predicate) => predicate,
            Clause::ForAll(id) => {
                store
                    .quantified(id)
                    .ok_or(infer::Error::InvalidIdentity)?
                    .predicate
            }
        };
        let Bound::Trait(id) = predicate.bound else {
            return Err(infer::Error::InvalidIdentity);
        };
        let application = store
            .trait_application(id)
            .ok_or(infer::Error::InvalidIdentity)?;
        if self
            .seen
            .len()
            .saturating_add(self.quantified.as_ref().map_or(0, |index| index.seen.len()))
            >= 1_000_000
        {
            return Err(infer::Error::Budget);
        }
        if let Clause::ForAll(id) = clause {
            let head = Head::canonical(
                store
                    .kind(predicate.ty)
                    .ok_or(infer::Error::InvalidIdentity)?,
            );
            let index = self.quantified.get_or_insert_with(Default::default);
            if index.seen.insert(id) {
                self.parameter_traits.take();
                index
                    .candidates
                    .entry((application.item, head))
                    .or_default()
                    .push(id);
            }
            return Ok(());
        }
        if self.seen.insert(predicate) {
            self.parameter_traits.take();
            let head = Head::canonical(
                store
                    .kind(predicate.ty)
                    .ok_or(infer::Error::InvalidIdentity)?,
            );
            self.candidates
                .entry((application.item, head))
                .or_default()
                .push(predicate);
        }
        Ok(())
    }
}
