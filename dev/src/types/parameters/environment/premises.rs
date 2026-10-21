//! Ordered trait clauses retain binders without widening each ordinary predicate.
use super::super::premise_index::ClauseIndex;
use super::{Bound, Clause, ParameterId, Predicate};
use crate::types::{ItemId, Kind, Store, parameters::clauses::QuantifiedId, substitute};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, OnceLock},
};

type ProjectionKey = (ItemId, ItemId);

#[derive(Clone, Default)]
pub(super) struct TraitAssumptions {
    pub region_mapping: bool,
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
    let Bound::Trait(id) = predicate.bound else {
        return Err("invalid projected trait assumption");
    };
    let trait_item = types
        .trait_application(id)
        .ok_or("missing assumed trait")?
        .item;
    Ok(Some((trait_item, subject.item)))
}

impl TraitAssumptions {
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
        total
    }
}
impl TraitAssumptions {
    pub(super) fn retain_index(
        &self,
        observer: &mut super::super::premise_index::Retained,
    ) -> crate::support::retained::Allocation {
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
        let mut work = 1;
        let index = self
            .index
            .get_or_init(|| {
                let mut index = ClauseIndex::default();
                for clause in self.clauses() {
                    work += 1;
                    index
                        .insert_clause(types, clause)
                        .map_err(|_| "invalid caller trait assumption index")?;
                }
                Ok(Arc::new(index))
            })
            .clone()?;
        Ok((index, work))
    }

    pub fn is_empty(&self) -> bool {
        self.predicates.is_empty()
            && self
                .quantified
                .as_ref()
                .is_none_or(|q| q.entries.is_empty())
    }

    pub fn insert(&mut self, types: &Store, clause: Clause) -> Result<(), &'static str> {
        self.region_mapping |= clause.has_free_regions(types)?;
        let count = self.unique.len() + self.quantified.as_ref().map_or(0, |q| q.unique.len());
        match clause {
            Clause::Predicate(predicate) => {
                if count >= 1_000_000 && !self.unique.contains(&predicate) {
                    return Err("trait assumption storage budget exceeded");
                }
                if self.unique.insert(predicate) {
                    self.index.take();
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
                if !matches!(value.predicate.bound, Bound::Trait(_)) {
                    return Err("quantified environment assumptions require a user trait");
                }
                let quantified = self.quantified.get_or_insert_with(Default::default);
                if count >= 1_000_000 && !quantified.unique.contains(&id) {
                    return Err("trait assumption storage budget exceeded");
                }
                if quantified.unique.insert(id) {
                    self.index.take();
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
        self.index.take();
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
                let Clause::ForAll(mapped) = fold.clause(types, Clause::ForAll(*id))? else {
                    return Err("substitution flattened quantified environment clause");
                };
                *id = mapped;
                region_mapping |= Clause::ForAll(mapped).has_free_regions(types)?;
            } else {
                self.predicates[position] = fold.predicate(types, self.predicates[position])?;
                region_mapping |= self.predicates[position].has_free_regions(types)?;
                position += 1;
            }
        }
        self.region_mapping = region_mapping;
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
