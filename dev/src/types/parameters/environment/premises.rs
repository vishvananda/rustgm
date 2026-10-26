//! Ordered trait clauses retain binders without widening each ordinary predicate.
use super::super::premise_index::ClauseIndex;
use super::{Clause, ParameterId, Predicate};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{ItemId, Kind, Store, parameters::clauses::QuantifiedId, substitute};
use std::sync::{Arc, OnceLock};

#[derive(Clone)]
pub(super) struct Mapped {
    pub source: Arc<TraitAssumptions>,
    pub index: Arc<ClauseIndex>,
}

type ProjectionKey = (ItemId, ItemId);

#[derive(Clone, Default)]
pub(super) struct TraitAssumptions {
    pub region_mapping: bool,
    pub(super) opening: OnceLock<Arc<super::opening_summary::Summary>>,
    pub(super) mapped: Option<Arc<Mapped>>,
    index: OnceLock<Result<Arc<ClauseIndex>, &'static str>>,
    predicates: Vec<Predicate>,
    pub unique: HashSet<Predicate>,
    pub constants: HashSet<ParameterId>,
    projected: HashMap<ProjectionKey, Vec<usize>>,
    quantified: Option<Box<Quantifiers>>,
}

#[derive(Clone, Default)]
struct Quantifiers {
    // Each clause precedes the ordinary predicate at its recorded position.
    entries: Vec<(usize, QuantifiedId)>,
    unique: HashSet<QuantifiedId>,
    projected: HashMap<ProjectionKey, Vec<QuantifiedId>>,
}

fn projection(types: &Store, predicate: Predicate) -> Result<Option<ProjectionKey>, &'static str> {
    let Some(Kind::Projection(subject)) = types.kind(predicate.ty) else {
        return Ok(None);
    };
    let Some((id, _)) = predicate.bound.trait_parts() else {
        return Err("invalid projected trait assumption");
    };
    let trait_item = types
        .trait_application(id)
        .ok_or("missing assumed trait")?
        .item;
    Ok(Some((trait_item, subject.item)))
}

impl TraitAssumptions {
    pub(super) fn leaf_allocation_bound(clauses: usize) -> usize {
        size_of::<Self>()
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(clauses.saturating_mul(4 * size_of::<Predicate>()))
            .saturating_add(crate::support::retained::new_map::<Predicate, ()>(clauses))
    }

