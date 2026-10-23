//! Immutable object contracts supply inference goals, never assumed evidence.
pub(in crate::sema) mod pending;
use super::graph::{self, Graph};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        Kind, RegionId, TypeId,
        parameters::{
            ParameterId,
            clauses::{Clause, Quantified},
            predicates::{Bound, Predicate},
        },
    },
};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../../../tests/invariants/contextual_demands.rs"]
mod tests;

#[derive(Clone)]
pub(in crate::sema) struct Template {
    pub parameter: ParameterId,
    pub clauses: Arc<[Clause]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Resolved(TypeId),
    Declared {
        principal: crate::types::traits::ApplicationId,
        bound_regions: u32,
    },
}
#[derive(Default)]
pub(super) struct Cache {
    entries: HashMap<Key, Template>,
    payload: usize,
    steps: usize,
}

pub(in crate::sema) fn get(
    facts: &mut Declarations,
    target: TypeId,
) -> Result<(Option<Template>, usize), &'static str> {
    let Some(Kind::Dynamic {
        shape,
        arguments,
        bound_regions,
        ..
    }) = facts.types.kind(target)
    else {
        return Err("object input contract requires an object");
    };
    // Only the outer storage lifetime is irrelevant to the principal's inputs.
    let key = facts.types.intern(Kind::Dynamic {
        shape,
        arguments,
        bound_regions,
        region: RegionId::STATIC,
    })?;
    if let Some(value) = cached(facts, Key::Resolved(key))? {
        return Ok((Some(value), 1));
    }
    let Some(application) = facts.types.dynamic_principal(key)? else {
        return Ok((None, 1));
    };
    let (value, steps) = prepare(facts, application, bound_regions)?;
    publish(facts, Key::Resolved(key), &value, steps)?;
    Ok((Some(value), steps))
}

fn cached(facts: &mut Declarations, key: Key) -> Result<Option<Template>, &'static str> {
    if let Some(cache) = facts
        .generics
        .traits
        .as_mut()
        .and_then(|c| c.object_bounds.demands.as_mut())
        && let Some(value) = cache.entries.get(&key)
    {
        cache.steps = cache.steps.saturating_add(1);
        if cache.steps.saturating_add(cache.entries.len()) > 1_000_000 {
            return Err("object input contract cache work budget exceeded");
        }
        return Ok(Some(value.clone()));
    }
    Ok(None)
}

fn prepare(
    facts: &mut Declarations,
    application: crate::types::traits::ApplicationId,
    bound_regions: u32,
) -> Result<(Template, usize), &'static str> {
    let item = facts
        .types
        .trait_application(application)
        .ok_or("invalid object principal")?
        .item;
    let subject = facts
        .generics
        .traits
        .as_ref()
        .and_then(|c| c.header(item))
        .ok_or("missing object input trait header")?
        .self_type;
    let Some(Kind::Parameter(parameter)) = facts.types.kind(subject) else {
        return Err("object input Self requires its declared parameter");
    };
    let root = facts.types.intern_quantified(Quantified {
        bound_regions,
        predicate: Predicate {
            ty: subject,
            bound: Bound::Trait(application),
        },
    })?;
    let mut graph = Graph::new(Clause::ForAll(root));
    let mut clauses = Vec::new();
    let mut seen = HashSet::default();
    let mut inspections = 0_usize;
    while let Some(visit) = graph.next(facts)? {
        for clause in std::iter::once(visit.clause).chain(visit.implied) {
            inspections = inspections.saturating_add(1);
            if inspections.saturating_add(graph.steps()) > 1_000_000 {
                return Err("object input contract work budget exceeded");
            }
            if callable_clause(facts, clause)? && seen.insert(clause) {
                clauses.push(clause);
            }
        }
    }
    let steps = graph.steps().saturating_add(inspections);
    let value = Template {
        parameter,
        clauses: clauses.into(),
    };
    Ok((value, steps))
}

fn callable_clause(facts: &Declarations, clause: Clause) -> Result<bool, &'static str> {
    let predicate = graph::quantified_value(&facts.types, clause)?;
    Ok(match predicate.bound {
        Bound::Trait(id) | Bound::ConstTrait(id) | Bound::MaybeConstTrait(id) => facts
            .types
            .trait_application(id)
            .is_some_and(|a| facts.associated.callable(DefId(a.item.0)).is_some()),
        Bound::ProjectionEqual(_) => matches!(facts.types.kind(predicate.ty),
            Some(Kind::Projection(a)) if Some(DefId(a.item.0)) == facts.associated.callable_output()),
        _ => false,
    })
}

fn publish(
    facts: &mut Declarations,
    key: Key,
    value: &Template,
    steps: usize,
) -> Result<(), &'static str> {
    let Some(contracts) = facts.generics.traits.as_mut() else {
        return Ok(());
    };
    if !contracts.object_bounds.enabled {
        return Ok(());
    }
    let cache = contracts
        .object_bounds
        .demands
        .get_or_insert_with(Default::default);
    let payload = cache
        .payload
        .saturating_add(value.clauses.len().saturating_mul(size_of::<Clause>()));
    let bytes = crate::support::retained::new_map::<Key, Template>(cache.entries.len() + 1)
        .saturating_add(payload);
    let work = cache.steps.saturating_add(steps);
    if bytes > 64 * 1024 * 1024 || work.saturating_add(cache.entries.len()) > 1_000_000 {
        return Err("object input contract cache budget exceeded");
    }
    cache.entries.insert(key, value.clone());
    cache.payload = payload;
    cache.steps = work;
    Ok(())
}
