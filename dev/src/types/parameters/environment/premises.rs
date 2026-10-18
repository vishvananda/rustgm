//! Ordered trait clauses retain binders without widening each ordinary predicate.
use super::{Bound, Clause, ParameterId, Predicate};
use crate::types::{ItemId, Kind, Store, parameters::clauses::QuantifiedId, substitute};
use std::collections::{HashMap, HashSet};

type ProjectionKey = (ItemId, ItemId);

#[derive(Clone, Default)]
pub(super) struct TraitAssumptions {
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
    pub fn is_empty(&self) -> bool {
        self.predicates.is_empty()
            && self
                .quantified
                .as_ref()
                .is_none_or(|q| q.entries.is_empty())
    }

    pub fn insert(&mut self, types: &Store, clause: Clause) -> Result<(), &'static str> {
        let count = self.unique.len() + self.quantified.as_ref().map_or(0, |q| q.unique.len());
        match clause {
            Clause::Predicate(predicate) => {
                if count >= 1_000_000 && !self.unique.contains(&predicate) {
                    return Err("trait assumption storage budget exceeded");
                }
                if self.unique.insert(predicate) {
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
        let mut ordinary = self.predicates.iter().enumerate().peekable();
        let mut quantified = self.quantified.iter().flat_map(|q| &q.entries).peekable();
        std::iter::from_fn(move || {
            if quantified.peek().is_some_and(|(position, _)| {
                ordinary.peek().is_none_or(|(index, _)| position <= index)
            }) {
                return quantified.next().map(|(_, id)| Clause::ForAll(*id));
            }
            ordinary.next().map(|(_, p)| Clause::Predicate(*p))
        })
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
        // Traverse mixed declaration order: fresh captured aliases must not depend
        // on storage kind, randomized hash order or duplicate canonical results.
        let mut position = 0;
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
            } else {
                self.predicates[position] = fold.predicate(types, self.predicates[position])?;
                position += 1;
            }
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
