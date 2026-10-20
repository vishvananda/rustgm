//! One complete object application shares its inherited membership and equations.
pub(in crate::sema) mod graph;
#[cfg(test)]
#[path = "../../../tests/invariants/object_bound_index.rs"]
mod tests;
use super::assumptions::Assumptions;
use crate::{
    sema::model::Declarations,
    types::{
        ItemId, Kind, TypeId,
        parameters::{clauses::Clause, predicates::Bound},
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Default)]
pub(super) struct Cache {
    enabled: bool,
    storage: Option<Box<Storage>>,
}
#[derive(Default)]
struct Storage {
    entries: HashMap<TypeId, Arc<Index>>,
    steps: usize,
}
impl Cache {
    pub(super) fn enable(&mut self) {
        self.enabled = true;
    }
    pub(super) fn invalidate(&mut self) {
        *self = Self::default();
    }
}

pub(in crate::sema) struct Index {
    pub rules: Arc<Assumptions>,
    pub traits: Vec<ItemId>,
    pub equalities: HashMap<ItemId, Arc<[Clause]>>,
    pub applications: Vec<Clause>,
}

pub(in crate::sema) fn get(
    facts: &mut Declarations,
    ty: TypeId,
) -> Result<(Arc<Index>, usize), &'static str> {
    if let Some(index) = facts
        .generics
        .traits
        .as_ref()
        .and_then(|contracts| contracts.object_bounds.storage.as_ref())
        .and_then(|storage| storage.entries.get(&ty))
    {
        return Ok((Arc::clone(index), 1));
    }
    let (clauses, mut steps) = facts.types.dynamic_clauses(ty)?;
    let mut rules = Assumptions::default();
    let mut traits = HashSet::new();
    let mut applications = Vec::new();
    let mut equalities = HashMap::<ItemId, Vec<Clause>>::new();
    let mut seen = HashSet::new();
    for clause in clauses {
        if matches!(
            graph::quantified_value(&facts.types, clause)?.bound,
            Bound::Auto(_)
        ) {
            continue;
        }
        if record_equality(facts, clause, &mut equalities, &mut seen)? {
            continue;
        }
        let mut graph = graph::Graph::new(clause);
        while let Some(visit) = graph.next(facts)? {
            applications.push(visit.clause);
            rules
                .insert_clause(&facts.types, visit.clause)
                .map_err(|_| "invalid object membership clause")?;
            let Bound::Trait(application) = visit.predicate.bound else {
                return Err("object membership graph requires a trait");
            };
            traits.insert(
                facts
                    .types
                    .trait_application(application)
                    .ok_or("invalid object member application")?
                    .item,
            );
            for implied in visit.implied {
                record_equality(facts, implied, &mut equalities, &mut seen)?;
            }
        }
        steps = steps.saturating_add(graph.steps());
        if steps > 1_000_000 {
            return Err("object bound index work budget exceeded");
        }
    }
    let mut traits: Vec<_> = traits.into_iter().collect();
    traits.sort_unstable_by_key(|item| item.0);
    let index = Arc::new(Index {
        rules: Arc::new(rules),
        traits,
        applications,
        equalities: equalities
            .into_iter()
            .map(|(item, clauses)| (item, clauses.into()))
            .collect(),
    });
    let Some(contracts) = facts.generics.traits.as_mut() else {
        return Ok((index, steps));
    };
    let cache = &mut contracts.object_bounds;
    if cache.enabled {
        let storage = cache.storage.get_or_insert_with(Default::default);
        storage.steps = storage.steps.saturating_add(steps);
        if storage.steps.saturating_add(storage.entries.len()) > 1_000_000 {
            return Err("object bound cache work budget exceeded");
        }
        storage.entries.insert(ty, Arc::clone(&index));
    }
    Ok((index, steps))
}

fn record_equality(
    facts: &Declarations,
    clause: Clause,
    equalities: &mut HashMap<ItemId, Vec<Clause>>,
    seen: &mut HashSet<Clause>,
) -> Result<bool, &'static str> {
    let predicate = graph::quantified_value(&facts.types, clause)?;
    if !matches!(predicate.bound, Bound::ProjectionEqual(_)) {
        return Ok(false);
    }
    let Some(Kind::Projection(application)) = facts.types.kind(predicate.ty) else {
        return Err("object equation requires a projection");
    };
    if seen.insert(clause) {
        equalities.entry(application.item).or_default().push(clause);
    }
    Ok(true)
}
