//! Close declaration rules once, then select candidates before mapping caller inputs.
mod consequences;
use crate::support::{
    hash::{HashMap, HashSet},
    retained::Growth,
};
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        ItemId,
        outlives::ResultId,
        parameters::{
            clauses::{Clause, Quantifier},
            predicates::Bound,
        },
    },
};
#[cfg(test)]
pub(in crate::sema) use consequences::controls as consequence_controls;
use std::sync::Arc;

#[derive(Default)]
pub(super) struct Index {
    candidates: HashMap<ItemId, Vec<Clause>>,
    equalities: HashMap<ItemId, Vec<Clause>>,
    pub(super) outlives: Vec<crate::types::RegionId>,
    pub(super) autos: u8,
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
    for_declaration(facts, ResultId::Associated(crate::types::ItemId(item.0)))
}

pub(super) fn for_declaration(
    facts: &mut Declarations,
    item: ResultId,
) -> Result<(Arc<Index>, usize), String> {
    let opened = begin::<false>(facts, item, &|_, _| Ok(()))?;
    finish::<false>(
        &mut facts.types,
        &mut facts.generics.traits,
        item,
        opened,
        &|_, _| Ok(()),
    )
}

pub(in crate::sema::traits) fn for_result_counted(
    facts: &mut Declarations,
    item: DefId,
) -> Result<(Arc<Index>, usize), String> {
    let payments =
        crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
            .map_err(|error| error.to_string())?;
    let charge = |steps, bytes| payments.charge(steps, bytes);
    let item = ResultId::Associated(ItemId(item.0));
    let opened = begin::<true>(facts, item, &charge)?;
    finish::<true>(
        &mut facts.types,
        &mut facts.generics.traits,
        item,
        opened,
        &charge,
    )
}

