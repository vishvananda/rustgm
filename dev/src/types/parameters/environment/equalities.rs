//! Projection equalities retain complete binders and index their declaring member.
use super::{Bound, Clause, Predicate};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{ItemId, Kind, Store, parameters::clauses::QuantifiedId, substitute};

#[derive(Clone, Default)]
pub(super) struct Equalities {
    pub region_mapping: bool,
    predicates: Vec<Predicate>,
    unique: HashSet<Predicate>,
    items: HashMap<ItemId, Vec<usize>>,
    quantified: Option<Box<Quantifiers>>,
}

#[derive(Clone, Default)]
struct Quantifiers {
    entries: Vec<(usize, QuantifiedId)>,
    unique: HashSet<QuantifiedId>,
    items: HashMap<ItemId, Vec<QuantifiedId>>,
}

fn member(types: &Store, predicate: Predicate) -> Result<ItemId, &'static str> {
    let (Some(Kind::Projection(application)), Bound::ProjectionEqual(_)) =
        (types.kind(predicate.ty), predicate.bound)
    else {
        return Err("projection equality requires a projected subject and result");
    };
    Ok(application.item)
}

impl Equalities {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.vector(&self.predicates);
        total.set(&self.unique);
        total.map(&self.items);
        for values in self.items.values() {
            total.vector(values);
        }
        if let Some(q) = &self.quantified {
            total.boxed(q.as_ref());
            total.vector(&q.entries);
            total.set(&q.unique);
            total.map(&q.items);
            for values in q.items.values() {
                total.vector(values);
            }
        }
        total
    }
}
impl Equalities {
    pub fn insert(&mut self, types: &Store, clause: Clause) -> Result<(), &'static str> {
        self.region_mapping |= clause.has_free_regions(types)?;
        let count = self.unique.len() + self.quantified.as_ref().map_or(0, |q| q.unique.len());
        match clause {
            Clause::Predicate(predicate) => {
                let item = member(types, predicate)?;
                if count >= 1_000_000 && !self.unique.contains(&predicate) {
                    return Err("projection assumption storage budget exceeded");
                }
                if self.unique.insert(predicate) {
                    self.items
                        .entry(item)
                        .or_default()
                        .push(self.predicates.len());
                    self.predicates.push(predicate);
                }
            }
            Clause::ForAll(id) => {
                let value = types.quantified(id).ok_or("invalid quantified equality")?;
                let item = member(types, value.predicate)?;
                let q = self.quantified.get_or_insert_with(Default::default);
                if count >= 1_000_000 && !q.unique.contains(&id) {
                    return Err("projection assumption storage budget exceeded");
                }
                if q.unique.insert(id) {
                    q.items.entry(item).or_default().push(id);
                    q.entries.push((self.predicates.len(), id));
                }
            }
        }
        Ok(())
    }

    pub fn clauses(&self) -> impl Iterator<Item = Clause> {
        super::premises::ordered(
            &self.predicates,
            self.quantified
                .as_ref()
                .map_or(&[], |q| q.entries.as_slice()),
        )
    }

    pub fn candidates(&self, item: ItemId) -> impl Iterator<Item = Clause> {
        self.items
            .get(&item)
            .into_iter()
            .flatten()
            .map(|&index| Clause::Predicate(self.predicates[index]))
            .chain(
                self.quantified
                    .iter()
                    .flat_map(move |q| q.items.get(&item).into_iter().flatten())
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
            self.prepay_rebuild(charge)?;
        }
        if self.quantified.is_some() {
            // Preserve mixed declaration order when allocating captured aliases.
            let original = std::mem::take(self);
            for clause in original.clauses() {
                let mapped = if CHARGED {
                    charge(1, 0)?;
                    fold.clause_charged(types, clause, charge)?
                } else {
                    fold.clause(types, clause)?
                };
                self.insert(types, mapped)?;
            }
            return Ok(());
        }
        // Ordinary environments reuse their existing storage and compact entries.
        self.region_mapping = false;
        for predicate in &mut self.predicates {
            *predicate = if CHARGED {
                charge(1, 0)?;
                fold.clause_charged(types, (*predicate).into(), charge)?
                    .ordinary()?
            } else {
                fold.predicate(types, *predicate)?
            };
            self.region_mapping |= predicate.has_free_regions(types)?;
        }
        self.unique.clear();
        self.predicates
            .retain(|predicate| self.unique.insert(*predicate));
        self.items.clear();
        for (index, &predicate) in self.predicates.iter().enumerate() {
            self.items
                .entry(member(types, predicate)?)
                .or_default()
                .push(index);
        }
        Ok(())
    }
}

impl Equalities {
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
        charge(self.items.len(), retained::map(&self.items))?;
        for values in self.items.values() {
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
            charge(q.items.len(), retained::map(&q.items))?;
            for values in q.items.values() {
                charge(1 + values.len(), retained::vector(values))?;
            }
        }
        Ok(())
    }
    fn prepay_rebuild(
        &self,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        use crate::support::retained::Growth;
        let n = self.predicates.len();
        let fresh = self.quantified.is_some();
        let mut growth =
            Growth::map::<Predicate, ()>(0, if fresh { 0 } else { self.unique.capacity() }, n);
        growth.include(Growth::vector::<Predicate>(
            0,
            if fresh { 0 } else { self.predicates.capacity() },
            n,
        ));
        growth.include(Growth::map::<ItemId, Vec<usize>>(
            0,
            if fresh { 0 } else { self.items.capacity() },
            n,
        ));
        growth.bytes = growth
            .bytes
            .saturating_add(n.saturating_mul(4 * size_of::<usize>()));
        growth.work = growth.work.saturating_add(n);
        if let Some(q) = &self.quantified {
            let count = q.entries.len();
            growth.bytes = growth.bytes.saturating_add(size_of::<Quantifiers>());
            growth.include(Growth::vector::<(usize, QuantifiedId)>(0, 0, count));
            growth.include(Growth::map::<QuantifiedId, ()>(0, 0, count));
            growth.include(Growth::map::<ItemId, Vec<QuantifiedId>>(0, 0, count));
            growth.bytes = growth
                .bytes
                .saturating_add(count.saturating_mul(4 * size_of::<QuantifiedId>()));
            growth.work = growth.work.saturating_add(count);
        }
        charge(growth.work, growth.bytes)
    }
}
