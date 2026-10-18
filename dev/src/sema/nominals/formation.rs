//! Trait declaration inputs are checked without satisfying deferred local conditions.
use super::{Declarations, Proof};
use crate::{
    sema::traits::formation::Formation,
    types::parameters::{clauses::Clause, predicates::Bound},
};

impl Proof {
    pub fn form_trait(
        &mut self,
        facts: &mut Declarations,
        clause: Clause,
    ) -> Result<Formation, &'static str> {
        // Preserve ordinary subject normalization and its diagnostic before
        // demanding trait arguments. Quantified subjects use the lifted input.
        if let Clause::Predicate(predicate) = clause {
            let ty = self.normalize(facts, predicate.ty)?;
            if !self.check(facts, ty)?.well_formed {
                return Err("bound subject must be well-formed");
            }
        }
        let formation = facts
            .generics
            .traits
            .as_mut()
            .ok_or("missing trait contracts")?
            .formation(&mut facts.types, clause)?;
        let subject = if matches!(clause, Clause::ForAll(_)) {
            formation.subject()
        } else {
            None
        };
        for ty in subject.into_iter().chain(formation.types().iter().copied()) {
            if !self.check(facts, ty)?.well_formed {
                return Err("trait argument must be well-formed");
            }
        }
        for &requirement in formation.required() {
            let requirement = match requirement {
                Clause::Predicate(value) if matches!(value.bound, Bound::Outlives(_)) => {
                    continue; // Proved with the completed nominal lifetime contracts.
                }
                Clause::Predicate(value) => self.normalize_builtin(facts, value)?.into(),
                Clause::ForAll(_) => requirement,
            };
            if !self.prove_clause(facts, requirement)? {
                return Err("trait argument does not satisfy its declaration requirements");
            }
        }
        Ok(formation)
    }
}
