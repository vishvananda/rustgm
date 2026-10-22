//! Expand only Self requirements into an owner's immutable assumption environment.
use super::{DefId, Generics, Predicate};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{
    Store,
    parameters::{
        clauses::{Clause, Comparison, Quantifier},
        predicates::{Bound, Predicate as Input},
    },
};
use std::collections::hash_map::Entry;

impl Generics {
    /// Body lifetime formation consumes the declaration's already filtered inputs.
    pub(crate) fn trait_formation_inputs(
        &self,
        id: crate::types::parameters::clauses::QuantifiedId,
    ) -> Result<impl Iterator<Item = crate::types::TypeId> + '_, &'static str> {
        self.traits
            .as_ref()
            .ok_or("missing completed trait contracts")?
            .completed_formation_inputs(id)
    }

    pub(super) fn elaborate_trait_bounds(
        &mut self,
        store: &mut Store,
        owner: DefId,
    ) -> Result<(), &'static str> {
        let mut has_traits = false;
        for predicate in self.predicates_in_scope(owner) {
            if trait_input(store, predicate.value)?.is_some() {
                has_traits = true;
                break;
            }
        }
        if !has_traits {
            return Ok(());
        }
        let inputs = self.predicates_in_scope(owner).copied().collect();
        let (work, initial) = self.expanded_trait_bounds(store, inputs)?;
        if work.len() > initial {
            let predicates = self.predicates.entry(owner).or_default();
            self.declared_predicate_counts
                .entry(owner)
                .or_insert(predicates.len());
            predicates.extend(work.into_iter().skip(initial));
        }
        Ok(())
    }

    /// Expand an explicit assumption set without publishing permissions on an
    /// owner. Conformance must exclude a family's projected result promises.
    pub(in crate::sema) fn expanded_trait_bounds(
        &mut self,
        store: &mut Store,
        mut inputs: Vec<Predicate>,
    ) -> Result<(Vec<Predicate>, usize), &'static str> {
        let mut written = HashSet::default();
        inputs.retain(|p| written.insert(p.value));
        let mut seen = Consequences::new(store, &inputs)?;
        let mut work = inputs;
        let initial = work.len();
        let mut next = 0;
        let mut steps = 0_usize;
        let mut binders = HashMap::default();
        let mut results: Vec<Predicate> = Vec::new();
        let mut ordinary = true;
        let mut ordinary_end = usize::MAX;
        while next < work.len() || !results.is_empty() {
            if work.len().saturating_add(results.len()) > 1_000_000 {
                return Err("supertrait assumption work budget exceeded");
            }
            if next == work.len() {
                // Complete ordinary inheritance before demand-driven result promises.
                ordinary = false;
                ordinary_end = work.len();
                for predicate in results.drain(..) {
                    if seen.insert(store, predicate.value)? {
                        work.push(predicate);
                    }
                }
                if next == work.len() {
                    break;
                }
            }
            let predicate = work[next];
            next += 1;
            let Some((input, count)) = trait_input(store, predicate.value)? else {
                continue;
            };
            let requirements = self
                .traits
                .as_mut()
                .ok_or("missing trait contracts")?
                .requirements(store, input)?;
            steps =
                steps.saturating_add(1 + requirements.implied.len() + requirements.results.len());
            if steps > 1_000_000 {
                return Err("supertrait assumption work budget exceeded");
            }
            for &value in &requirements.implied {
                let value = lift(store, &mut binders, count, value, &mut steps)?;
                if seen.insert(store, value)? {
                    work.push(Predicate {
                        source: predicate.source,
                        value,
                    });
                }
            }
            if ordinary {
                for &value in &requirements.results {
                    results.push(Predicate {
                        source: predicate.source,
                        value: lift(store, &mut binders, count, value, &mut steps)?,
                    });
                }
            }
        }
        let mut retained = initial;
        if work.len() > initial {
            // Opaque result trait promises require their own parent normalization.
            // Preserve finite builtin/equality implications, without installing
            // those user-trait results as independent caller permissions.
            for index in initial..work.len() {
                let predicate = work[index];
                let quantified_result_lifetime = index >= ordinary_end
                    && matches!(predicate.value, Clause::ForAll(_))
                    && predicate.value.is_outlives(store)?;
                if !quantified_result_lifetime
                    && (index < ordinary_end || trait_input(store, predicate.value)?.is_none())
                {
                    work[retained] = predicate;
                    retained += 1;
                }
            }
        }
        work.truncate(retained);
        Ok((work, initial))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Identity {
    Ordinary(Input),
    Quantified(Comparison),
}

/// Written clauses retain their identities and declaration duties. Derived
/// consequences may share logically equivalent binders, avoiding a separate
/// inherited environment entry for every path length through a shared graph.
#[derive(Default)]
struct Consequences {
    seen: HashSet<Identity>,
    steps: usize,
}
impl Consequences {
    fn new(store: &mut Store, inputs: &[Predicate]) -> Result<Self, &'static str> {
        let mut result = Self::default();
        for input in inputs {
            result.insert(store, input.value)?;
        }
        Ok(result)
    }
    fn insert(&mut self, store: &mut Store, clause: Clause) -> Result<bool, &'static str> {
        let (key, work) = match clause {
            Clause::Predicate(value) => (Identity::Ordinary(value), 1),
            Clause::ForAll(id) => {
                let (key, work) = store.clause_comparison(id)?;
                (Identity::Quantified(key), work)
            }
        };
        self.steps = self.steps.saturating_add(work);
        if self.steps.saturating_add(self.seen.len()) > 1_000_000 {
            return Err("supertrait consequence comparison budget exceeded");
        }
        Ok(self.seen.insert(key))
    }
}

/// Inspect a formula only together with the quantifier its consequences must retain.
fn trait_input(
    store: &Store,
    clause: Clause,
) -> Result<Option<(Input, Option<u32>)>, &'static str> {
    let (predicate, count) = match clause {
        Clause::Predicate(predicate) => (predicate, None),
        Clause::ForAll(id) => {
            let value = store.quantified(id).ok_or("invalid supertrait premise")?;
            (value.predicate, Some(value.bound_regions))
        }
    };
    Ok(matches!(predicate.bound, Bound::Trait(_)).then_some((predicate, count)))
}

fn lift(
    store: &mut Store,
    binders: &mut HashMap<u32, Quantifier>,
    count: Option<u32>,
    clause: Clause,
    steps: &mut usize,
) -> Result<Clause, &'static str> {
    let Some(count) = count else {
        return Ok(clause);
    };
    let binder = match binders.entry(count) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => entry.insert(Quantifier::new(count)?),
    };
    let before = binder.steps;
    let result = binder.clause(store, clause);
    *steps = steps.saturating_add(binder.steps.saturating_sub(before));
    if *steps > 1_000_000 {
        return Err("supertrait assumption work budget exceeded");
    }
    result
}
