//! The premises a structural census of trait callers reads as content.
use super::{Abilities, Clause, Environment, ParameterId, TypeId};

/// One premise of an environment.
#[derive(Clone, Copy)]
pub(crate) enum Premise {
    Parameter(ParameterId, Abilities),
    Subject(TypeId, Abilities),
    Sized(TypeId),
    Clause(Clause),
}

impl Environment {
    /// Visit every premise; `false` when an opened view or region equalities
    /// make the environment more than the visited premises.
    pub(crate) fn census_premises(&self, visit: &mut impl FnMut(Premise)) -> bool {
        if self.has_trait_opening()
            || self
                .regions
                .as_ref()
                .is_some_and(|regions| !regions.is_empty())
        {
            return false;
        }
        for (&id, &abilities) in self.parameters.entries() {
            visit(Premise::Parameter(id, abilities));
        }
        if let Some(assumptions) = &self.assumptions {
            for (&ty, &abilities) in &assumptions.predicates {
                visit(Premise::Subject(ty, abilities));
            }
            for &ty in &assumptions.sized {
                visit(Premise::Sized(ty));
            }
        }
        let builtins = self.builtins.iter().flat_map(|builtins| builtins.clauses());
        let traits = self.traits.iter().flat_map(|traits| traits.clauses());
        for clause in builtins.chain(traits).chain(self.projection_clauses()) {
            visit(Premise::Clause(clause));
        }
        true
    }
}
