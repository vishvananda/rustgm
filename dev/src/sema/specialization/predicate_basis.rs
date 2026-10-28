//! Minimal admission compares real elaborated clauses, not rendered predicate names.
use super::{header_conditions::Conditions, retention_work::Budget};
use crate::{
    sema::model::Declarations,
    support::{hash::HashSet, retained::Growth},
    types::{outlives::Pair, parameters::clauses::Clause},
};
#[derive(Default)]
pub(super) struct Basis {
    pub clauses: HashSet<Clause>,
    pub regions: HashSet<Pair>,
    pending: Vec<Clause>,
}
impl Basis {
    pub(super) fn conditions(
        &mut self,
        conditions: Conditions,
        budget: &Budget,
    ) -> Result<(), &'static str> {
        for clause in conditions.clauses {
            self.insert(clause, budget)?;
        }
        for pair in conditions.outlives {
            let growth = Growth::map::<Pair, ()>(
                self.regions.len(),
                self.regions.capacity(),
                usize::from(!self.regions.contains(&pair)),
            );
            budget.charge(1 + growth.work, growth.bytes)?;
            self.regions.insert(pair);
        }
        Ok(())
    }
    pub(super) fn insert(&mut self, clause: Clause, budget: &Budget) -> Result<(), &'static str> {
        budget.charge(1, 0)?;
        if self.clauses.contains(&clause) {
            return Ok(());
        }
        let mut growth = Growth::map::<Clause, ()>(self.clauses.len(), self.clauses.capacity(), 1);
        growth.include(Growth::vector::<Clause>(
            self.pending.len(),
            self.pending.capacity(),
            1,
        ));
        budget.charge(growth.work, growth.bytes)?;
        self.clauses.insert(clause);
        self.pending.push(clause);
        Ok(())
    }
    pub(super) fn elaborate(
        &mut self,
        facts: &mut Declarations,
        budget: &Budget,
    ) -> Result<(), &'static str> {
        while let Some(clause) = self.pending.pop() {
            budget.charge(1, 0)?;
            if matches!(clause, Clause::UnstableFeature(_)) {
                continue; // Retained in clauses; it has no supertrait type consequence.
            }
            let Clause::Predicate(predicate) = clause else {
                return Err(
                    "higher-ranked specialization predicate elaboration is not implemented",
                );
            };
            if predicate.bound.trait_parts().is_none() {
                continue;
            }
            let requirements = facts
                .generics
                .traits
                .as_deref_mut()
                .ok_or("specialization requires actual trait contracts")?
                .requirements_counted(&mut facts.types, predicate, &|steps, bytes| {
                    budget.charge(steps, bytes)
                })?;
            for &clause in requirements
                .required
                .iter()
                .chain(&requirements.implied)
                .chain(&requirements.results)
            {
                self.insert(clause, budget)?;
            }
            for &pair in &requirements.regions {
                let growth = Growth::map::<Pair, ()>(
                    self.regions.len(),
                    self.regions.capacity(),
                    usize::from(!self.regions.contains(&pair)),
                );
                budget.charge(1 + growth.work, growth.bytes)?;
                self.regions.insert(pair);
            }
        }
        Ok(())
    }
}
