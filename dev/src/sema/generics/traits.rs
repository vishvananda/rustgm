//! Expand only Self requirements into an owner's immutable assumption environment.
use super::{DefId, Generics, Predicate};
use crate::types::{Store, parameters::predicates::Bound};
use std::collections::HashSet;

impl Generics {
    pub(super) fn elaborate_trait_bounds(
        &mut self,
        store: &mut Store,
        owner: DefId,
    ) -> Result<(), &'static str> {
        if !self
            .predicates_in_scope(owner)
            .any(|p| matches!(p.value.bound, Bound::Trait(_)))
        {
            return Ok(());
        }
        let mut seen = HashSet::new();
        let mut work: Vec<_> = self
            .predicates_in_scope(owner)
            .filter(|p| seen.insert(p.value))
            .copied()
            .collect();
        let initial = work.len();
        let mut next = 0;
        let mut steps = 0_usize;
        let mut results: Vec<Predicate> = Vec::new();
        let mut ordinary = true;
        let mut ordinary_end = usize::MAX;
        while next < work.len() || !results.is_empty() {
            if work.len().saturating_add(results.len()) > 1_000_000 {
                return Err("supertrait assumption work budget exceeded");
            }
            if next == work.len() {
                // Finish the finite ordinary closure before processing result
                // promises. Their own result clauses are queried on demand.
                ordinary = false;
                ordinary_end = work.len();
                for predicate in results.drain(..) {
                    if seen.insert(predicate.value) {
                        work.push(predicate);
                    }
                }
                if next == work.len() {
                    break;
                }
            }
            let predicate = work[next];
            next += 1;
            if !matches!(predicate.value.bound, Bound::Trait(_)) {
                continue;
            }
            let requirements = self
                .traits
                .as_mut()
                .ok_or("missing trait contracts")?
                .requirements(store, predicate.value)?;
            steps =
                steps.saturating_add(1 + requirements.implied.len() + requirements.results.len());
            if steps > 1_000_000 {
                return Err("supertrait assumption work budget exceeded");
            }
            for &value in &requirements.implied {
                if seen.insert(value) {
                    work.push(Predicate {
                        source: predicate.source,
                        value,
                    });
                }
            }
            if ordinary {
                results.extend(requirements.results.iter().map(|&value| Predicate {
                    source: predicate.source,
                    value,
                }));
            }
        }
        if work.len() > initial {
            let predicates = self.predicates.entry(owner).or_default();
            self.declared_predicate_counts
                .entry(owner)
                .or_insert(predicates.len());
            // User-trait promises on opaque results are proved from their
            // declaration after parent normalization. Adding those applications
            // here would let a different opaque result block a deeper exact proof.
            // Keep the finite ordinary equality and builtin/lifetime implications.
            predicates.extend(
                work.into_iter()
                    .enumerate()
                    .skip(initial)
                    .filter(|(index, predicate)| {
                        *index < ordinary_end || !matches!(predicate.value.bound, Bound::Trait(_))
                    })
                    .map(|(_, predicate)| predicate),
            );
        }
        Ok(())
    }
}
