//! Immutable clause lookup structure is shared independently of each query's proof.
use super::{
    ParameterId,
    clauses::{Clause, QuantifiedId},
    predicates::Predicate,
};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{ItemId, Store, infer, subject_head::Head};
use std::sync::OnceLock;
mod opening;
pub(crate) use opening::Opening;
use std::sync::Arc;

#[derive(Clone)]
struct View {
    base: Arc<ClauseIndex>,
    opening: Arc<Opening>,
}

#[derive(Clone, Default)]
pub(crate) struct ClauseIndex {
    view: Option<Box<View>>,
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
    mapped: Option<Box<HashMap<usize, usize>>>,
}
impl ClauseIndex {
    pub(crate) fn retain_allocation(
        &self,
        observer: &mut Retained,
    ) -> crate::support::retained::Allocation {
        if let Some(view) = &self.view {
            let mut total = view.base.retain_allocation(observer);
            let mut own = view.opening.allocation();
            own.boxed(self);
            own.boxed(view.as_ref());
            own.add(2 * size_of::<usize>());
            let key = std::ptr::from_ref(self) as usize;
            let previous = observer
                .mapped
                .get_or_insert_with(Default::default)
                .entry(key)
                .or_default();
            total.bytes = total
                .bytes
                .saturating_add(own.bytes.saturating_sub(*previous));
            total.visits = total.visits.saturating_add(own.visits).saturating_add(1);
            *previous = (*previous).max(own.bytes);
            return total;
        }
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
    pub(in crate::types::parameters) fn opened(
        base: Arc<Self>,
        opening: Arc<Opening>,
        types: &Store,
        work: &mut usize,
    ) -> Result<Arc<Self>, &'static str> {
        *work = work.saturating_add(1);
        if base.view.is_some() || base.quantified.is_some() {
            return Err("trait opening requires an ordinary canonical index");
        }
        opening.require_base(types, &base, work)?;
        Ok(Arc::new(Self {
            view: Some(Box::new(View { base, opening })),
            ..Self::default()
        }))
    }

    pub(crate) fn canonical(&self) -> &Self {
        self.view.as_ref().map_or(self, |view| &view.base)
    }

    pub(crate) fn opening(&self) -> Option<&Arc<Opening>> {
        self.view.as_ref().map(|view| &view.opening)
    }

    pub fn parameter_traits(&self, parameter: ParameterId) -> &[ItemId] {
        if let Some(view) = &self.view {
            return view.base.parameter_traits(parameter);
        }
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
        if self.view.is_some() {
            return Err(infer::Error::InvalidIdentity);
        }
        let predicate = match clause {
            Clause::Predicate(predicate) => predicate,
            Clause::ForAll(id) => {
                store
                    .quantified(id)
                    .ok_or(infer::Error::InvalidIdentity)?
                    .predicate
            }
        };
        let Some((id, _)) = predicate.bound.trait_parts() else {
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
