//! Describe a premise's outer conditions without publishing its local witnesses.
use super::{Error, Graph, HashSet, Kind, Region, Solver};
#[cfg(test)]
#[path = "../../../tests/invariants/premise_region_models.rs"]
mod tests;

impl Graph {
    /// These are necessary conditions, never evidence that a goal was proved.
    /// The consumer needs independent authority for assuming the original premise.
    /// Eliminate existential paths with the same scoped propagation as `solve`.
    /// A condition on a younger universal cannot become an outer assumption.
    pub(crate) fn outer_conditions(&self) -> Result<Vec<(Region, Region)>, Error> {
        if self.verifications.is_some() {
            return Err(Error::Identity);
        }
        let mut conditions = HashSet::new();
        let mut solver = Solver::new(self);
        solver.propagate(|longer, shorter, source| {
            let outer = |region| {
                self.kind(region)
                    .map(|kind| matches!(kind, Kind::Static | Kind::Universal { universe: 0 }))
            };
            if !outer(longer)? || !outer(shorter)? {
                return Err(Error::Unsatisfied {
                    source,
                    longer,
                    shorter,
                });
            }
            conditions.insert((longer, shorter));
            Ok(())
        })?;
        let mut conditions: Vec<_> = conditions.into_iter().collect();
        conditions.sort_unstable_by_key(|(longer, shorter)| (longer.0, shorter.0));
        Ok(conditions)
    }
}
