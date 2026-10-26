//! Adjacent universal binders compose formulas without introducing proof permissions.
use super::{Clause, Quantified};
use crate::support::{hash::HashMap, retained::Growth};
use crate::types::{GenericArgs, RegionId, Store, TypeId, arguments::Argument, bound_uses::captures::Slots, substitute::Bound};

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
        if count > u32::from(u16::MAX) + 1 { return Err("too many composed universal lifetime slots") }
        Ok(Self { count, outer_slots: Slots::at(1), openings: HashMap::default(), steps: 0 })
    }
    /// Preserve an ordinary body or merge its quantifier after the outer slots.
    /// Empty binders still remove one capture depth; callable locals stay local.
    /// This is structural substitution, never proof that a premise holds.
    /// # Errors
    /// Rejects invalid identities, unbound parent slots, overflow and exhausted work.
    pub fn clause(&mut self, store: &mut Store, value: Clause) -> Result<Clause, &'static str> {
        self.clause_observed::<false>(store, value, &|_, _| Ok(()))
    }
    pub(crate) fn clause_charged(&mut self, store: &mut Store, value: Clause, charge: &impl Fn(usize, usize) -> Result<(), &'static str>) -> Result<Clause, &'static str> {
        self.clause_observed::<true>(store, value, charge)
    }
    fn clause_observed<const CHARGED: bool>(&mut self, store: &mut Store, value: Clause, charge: &impl Fn(usize, usize) -> Result<(), &'static str>) -> Result<Clause, &'static str> {
        self.account_observed::<CHARGED>(1, charge)?;
        let (predicate, bound_regions) = match value {
            Clause::Predicate(predicate) => (predicate, self.count),
            Clause::ForAll(id) => {
                let inner = store.quantified(id).ok_or("invalid composed clause")?;
                let count = self.combined_count(inner.bound_regions)?;
                let before = self.outer_slots.steps;
                let required = if CHARGED { self.outer_slots.predicate_charged(store, inner.predicate, charge) } else { self.outer_slots.predicate(store, inner.predicate) };
                self.account_observed::<CHARGED>(self.outer_slots.steps.saturating_sub(before), charge)?;
                self.validate_outer(required?)?;
                let (result, work) = {
                    let opening = self.opening::<CHARGED>(inner.bound_regions, charge)?;
                    let before = opening.steps;
                    let result = if CHARGED { opening.predicate_charged(store, inner.predicate, charge) } else { opening.predicate(store, inner.predicate) };
                    (result, opening.steps.saturating_sub(before))
                };
                self.account_observed::<CHARGED>(work, charge)?;
                (result?, count)
            }
        };
        let value = Quantified { bound_regions, predicate };
        Ok(Clause::ForAll(if CHARGED { store.intern_quantified_charged(value, charge)? } else { store.intern_quantified(value)? }))
    }
    /// Merge real subject/argument roots below an inner binder with this parent.
    /// No predicate, trait or proof is fabricated to carry these inputs.
    pub(crate) fn inputs_charged(&mut self, store: &mut Store, subject: TypeId, arguments: GenericArgs, inner: u32, charge: &impl Fn(usize, usize) -> Result<(), &'static str>) -> Result<(TypeId, GenericArgs, u32), &'static str> {
        self.account_observed::<true>(1, charge)?;
        let count = self.combined_count(inner)?;
        let before = self.outer_slots.steps;
        let required = self.outer_slots.collect_charged(store, subject, charge)
            .and_then(|subject| self.outer_slots.arguments_charged(store, arguments, charge).map(|arguments| subject.max(arguments)));
        self.account_observed::<true>(self.outer_slots.steps.saturating_sub(before), charge)?;
        self.validate_outer(required?)?;
        let (result, work) = {
            let opening = self.opening::<true>(inner, charge)?;
            let before = opening.steps;
            let result = opening.apply_argument_charged(store, Argument::Type(subject), charge)
                .and_then(|subject| opening.arguments_charged(store, arguments, charge).map(|arguments| (subject, arguments)));
            (result, opening.steps.saturating_sub(before))
        };
        self.account_observed::<true>(work, charge)?;
        let (Argument::Type(subject), arguments) = result? else { return Err("composed subject changed argument kind") };
        Ok((subject, arguments, count))
    }
    fn combined_count(&self, inner: u32) -> Result<u32, &'static str> {
        self.count.checked_add(inner).filter(|count| *count <= u32::from(u16::MAX) + 1).ok_or("too many composed universal lifetime slots")
    }
    fn validate_outer(&self, required: u32) -> Result<(), &'static str> {
        if required > self.count { return Err("composed clause references an unbound outer lifetime") }
        Ok(())
    }
    fn opening<const CHARGED: bool>(&mut self, inner: u32, charge: &impl Fn(usize, usize) -> Result<(), &'static str>) -> Result<&mut Bound, &'static str> {
        let count = self.combined_count(inner)?;
        if !self.openings.contains_key(&inner) {
            self.account_observed::<CHARGED>(inner as usize, charge)?;
            if CHARGED {
                let growth = Growth::map::<u32, Bound>(self.openings.len(), self.openings.capacity(), 1);
                charge(growth.work, growth.bytes.saturating_add((inner as usize).saturating_mul(size_of::<RegionId>())))?;
            }
            let regions = (self.count..count).map(|slot| {
                let slot = u16::try_from(slot).map_err(|_| "invalid composed lifetime slot")?;
                RegionId::bound(0, slot)
            }).collect::<Result<Vec<_>, _>>()?;
            self.openings.insert(inner, Bound::new(regions));
        }
        self.openings.get_mut(&inner).ok_or("missing composed binder opening")
    }
    fn account_observed<const CHARGED: bool>(&mut self, work: usize, charge: &impl Fn(usize, usize) -> Result<(), &'static str>) -> Result<(), &'static str> {
        let own = self.account(work);
        let shared = if CHARGED { charge(work, 0) } else { Ok(()) };
        own?;
        shared
    }
    fn account(&mut self, work: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(work);
        if self.steps > 1_000_000 { return Err("universal binder composition work budget exceeded") }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/clause_composition.rs"]
mod tests;
