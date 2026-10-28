//! Immutable clause lookup structure is shared independently of each query's proof.
use super::{
    ParameterId,
    clauses::{Clause, QuantifiedId},
    predicates::Predicate,
};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{ItemId, Store, infer, subject_head::Head};
use std::sync::OnceLock;
mod features;
mod indexed_clauses;
mod opening;
pub(crate) use opening::Opening;
use std::sync::Arc;

#[derive(Clone)]
struct View {
    base: Arc<ClauseIndex>,
    opening: Option<Arc<Opening>>,
}

#[derive(Clone, Default)]
pub(crate) struct ClauseIndex {
    pub(super) features: Option<Arc<features::Set>>,
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
    pub(in crate::types::parameters) const fn opening_allocation_bytes() -> usize {
        size_of::<Self>() + size_of::<View>() + 2 * size_of::<usize>()
    }

    pub(crate) fn retain_allocation(
        &self,
        observer: &mut Retained,
    ) -> crate::support::retained::Allocation {
        if let Some(view) = &self.view {
            let mut total = view.base.retain_allocation(observer);
            let mut own = view
                .opening
                .as_ref()
                .map_or_else(Default::default, |opening| opening.allocation());
            own.include(self.payload_allocation());
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
            if let Some(features) = &self.features {
                total.boxed(features.as_ref());
                total.add(2 * size_of::<usize>());
                total.set(&features.values);
            }
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
            features: base.features.clone(),
            view: Some(Box::new(View {
                base,
                opening: Some(opening),
            })),
            ..Self::default()
        }))
    }

    #[cfg(test)]
    pub(crate) fn canonical(&self) -> &Self {
        self.view
            .as_ref()
            .filter(|view| view.opening.is_some())
            .map_or(self, |view| &view.base)
    }

    pub(crate) fn opening(&self) -> Option<&Arc<Opening>> {
        self.view.as_ref().and_then(|view| view.opening.as_ref())
    }

    pub fn parameter_traits(&self, parameter: ParameterId) -> &[ItemId] {
        if let Some(view) = &self.view
            && view.opening.is_some()
        {
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
                if let Some(view) = &self.view {
                    for (&parameter, items) in &mut index {
                        for &item in view.base.parameter_traits(parameter) {
                            if !items.contains(&item) {
                                items.push(item);
                            }
                        }
                    }
                }
                index
            })
            .get(&parameter)
            .map_or_else(
                || {
                    self.view
                        .as_ref()
                        .map_or(&[][..], |view| view.base.parameter_traits(parameter))
                },
                Vec::as_slice,
            )
    }
    pub fn insert_clause(&mut self, store: &Store, clause: Clause) -> Result<(), infer::Error> {
        self.insert_observed::<false, infer::Error>(store, clause, &|_, _| Ok(()))
    }
}

impl ClauseIndex {
    /// Extend a private query without cloning or reindexing the immutable prefix.
    /// Added clauses are already in the requesting query's lifetime namespace.
    pub(crate) fn extended(
        base: Option<Arc<Self>>,
        types: &Store,
        clauses: &[Predicate],
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Arc<Self>, &'static str> {
        charge(1, size_of::<Self>() + 2 * size_of::<usize>())?;
        let mut result = Self::default();
        for &clause in clauses {
            result.insert_clause_charged(types, clause.into(), charge)?;
        }
        if let Some(base) = base {
            result.features.clone_from(&base.features);
            charge(1, size_of::<View>())?;
            result.view = Some(Box::new(View {
                base,
                opening: None,
            }));
        }
        Ok(Arc::new(result))
    }

    /// An opening belongs only to its actual canonical prefix. Identity query
    /// extensions never apply that opening to their newly introduced premises.
    pub(crate) fn layers(&self) -> impl Iterator<Item = (&Self, Option<&Arc<Opening>>)> {
        let mut next = Some(self);
        std::iter::from_fn(move || {
            let current = next.take()?;
            if let Some(view) = &current.view {
                if let Some(opening) = &view.opening {
                    return Some((view.base.as_ref(), Some(opening)));
                }
                next = Some(view.base.as_ref());
            }
            Some((current, None))
        })
    }

    fn payload_allocation(&self) -> crate::support::retained::Allocation {
        let mut own = crate::support::retained::Allocation::default();
        own.map(&self.candidates);
        own.set(&self.seen);
        for values in self.candidates.values() {
            own.vector(values);
        }
        if let Some(index) = self.parameter_traits.get() {
            own.map(index);
            for values in index.values() {
                own.vector(values);
            }
        }
        own
    }
}
