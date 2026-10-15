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
        while next < work.len() {
            if work.len() > 1_000_000 {
                return Err("supertrait assumption work budget exceeded");
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
            steps = steps.saturating_add(1 + requirements.implied.len());
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
        }
        if work.len() > initial {
            self.predicates
                .entry(owner)
                .or_default()
                .extend(work.drain(initial..));
        }
        Ok(())
    }
}
