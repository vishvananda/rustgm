//! Adjacent universal binders compose formulas without introducing proof permissions.
use super::{Clause, Quantified};
use crate::support::hash::HashMap;
use crate::types::{RegionId, Store, bound_uses::captures::Slots, substitute::Bound};

/// Bind clauses already written inside one surrounding universal binder.
/// Reuse this folder for one store and fixed outer arity across the whole batch.
pub struct Quantifier {
    count: u32,
    outer_slots: Slots,
    openings: HashMap<u32, Bound>,
    pub steps: usize,
}

impl Quantifier {
    /// # Errors
    /// Rejects counts outside the compact lifetime-slot representation.
    pub fn new(count: u32) -> Result<Self, &'static str> {
        if count > u32::from(u16::MAX) + 1 {
            return Err("too many composed universal lifetime slots");
        }
        Ok(Self {
            count,
            outer_slots: Slots::at(1),
            openings: HashMap::default(),
            steps: 0,
        })
    }

    /// Preserve an ordinary body or merge its own quantifier after the outer slots.
    /// Empty binders still remove one capture depth; nested callable locals stay local.
    /// This is structural substitution, never evidence that either premise holds.
    /// # Errors
    /// Rejects invalid identities, unbound outer slots, count overflow and exhausted work.
    pub fn clause(&mut self, store: &mut Store, value: Clause) -> Result<Clause, &'static str> {
        self.account(1)?;
        let (predicate, bound_regions) = match value {
            Clause::Predicate(predicate) => (predicate, self.count),
            Clause::ForAll(id) => {
                let inner = store.quantified(id).ok_or("invalid composed clause")?;
                let count = self
                    .count
                    .checked_add(inner.bound_regions)
                    .filter(|count| *count <= u32::from(u16::MAX) + 1)
                    .ok_or("too many composed universal lifetime slots")?;
                let before = self.outer_slots.steps;
                let required = self.outer_slots.predicate(store, inner.predicate);
                self.account(self.outer_slots.steps.saturating_sub(before))?;
                // Check before merging: an invalid parent slot could otherwise
                // collide with a valid appended slot of the inner quantifier.
                if required? > self.count {
                    return Err("composed clause references an unbound outer lifetime");
                }
                if !self.openings.contains_key(&inner.bound_regions) {
                    self.account(inner.bound_regions as usize)?;
                    let regions = (self.count..count)
                        .map(|slot| {
                            let slot = u16::try_from(slot)
                                .map_err(|_| "invalid composed lifetime slot")?;
                            RegionId::bound(0, slot)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    self.openings
                        .insert(inner.bound_regions, Bound::new(regions));
                }
                let opening = self
                    .openings
                    .get_mut(&inner.bound_regions)
                    .ok_or("missing composed binder opening")?;
                let before = opening.steps;
                let predicate = opening.predicate(store, inner.predicate);
                let work = opening.steps.saturating_sub(before);
                self.account(work)?;
                (predicate?, count)
            }
        };
        Ok(Clause::ForAll(store.intern_quantified(Quantified {
            bound_regions,
            predicate,
        })?))
    }

    fn account(&mut self, work: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(work);
        if self.steps > 1_000_000 {
            return Err("universal binder composition work budget exceeded");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/clause_composition.rs"]
mod tests;