enum Opened {
    Cached(Arc<Index>),
    Close(Closure),
}
fn begin<const CHARGED: bool>(
    facts: &Declarations,
    item: ResultId,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<Opened, String> {
    if CHARGED {
        charge(1, 0)?;
    }
    if let Some(index) = facts
        .generics
        .traits
        .as_ref()
        .and_then(|contracts| contracts.result_rules.templates.get(&item))
    {
        return Ok(Opened::Cached(Arc::clone(index)));
    }
    let (template, bounds) = match item {
        ResultId::Associated(item) => {
            let contract = facts
                .associated
                .type_contract(DefId(item.0))
                .ok_or("missing associated result contract")?;
            (contract.projection, &contract.bounds)
        }
        ResultId::Opaque(id) => {
            let contract = facts
                .opaques
                .declarations
                .get(&id)
                .ok_or("missing opaque result contract")?;
            (contract.application, &contract.bounds)
        }
    };
    Ok(Opened::Close(Closure::new::<CHARGED>(
        template,
        bounds.iter().copied(),
        charge,
    )?))
}
fn finish<const CHARGED: bool>(
    store: &mut crate::types::Store,
    contracts: &mut Option<Box<super::super::Contracts>>,
    item: ResultId,
    opened: Opened,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(Arc<Index>, usize), String> {
    let closure = match opened {
        Opened::Cached(index) => return Ok((index, 1)),
        Opened::Close(closure) => closure,
    };
    let (index, steps) = closure.complete::<CHARGED>(store, contracts, charge)?;
    if CHARGED {
        charge(1, size_of::<Index>() + 2 * size_of::<usize>())?;
    }
    let index = Arc::new(index);
    if let Some(contracts) = contracts {
        let cache = &mut contracts.result_rules;
        if cache.enabled {
            cache.account(steps)?;
            if CHARGED {
                let growth = Growth::map::<ResultId, Arc<Index>>(
                    cache.templates.len(),
                    cache.templates.capacity(),
                    1,
                );
                charge(growth.work, growth.bytes)?;
            }
            cache.templates.insert(item, Arc::clone(&index));
        }
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
    fn new<const CHARGED: bool>(
        template: crate::types::TypeId,
        bounds: impl Iterator<Item = Clause>,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        let mut steps = 0;
        let pending = if CHARGED {
            let mut pending = Vec::new();
            for clause in bounds {
                charge(1, 0)?;
                steps += 1;
                if relevant(template, clause) {
                    let growth = Growth::vector::<Clause>(pending.len(), pending.capacity(), 1);
                    charge(growth.work, growth.bytes)?;
                    pending.push(clause);
                }
            }
            pending
        } else {
            bounds
                .inspect(|_| steps += 1)
                .filter(|&clause| relevant(template, clause))
                .collect()
        };
        Ok(Self {
            template,
            pending,
            index: Index::default(),
            binders: HashMap::default(),
            steps,
        })
    }
    fn charge(&mut self, work: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(work);
        if self.steps.saturating_add(self.pending.len()) > 1_000_000 {
            return Err("associated result rule work budget exceeded");
        }
        Ok(())
    }
    fn complete<const CHARGED: bool>(
        mut self,
        store: &mut crate::types::Store,
        contracts: &mut Option<Box<super::super::Contracts>>,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(Index, usize), String> {
        let mut seen = HashSet::default();
        while let Some(clause) = self.pending.pop() {
            self.charge(1)?;
            if CHARGED {
                let growth = Growth::map::<Clause, ()>(
                    seen.len(),
                    seen.capacity(),
                    usize::from(!seen.contains(&clause)),
                );
                charge(1 + growth.work, growth.bytes)?;
            }
            if seen.insert(clause) {
                self.insert::<CHARGED>(store, contracts, clause, charge)?;
            }
        }
        let normalized = consequences::normalize::<CHARGED>(store, &mut self.index, charge)?;
        self.charge(normalized)?;
        Ok((self.index, self.steps))
    }
    fn insert<const CHARGED: bool>(
        &mut self,
        store: &mut crate::types::Store,
        contracts: &mut Option<Box<super::super::Contracts>>,
        clause: Clause,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), String> {
        let (predicate, count) = match clause {
            Clause::UnstableFeature(_) => return Ok(()),
            Clause::Predicate(value) => (value, None),
            Clause::ForAll(id) => {
                let value = store.quantified(id).ok_or("invalid result clause")?;
                (value.predicate, Some(value.bound_regions))
            }
        };
        let (subject, work) = if count.is_some() {
            if CHARGED {
                store.shift_type_charged(self.template, 1, charge)?
            } else {
                store.shift_type(self.template, 1)?
            }
        } else {
            (self.template, 0)
        };
        self.charge(work)?;
        if let Bound::ProjectionEqual(_) = predicate.bound {
            if let Some(crate::types::Kind::Projection(application)) = store.kind(predicate.ty)
                && application.subject == subject
            {
                index_clause::<CHARGED>(
                    &mut self.index.equalities,
                    application.item,
                    clause,
                    charge,
                )?;
            }
            return Ok(());
        }
        if predicate.ty != subject {
            return Ok(());
        }
        if let Bound::Auto(marker) = predicate.bound {
            self.index.autos |= 1 << marker as u8;
            return Ok(());
        }
        if let Bound::Outlives(region) = predicate.bound {
            // The exact subject comparison above proves that the result itself
            // is independent of this quantifier. Only binder-independent targets
            // become scalar promises; pinned Rust does not expose a quantified
            // supertrait lifetime as an unconditional static capture promise.
            let region = if count.is_some() {
                match region.bound_parts() {
                    Some((0, _)) => return Ok(()),
                    Some((depth, index)) => crate::types::RegionId::bound(depth - 1, index)?,
                    None => region,
                }
            } else {
                region
            };
            if CHARGED {
                let growth = Growth::vector::<crate::types::RegionId>(
                    self.index.outlives.len(),
                    self.index.outlives.capacity(),
                    1,
                );
                charge(growth.work, growth.bytes)?;
            }
            self.index.outlives.push(region);
            return Ok(());
        }
        let Some((application, _)) = predicate.bound.trait_parts() else {
            return Ok(());
        };
        let item = store
            .trait_application(application)
            .ok_or("missing result rule application")?
            .item;
        index_clause::<CHARGED>(&mut self.index.candidates, item, clause, charge)?;
        let contracts = contracts
            .as_mut()
            .ok_or("missing result trait requirements")?;
        let requirements = if CHARGED {
            contracts.requirements_counted(store, predicate, charge)?
        } else {
            contracts.requirements(store, predicate)?
        };
        for &implied in &requirements.implied {
            let implied = self.lift::<CHARGED>(store, count, implied, charge)?;
            if CHARGED {
                let growth =
                    Growth::vector::<Clause>(self.pending.len(), self.pending.capacity(), 1);
                charge(1 + growth.work, growth.bytes)?;
            }
            self.pending.push(implied);
            self.charge(0)?;
        }
        Ok(())
    }
    fn lift<const CHARGED: bool>(
        &mut self,
        store: &mut crate::types::Store,
        count: Option<u32>,
        clause: Clause,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Clause, &'static str> {
        let Some(count) = count else {
            return Ok(clause);
        };
        if CHARGED {
            let growth = Growth::map::<u32, Quantifier>(
                self.binders.len(),
                self.binders.capacity(),
                usize::from(!self.binders.contains_key(&count)),
            );
            charge(growth.work, growth.bytes)?;
        }
        let binder = match self.binders.entry(count) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(Quantifier::new(count)?)
            }
        };
        let before = binder.steps;
        let result = if CHARGED {
            binder.clause_charged(store, clause, charge)
        } else {
            binder.clause(store, clause)
        };
        let work = binder.steps.saturating_sub(before);
        self.charge(work)?;
        result
    }
}

fn index_clause<const CHARGED: bool>(
    index: &mut HashMap<ItemId, Vec<Clause>>,
    item: ItemId,
    clause: Clause,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    if CHARGED {
        let prior = index.get(&item);
        let mut growth = Growth::map::<ItemId, Vec<Clause>>(
            index.len(),
            index.capacity(),
            usize::from(prior.is_none()),
        );
        growth.include(Growth::vector::<Clause>(
            prior.map_or(0, Vec::len),
            prior.map_or(0, Vec::capacity),
            1,
        ));
        charge(growth.work, growth.bytes)?;
    }
    index.entry(item).or_default().push(clause);
    Ok(())
}

fn relevant(template: crate::types::TypeId, clause: Clause) -> bool {
    match clause {
        Clause::UnstableFeature(_) => false,
        Clause::Predicate(predicate) => {
            (predicate.ty == template
                && matches!(
                    predicate.bound,
                    Bound::Trait(_)
                        | Bound::ConstTrait(_)
                        | Bound::MaybeConstTrait(_)
                        | Bound::Outlives(_)
                        | Bound::Auto(_)
                ))
                || matches!(predicate.bound, Bound::ProjectionEqual(_))
        }
        Clause::ForAll(_) => true,
    }
}
