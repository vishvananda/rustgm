//! Conjunctive source-root formation requirements remain debts at every actual caller.
use crate::support::hash::HashSet;
use crate::{
    body::regions::Contract,
    types::{
        RegionId, Store,
        callable::Templates,
        outlives::{Bounds, Pair, Queries},
    },
};
use std::sync::Arc;

pub(super) struct Collector<'a> {
    types: &'a Store,
    bounds: &'a Bounds,
    callables: &'a Templates,
    queries: Option<Queries<'a>>,
    work: usize,
}
impl<'a> Collector<'a> {
    pub(super) const fn new(
        types: &'a Store,
        bounds: &'a Bounds,
        callables: &'a Templates,
    ) -> Self {
        Self {
            types,
            bounds,
            callables,
            queries: None,
            work: 0,
        }
    }
    pub(super) const fn steps(&self) -> usize {
        self.work
    }
    pub(super) fn source(
        &mut self,
        input: &Contract,
        universals: u32,
        existing: Option<&Arc<[Pair]>>,
    ) -> Result<Option<Arc<[Pair]>>, &'static str> {
        let Some(formation) = input
            .output
            .as_ref()
            .and_then(|link| link.formation.as_ref())
        else {
            return Ok(existing.cloned());
        };
        if input.key != crate::body::ir::BodyKey::Closure(formation.identity) {
            return Err("formation conditions belong to another source closure");
        }
        let mut combined = None::<Vec<Pair>>;
        let mut seen = None::<HashSet<Pair>>;
        for root in &formation.inputs {
            charge(&mut self.work, 1)?;
            let queries = self
                .queries
                .get_or_insert_with(|| Queries::new(self.types, self.bounds, self.callables));
            let before = queries.steps();
            let requirements = queries.symbolic_requirements(root.child)?;
            // These are assumption-independent conjunctive requirements. Keep
            // all parameter/projection/private duties in the original contract.
            charge(&mut self.work, requirements.scalars.regions.len())?;
            for &pair in requirements.scalars.regions {
                if !formal(pair.0, universals) || !formal(pair.1, universals) {
                    continue;
                }
                if seen.is_none() {
                    let count = existing.map_or(0, |pairs| pairs.len());
                    charge(&mut self.work, count)?;
                    pending(count)?;
                    seen = Some(
                        existing
                            .into_iter()
                            .flat_map(|pairs| pairs.iter().copied())
                            .collect(),
                    );
                }
                charge(&mut self.work, 1)?;
                let seen = seen.as_mut().ok_or("missing formation pair index")?;
                if seen.contains(&pair) {
                    continue;
                }
                pending(seen.len().saturating_add(1))?;
                if combined.is_none() {
                    charge(&mut self.work, existing.map_or(0, |pairs| pairs.len()))?;
                    combined = Some(existing.map_or_else(Vec::new, |pairs| pairs.to_vec()));
                }
                seen.insert(pair);
                combined
                    .as_mut()
                    .ok_or("missing formation pair buffer")?
                    .push(pair);
            }
            charge(&mut self.work, queries.steps().saturating_sub(before))?;
        }
        let Some(combined) = combined else {
            return Ok(existing.cloned());
        };
        charge(&mut self.work, combined.len())?;
        Ok(Some(combined.into()))
    }
}
fn charge(work: &mut usize, amount: usize) -> Result<(), &'static str> {
    *work = work.saturating_add(amount);
    if *work > 1_000_000 {
        return Err("closure formation condition work budget exceeded");
    }
    Ok(())
}
fn formal(region: RegionId, universals: u32) -> bool {
    region == RegionId::STATIC
        || region
            .bound_parts()
            .is_some_and(|(depth, slot)| depth == 0 && u32::from(slot) < universals)
}

// Conservatively reserve hash, geometric vector and final Arc payload slots
// before extending either temporary collection. This is logical accounting.
fn pending(entries: usize) -> Result<(), &'static str> {
    let bytes = crate::support::retained::new_map::<Pair, ()>(entries)
        .saturating_add(entries.saturating_mul(5).saturating_mul(size_of::<Pair>()))
        .saturating_add(
            size_of::<HashSet<Pair>>() + size_of::<Vec<Pair>>() + 2 * size_of::<usize>(),
        );
    if bytes > 64_000_000 {
        return Err("closure formation condition buffer exceeds storage budget");
    }
    Ok(())
}
#[cfg(test)]
#[path = "../../../tests/invariants/root_conditions.rs"]
mod tests;