    pub(super) fn mapped(
        source: Arc<Self>,
        index: Arc<ClauseIndex>,
    ) -> Result<Arc<Self>, &'static str> {
        if source.mapped.is_some() || index.opening().is_none() {
            return Err("invalid canonical trait opening source");
        }
        Ok(Arc::new(Self {
            region_mapping: true,
            mapped: Some(Arc::new(Mapped { source, index })),
            ..Self::default()
        }))
    }

    pub(super) fn permits_opened_view(&self) -> bool {
        self.mapped.is_none() && self.quantified.is_none() && self.constants.is_empty()
    }

    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.vector(&self.predicates);
        total.set(&self.unique);
        total.set(&self.constants);
        total.map(&self.projected);
        for values in self.projected.values() {
            total.vector(values);
        }
        if let Some(q) = &self.quantified {
            total.boxed(q.as_ref());
            total.vector(&q.entries);
            total.set(&q.unique);
            total.map(&q.projected);
            for values in q.projected.values() {
                total.vector(values);
            }
        }
        if let Some(summary) = self.opening.get() {
            total.include(summary.allocation());
        }
        if let Some(mapped) = &self.mapped {
            total.boxed(mapped.as_ref());
            total.add(2 * size_of::<usize>());
        }
        total
    }
}
impl TraitAssumptions {
    pub(super) fn retain_index(
        &self,
        observer: &mut super::super::premise_index::Retained,
    ) -> crate::support::retained::Allocation {
        if let Some(mapped) = &self.mapped {
            return mapped.index.retain_allocation(observer);
        }
        self.index
            .get()
            .and_then(|result| result.as_ref().ok())
            .map_or_else(crate::support::retained::Allocation::default, |index| {
                index.retain_allocation(observer)
            })
    }
}
impl TraitAssumptions {
    pub fn index(&self, types: &Store) -> Result<(Arc<ClauseIndex>, usize), &'static str> {
        let mut work = 0;
        self.index_counted(types, &mut work)
            .map(|index| (index, work))
    }

    pub(super) fn index_counted(
        &self,
        types: &Store,
        work: &mut usize,
    ) -> Result<Arc<ClauseIndex>, &'static str> {
        *work = work.saturating_add(1);
        if let Some(mapped) = &self.mapped {
            mapped
                .index
                .opening()
                .ok_or("missing mapped trait opening")?
                .validate(types)?;
            return Ok(Arc::clone(&mapped.index));
        }
        let index = self
            .index
            .get_or_init(|| {
                let mut index = ClauseIndex::default();
                for clause in self.clauses() {
                    *work = work.saturating_add(1);
                    index
                        .insert_clause(types, clause)
                        .map_err(|_| "invalid caller trait assumption index")?;
                }
                Ok(Arc::new(index))
            })
            .clone()?;
        Ok(index)
    }

    pub(super) fn index_charged(
        &self,
        types: &Store,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Arc<ClauseIndex>, &'static str> {
        charge(1, 0)?;
        if let Some(mapped) = &self.mapped {
            mapped
                .index
                .opening()
                .ok_or("missing mapped trait opening")?
                .validate(types)?;
            return Ok(Arc::clone(&mapped.index));
        }
        if let Some(result) = self.index.get() {
            return result.clone();
        }
        // Retention failure is query-local: it must not become a permanent
        // semantic Err in the immutable source's OnceLock.
        charge(1, size_of::<ClauseIndex>() + 2 * size_of::<usize>())?;
        let mut index = ClauseIndex::default();
        for clause in self.clauses() {
            index.insert_clause_charged(types, clause, charge)?;
        }
        let index = Arc::new(index);
        if self.index.set(Ok(Arc::clone(&index))).is_ok() {
            return Ok(index);
        }
        self.index
            .get()
            .ok_or("caller trait index publication disappeared")?
            .clone()
    }

    pub fn is_empty(&self) -> bool {
        if let Some(mapped) = &self.mapped {
            return mapped.source.is_empty();
        }
        self.predicates.is_empty()
            && self
                .quantified
                .as_ref()
                .is_none_or(|q| q.entries.is_empty())
    }

    pub fn insert(&mut self, types: &Store, clause: Clause) -> Result<(), &'static str> {
        if self.mapped.is_some() {
            return Err("opened caller clauses must be materialized before extension");
        }
        self.region_mapping |= clause.has_free_regions(types)?;
        let count = self.unique.len() + self.quantified.as_ref().map_or(0, |q| q.unique.len());
        match clause {
            Clause::Predicate(predicate) => {
                if count >= 1_000_000 && !self.unique.contains(&predicate) {
                    return Err("trait assumption storage budget exceeded");
                }
                if self.unique.insert(predicate) {
                    self.index.take();
                    self.opening.take();
                    if let Some(key) = projection(types, predicate)? {
                        self.projected
                            .entry(key)
                            .or_default()
                            .push(self.predicates.len());
                    }
                    self.predicates.push(predicate);
                }
            }
            Clause::ForAll(id) => {
                let value = types
                    .quantified(id)
                    .ok_or("invalid quantified assumption")?;
                if value.predicate.bound.trait_parts().is_none() {
                    return Err("quantified environment assumptions require a user trait");
                }
                let quantified = self.quantified.get_or_insert_with(Default::default);
                if count >= 1_000_000 && !quantified.unique.contains(&id) {
                    return Err("trait assumption storage budget exceeded");
                }
                if quantified.unique.insert(id) {
                    self.index.take();
                    self.opening.take();
                    if let Some(key) = projection(types, value.predicate)? {
                        quantified.projected.entry(key).or_default().push(id);
                    }
                    quantified.entries.push((self.predicates.len(), id));
                }
            }
        }
        Ok(())
    }

    pub fn clauses(&self) -> impl Iterator<Item = Clause> {
        ordered(
            &self.predicates,
            self.quantified
                .as_ref()
                .map_or(&[], |q| q.entries.as_slice()),
        )
    }

    pub fn projected(&self, key: ProjectionKey) -> impl Iterator<Item = Clause> {
        self.projected
            .get(&key)
            .into_iter()
            .flatten()
            .map(|&index| Clause::Predicate(self.predicates[index]))
            .chain(
                self.quantified
                    .iter()
                    .flat_map(move |q| q.projected.get(&key).into_iter().flatten())
                    .copied()
                    .map(Clause::ForAll),
            )
    }

    pub fn substitute(
        &mut self,
        types: &mut Store,
        fold: &mut substitute::Bound,
    ) -> Result<(), &'static str> {
        self.substitute_observed::<false>(types, fold, &|_, _| Ok(()))
    }
    pub(super) fn substitute_charged(
        &mut self,
        types: &mut Store,
        fold: &mut substitute::Bound,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        self.substitute_observed::<true>(types, fold, charge)
    }
    fn substitute_observed<const CHARGED: bool>(
        &mut self,
        types: &mut Store,
        fold: &mut substitute::Bound,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        if CHARGED {
            charge(1, 0)?;
            if self.mapped.is_some() {
                return Err("charged declaration mapping requires an unopened caller environment");
            }
        }
        if let Some(mapped) = self.mapped.take() {
            let opening = mapped
                .index
                .opening()
                .ok_or("missing composed trait opening")?;
            let mut materialized = Self::default();
            for clause in mapped.source.clauses() {
                let Clause::Predicate(predicate) = clause else {
                    return Err("quantified trait opening requires full preparation");
                };
                let mut work = 0;
                let result = opening.substitute(types, predicate, &mut work);
                fold.account_region_summary_work(work)?;
                materialized.insert(types, result?.into())?;
            }
            *self = materialized;
        }
        self.index.take();
        self.opening.take();
        // Traverse mixed declaration order: fresh captured aliases must not depend
        // on storage kind, randomized hash order or duplicate canonical results.
        let mut position = 0;
        let mut region_mapping = false;
        let mut quantified = self
            .quantified
            .iter_mut()
            .flat_map(|q| &mut q.entries)
            .peekable();
        while position < self.predicates.len() || quantified.peek().is_some() {
            if quantified
                .peek()
                .is_some_and(|(index, _)| *index <= position)
            {
                let (_, id) = quantified
                    .next()
                    .ok_or("missing quantified environment entry")?;
                let mapped = if CHARGED {
                    charge(1, 0)?;
                    fold.clause_charged(types, Clause::ForAll(*id), charge)?
                } else {
                    fold.clause(types, Clause::ForAll(*id))?
                };
                let Clause::ForAll(mapped) = mapped else {
                    return Err("substitution flattened quantified environment clause");
                };
                *id = mapped;
                region_mapping |= Clause::ForAll(mapped).has_free_regions(types)?;
            } else {
                self.predicates[position] = if CHARGED {
                    charge(1, 0)?;
                    fold.clause_charged(types, self.predicates[position].into(), charge)?
                        .ordinary()?
                } else {
                    fold.predicate(types, self.predicates[position])?
                };
                region_mapping |= self.predicates[position].has_free_regions(types)?;
                position += 1;
            }
        }
        self.region_mapping = region_mapping;
        self.rebuild_observed::<CHARGED>(types, charge)
    }
    fn rebuild_observed<const CHARGED: bool>(
        &mut self,
        types: &Store,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        if CHARGED {
            self.prepay_rebuild(charge)?;
        }
        self.unique = self.predicates.iter().copied().collect();
        self.projected.clear();
        for (index, &predicate) in self.predicates.iter().enumerate() {
            if let Some(key) = projection(types, predicate)? {
                self.projected.entry(key).or_default().push(index);
            }
        }
        if let Some(q) = &mut self.quantified {
            q.unique.clear();
            q.projected.clear();
            q.entries.retain(|(_, id)| q.unique.insert(*id));
            for &(_, id) in &q.entries {
                let value = types
                    .quantified(id)
                    .ok_or("invalid mapped quantified assumption")?;
                if let Some(key) = projection(types, value.predicate)? {
                    q.projected.entry(key).or_default().push(id);
                }
            }
        }
        Ok(())
    }
}

