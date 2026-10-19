//! Close declaration rules once, then select candidates before mapping caller inputs.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        ItemId,
        parameters::{
            clauses::{Clause, Quantifier},
            predicates::Bound,
        },
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Default)]
pub(super) struct Index {
    candidates: HashMap<ItemId, Vec<Clause>>,
    equalities: HashMap<ItemId, Vec<Clause>>,
    pub(super) outlives: Vec<crate::types::RegionId>,
}
impl Index {
    pub(super) fn candidates(&self, item: ItemId) -> Option<&[Clause]> {
        self.candidates.get(&item).map(Vec::as_slice)
    }
    pub(super) fn equalities(&self, item: ItemId) -> Option<&[Clause]> {
        self.equalities.get(&item).map(Vec::as_slice)
    }
}

pub(super) fn for_result(
    facts: &mut Declarations,
    item: DefId,
) -> Result<(Arc<Index>, usize), String> {
    if let Some(index) = facts
        .generics
        .traits
        .as_ref()
        .and_then(|contracts| contracts.result_rules.templates.get(&item))
    {
        return Ok((Arc::clone(index), 1));
    }
    let contract = facts
        .associated
        .type_contract(item)
        .ok_or("missing associated result contract")?;
    let (index, steps) = Closure::new(contract).complete(facts)?;
    let index = Arc::new(index);
    let cache = &mut facts
        .generics
        .traits
        .as_mut()
        .ok_or("missing result rule contracts")?
        .result_rules;
    if cache.enabled {
        cache.account(steps)?;
        cache.templates.insert(item, Arc::clone(&index));
    }
    Ok((index, steps))
}

/// Close finite declaration consequences once; recursive result discovery remains lazy.
struct Closure {
    template: crate::types::TypeId,
    pending: Vec<Clause>,
    index: Index,
    binders: HashMap<u32, Quantifier>,
    steps: usize,
}
impl Closure {
    fn new(contract: &crate::sema::associated::TypeContract) -> Self {
        let template = contract.projection;
        let pending = contract
            .bounds
            .iter()
            .copied()
            .filter(|clause| match clause {
                Clause::Predicate(predicate) => {
                    (predicate.ty == template
                        && matches!(predicate.bound, Bound::Trait(_) | Bound::Outlives(_)))
                        || matches!(predicate.bound, Bound::ProjectionEqual(_))
                }
                Clause::ForAll(_) => true,
            })
            .collect();
        Self {
            template,
            pending,
            index: Index::default(),
            binders: HashMap::new(),
            steps: contract.bounds.len(),
        }
    }
    fn charge(&mut self, work: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(work);
        if self.steps.saturating_add(self.pending.len()) > 1_000_000 {
            return Err("associated result rule work budget exceeded");
        }
        Ok(())
    }
    fn complete(mut self, facts: &mut Declarations) -> Result<(Index, usize), String> {
        let mut seen = HashSet::new();
        while let Some(clause) = self.pending.pop() {
            self.charge(1)?;
            if seen.insert(clause) {
                self.insert(facts, clause)?;
            }
        }
        Ok((self.index, self.steps))
    }
    fn insert(&mut self, facts: &mut Declarations, clause: Clause) -> Result<(), String> {
        let (predicate, count) = match clause {
            Clause::Predicate(value) => (value, None),
            Clause::ForAll(id) => {
                let value = facts.types.quantified(id).ok_or("invalid result clause")?;
                (value.predicate, Some(value.bound_regions))
            }
        };
        let (subject, work) = if count.is_some() {
            facts.types.shift_type(self.template, 1)?
        } else {
            (self.template, 0)
        };
        self.charge(work)?;
        if let Bound::ProjectionEqual(_) = predicate.bound {
            if let Some(crate::types::Kind::Projection(application)) =
                facts.types.kind(predicate.ty)
                && application.subject == subject
            {
                self.index
                    .equalities
                    .entry(application.item)
                    .or_default()
                    .push(clause);
            }
            return Ok(());
        }
        if predicate.ty != subject {
            return Ok(());
        }
        if let Bound::Outlives(region) = predicate.bound {
            // Quantified result requirements are not unconditional scalar promises.
            if count.is_none() {
                self.index.outlives.push(region);
            }
            return Ok(());
        }
        let Bound::Trait(application) = predicate.bound else {
            return Ok(());
        };
        let item = facts
            .types
            .trait_application(application)
            .ok_or("missing result rule application")?
            .item;
        self.index.candidates.entry(item).or_default().push(clause);
        let requirements = facts
            .generics
            .traits
            .as_mut()
            .ok_or("missing result trait requirements")?
            .requirements(&mut facts.types, predicate)?;
        for &implied in &requirements.implied {
            let implied = self.lift(&mut facts.types, count, implied)?;
            self.pending.push(implied);
            self.charge(0)?;
        }
        Ok(())
    }
    fn lift(
        &mut self,
        store: &mut crate::types::Store,
        count: Option<u32>,
        clause: Clause,
    ) -> Result<Clause, &'static str> {
        let Some(count) = count else {
            return Ok(clause);
        };
        let binder = match self.binders.entry(count) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(Quantifier::new(count)?)
            }
        };
        let before = binder.steps;
        let result = binder.clause(store, clause);
        let work = binder.steps.saturating_sub(before);
        self.charge(work)?;
        result
    }
}
