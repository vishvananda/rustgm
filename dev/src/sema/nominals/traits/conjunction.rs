//! Full canonical nominal contracts share the existing selection proof DAG.
use super::{Declarations, State, selection};
use crate::types::parameters::{
    clauses::Clause,
    predicates::{Bound, Predicate},
};

#[derive(Clone, Copy)]
pub(in crate::sema::nominals) enum Completion {
    Proven,
    Unproven(Predicate),
}

pub(in crate::sema::nominals) fn supported(clause: &Clause) -> bool {
    matches!(clause, Clause::Predicate(p) if p.bound.trait_parts().is_some() || p.bound == Bound::Sized)
}

impl State {
    /// Unsupported clause kinds keep their existing dedicated producers. No
    /// partially selected conjunction is published as a satisfied requirement.
    pub(in crate::sema::nominals) fn prove_conjunction(
        &mut self,
        facts: &mut Declarations,
        requirements: &[Clause],
    ) -> Result<Option<Completion>, &'static str> {
        let Some(fold) = &mut self.fold else {
            return Ok(None);
        };
        self.context
            .account_work(requirements.len())
            .map_err(|_| "declaration conjunction input budget exceeded")?;
        let mut predicates = Vec::new();
        predicates
            .try_reserve_exact(requirements.len())
            .map_err(|_| "declaration conjunction input allocation failed")?;
        for &clause in requirements {
            let Clause::Predicate(predicate) = clause else {
                return Ok(None);
            };
            if !supported(&clause) {
                return Ok(None);
            }
            // Builtin Sized retains its original shape producer in Proof::check.
            // Only the actual canonical trait roots share this selection query.
            if predicate.bound != Bound::Sized {
                predicates.push(predicate);
            }
        }
        if predicates.len() < 2 {
            return Ok(None);
        }
        let before = fold.steps;
        for predicate in &mut predicates {
            *predicate = fold.predicate(&mut facts.types, *predicate)?;
        }
        self.context
            .account_work(fold.steps.saturating_sub(before))
            .map_err(|_| "declaration conjunction input budget exceeded")?;
        match selection::prove_conjunction(
            facts,
            &mut self.context,
            &predicates,
            selection::Caller {
                assumptions: &self.assumptions,
                environment: self.environment.as_ref(),
                first_region: self.obligations.regions.end,
                retain_sized: self.retain_sized,
                nesting: self.nesting.nested(),
                placeholders: self.placeholders.as_ref(),
            },
            self.availability,
            None,
        )
        .map_err(|error| {
            crate::sema::census::fold_cause(
                &error,
                "canonical trait selection work exhausted",
                "invalid canonical trait selection",
            )
        })? {
            selection::Conjunction::Proven {
                proof,
                sized,
                provisional_size,
                selected: _,
            } => {
                self.obligations.extend(proof)?;
                self.sized.extend(sized);
                self.provisional_size |= provisional_size;
                Ok(Some(Completion::Proven))
            }
            selection::Conjunction::Unproven(index) => {
                let predicate = requirements
                    .iter()
                    .filter_map(|clause| match clause {
                        Clause::Predicate(p) if p.bound.trait_parts().is_some() => Some(*p),
                        _ => None,
                    })
                    .nth(index)
                    .ok_or("canonical conjunction changed its actual root coverage")?;
                Ok(Some(Completion::Unproven(predicate)))
            }
        }
    }
}