/// Merge compact ordinary storage and sparse quantified entries in source order.
pub(super) fn ordered<'a>(
    predicates: &'a [Predicate],
    quantifiers: &'a [(usize, QuantifiedId)],
) -> impl Iterator<Item = Clause> + 'a {
    let mut ordinary = predicates.iter().enumerate().peekable();
    let mut quantified = quantifiers.iter().peekable();
    std::iter::from_fn(move || {
        if quantified
            .peek()
            .is_some_and(|(position, _)| ordinary.peek().is_none_or(|(index, _)| position <= index))
        {
            return quantified.next().map(|(_, id)| Clause::ForAll(*id));
        }
        ordinary.next().map(|(_, p)| Clause::Predicate(*p))
    })
}

impl TraitAssumptions {
    pub(super) fn prepay_clone(
        &self,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        use crate::support::retained;
        charge(1, size_of::<Self>() + 2 * size_of::<usize>())?;
        charge(self.predicates.len(), retained::vector(&self.predicates))?;
        charge(
            self.unique.len(),
            self.unique
                .capacity()
                .saturating_mul(size_of::<Predicate>()),
        )?;
        charge(
            self.constants.len(),
            self.constants
                .capacity()
                .saturating_mul(size_of::<ParameterId>()),
        )?;
        charge(self.projected.len(), retained::map(&self.projected))?;
        for values in self.projected.values() {
            charge(1 + values.len(), retained::vector(values))?;
        }
        if let Some(q) = &self.quantified {
            charge(1, size_of::<Quantifiers>())?;
            charge(q.entries.len(), retained::vector(&q.entries))?;
            charge(
                q.unique.len(),
                q.unique
                    .capacity()
                    .saturating_mul(size_of::<QuantifiedId>()),
            )?;
            charge(q.projected.len(), retained::map(&q.projected))?;
            for values in q.projected.values() {
                charge(1 + values.len(), retained::vector(values))?;
            }
        }
        Ok(())
    }
}

impl TraitAssumptions {
    fn prepay_rebuild(
        &self,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        use crate::support::retained::Growth;
        let n = self.predicates.len();
        let mut growth = Growth::map::<Predicate, ()>(0, 0, n);
        growth.include(Growth::map::<ProjectionKey, Vec<usize>>(
            0,
            self.projected.capacity(),
            n,
        ));
        // Every emitted projection bucket has at least one input row; four
        // slots per row bounds all geometric bucket buffers together.
        growth.bytes = growth
            .bytes
            .saturating_add(n.saturating_mul(4 * size_of::<usize>()));
        growth.work = growth.work.saturating_add(n);
        if let Some(q) = &self.quantified {
            growth.include(Growth::map::<QuantifiedId, ()>(
                0,
                q.unique.capacity(),
                q.entries.len(),
            ));
            growth.include(Growth::map::<ProjectionKey, Vec<QuantifiedId>>(
                0,
                q.projected.capacity(),
                q.entries.len(),
            ));
            growth.bytes = growth.bytes.saturating_add(
                q.entries
                    .len()
                    .saturating_mul(4 * size_of::<QuantifiedId>()),
            );
            growth.work = growth.work.saturating_add(q.entries.len());
        }
        charge(growth.work, growth.bytes)
    }
}
