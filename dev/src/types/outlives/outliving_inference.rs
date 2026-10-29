//! Infer only a demanded nominal closure; cached dependencies remain immutable.
use super::{Bounds, Inference, Inferred, Requirement};
use crate::{
    support::{
        hash::HashMap,
        retained::{Growth, Payment},
    },
    types::{
        ItemId, Store,
        aggregate::Definitions,
        outlives::Pair,
        parameters::{clauses::Clause, predicates::Bound},
    },
};

impl Bounds {
    /// Actual indexed declaration inputs, never a caller's assumed environment.
    /// Projected fields and quantified outlives clauses retain a wider completion
    /// requirement until their source normalization and scratch are also charged.
    pub(crate) fn append_reached<C: IntoIterator<Item = Clause>>(
        &mut self,
        types: &mut Store,
        definitions: &Definitions,
        root: ItemId,
        explicit: &HashMap<ItemId, Vec<Pair>>,
        clauses: impl Fn(ItemId) -> C,
        pay: Payment<'_>,
    ) -> Result<(), &'static str> {
        pay(1, 0)?;
        if self.items.contains_key(&root) {
            return Ok(());
        }
        let mut inference = Inference::default();
        inference.scan.budget.1 = Some(pay);
        let mut pending = Vec::new();
        inference.scan.budget.push(&mut pending, root)?;
        while let Some(item) = pending.pop() {
            inference.scan.budget.charge()?;
            if inference.result.items.contains_key(&item) {
                continue;
            }
            if let Some(known) = self.items.get(&item) {
                self.seed_cached(&mut inference, item, known)?;
                continue;
            }
            let definition = definitions
                .get(item)
                .ok_or("demanded nominal fields are incomplete")?;
            inference.fields(types, definitions, definition)?;
            for use_ in &inference.scan.uses {
                inference.scan.budget.charge()?;
                inference.scan.budget.push(&mut pending, use_.item)?;
            }
            for &pair in explicit.get(&item).into_iter().flatten() {
                inference.scan.budget.charge()?;
                inference.scan.clear()?;
                inference.scan.record(pair)?;
                inference.enqueue(types, item)?;
            }
            for clause in clauses(item) {
                inference.declared(types, item, clause)?;
            }
        }
        let inferred = inference.solve(types)?;
        self.publish_reached(inferred, pay)
    }
    fn seed_cached(
        &self,
        inference: &mut Inference<'_>,
        item: ItemId,
        known: &Inferred,
    ) -> Result<(), &'static str> {
        if !self.projections(item).is_empty() || self.has_family_inputs() {
            return Err("cached projected nominal outlives contracts require completion");
        }
        inference.scan.budget.map(&mut inference.result.items, 1)?;
        inference.result.items.insert(item, Inferred::default());
        for &pair in &known.regions {
            inference.push(item, Requirement::Region(pair))?;
        }
        for &pair in &known.parameters {
            inference.push(item, Requirement::Parameter(pair))?;
        }
        Ok(())
    }
    fn publish_reached(&mut self, inferred: Self, pay: Payment<'_>) -> Result<(), &'static str> {
        let steps = self
            .steps
            .checked_add(inferred.steps)
            .filter(|steps| *steps <= super::super::LIMIT)
            .ok_or("type outlives work budget exceeded")?;
        pay(inferred.items.len(), 0)?;
        let additional = inferred
            .items
            .keys()
            .filter(|item| !self.items.contains_key(item))
            .count();
        let growth =
            Growth::map::<ItemId, Inferred>(self.items.len(), self.items.capacity(), additional);
        pay(growth.work, growth.bytes)?;
        self.items
            .try_reserve(additional)
            .map_err(|_| "outlives retained allocation failed")?;
        for (item, value) in inferred.items {
            self.items.entry(item).or_insert(value);
        }
        self.steps = steps;
        Ok(())
    }
}
impl Inference<'_> {
    fn declared(
        &mut self,
        types: &mut Store,
        item: ItemId,
        clause: Clause,
    ) -> Result<(), &'static str> {
        self.scan.budget.charge()?;
        if let Clause::ForAll(id) = clause {
            let predicate = types
                .quantified(id)
                .ok_or("invalid nominal outlives clause")?
                .predicate;
            if matches!(predicate.bound, Bound::Outlives(_)) {
                return Err("demanded quantified nominal outlives clauses require completion");
            }
            return Ok(());
        }
        let mut declarations = super::super::clauses::Declarations::default();
        if let Some(predicate) = declarations.outlives(types, clause, &mut self.scan.budget)?
            && let Bound::Outlives(shorter) = predicate.bound
        {
            if types.contains_projections(predicate.ty) != Some(false) {
                return Err("demanded projected nominal outlives clauses require completion");
            }
            self.scan.clear()?;
            self.scan.type_bound(types, predicate.ty, shorter, 0)?;
            self.enqueue(types, item)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/outliving_contracts.rs"]
mod tests;
