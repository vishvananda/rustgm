//! Ready source sizing roots share the same actual body caller and selection DAG.
use super::{Caller, Checker, Error, NodeId, Reservation, selection};
use crate::types::parameters::{
    clauses::Clause,
    predicates::{Bound, Predicate},
};

impl Checker<'_> {
    pub(in crate::sema::bodies) fn prove_sizing_conjunction(
        &mut self,
        node: NodeId,
        requirements: &[Clause],
    ) -> Result<bool, Error> {
        let Some(predicates) = self.ready_sizing_roots(node, requirements)? else {
            return Ok(false);
        };
        let key = self.key(node);
        let assumptions = self.trait_assumptions(node)?.unwrap_or_default();
        let environment = self.trait_query_environment(node)?;
        let first_region = self.trait_region_start(node)?;
        if self.state.canonical_trait_proofs.is_none() {
            for &predicate in &predicates {
                if selection::Reuse::needed(self.facts, predicate)
                    .map_err(|e| self.error(node, e))?
                {
                    self.state.canonical_trait_proofs = Some(Box::default());
                    break;
                }
            }
        }
        let context = self
            .state
            .canonical_trait_inference
            .get_or_insert_with(Default::default);
        let result = selection::prove_conjunction(
            self.facts,
            context,
            &predicates,
            Caller {
                assumptions: &assumptions,
                environment: environment.as_ref(),
                first_region,
                retain_sized: false,
                placeholders: None,
            },
            self.availability,
            self.state.canonical_trait_proofs.as_deref_mut(),
        );
        let equalities = std::mem::take(&mut context.region_equalities);
        match result.map_err(|error| {
            self.error(
                node,
                match error {
                    selection::Failure::Invalid(message) => message,
                    selection::Failure::Pending(_) => {
                        "ready body sizing unexpectedly requires pending declarations".into()
                    }
                },
            )
        })? {
            selection::Conjunction::Unproven(index) => {
                return Err(self.canonical_trait_failure(
                    node,
                    predicates[index],
                    "user trait obligation requires a matching assumption or implementation",
                ));
            }
            selection::Conjunction::Proven {
                proof,
                sized,
                provisional_size,
                selected,
            } => {
                if provisional_size || !sized.is_empty() {
                    return Err(
                        self.error(node, "validated body sizing retained provisional storage")
                    );
                }
                Reservation::default()
                    .publish(proof, self.facts, self.state.body, key)
                    .map_err(|e| self.error(node, e))?;
                if !selected.is_empty() {
                    self.facts
                        .generics
                        .trait_selections
                        .entry(key)
                        .or_default()
                        .extend(selected);
                }
            }
        }
        self.facts
            .region_equalities
            .extend(equalities.into_iter().map(|(left, right)| {
                crate::sema::model::RegionEquality {
                    body: self.state.body,
                    left,
                    right,
                }
            }));
        Ok(true)
    }
    fn ready_sizing_roots(
        &mut self,
        node: NodeId,
        requirements: &[Clause],
    ) -> Result<Option<Vec<Predicate>>, Error> {
        if requirements.len() < 2
            || self.facts.source_bindings.is_none()
            || !self.facts.types.nominal_sizes_sealed()
            || !matches!(
                self.availability,
                crate::sema::traits::implementations::Availability::Validated
            )
            || self
                .state
                .canonical_requirements
                .as_ref()
                .is_some_and(|p| p.waiting())
        {
            return Ok(None);
        }
        self.state
            .canonical_trait_inference
            .get_or_insert_with(Default::default)
            .account_work(requirements.len())
            .map_err(|e| self.error(node, e))?;
        let mut predicates = Vec::new();
        predicates
            .try_reserve_exact(requirements.len())
            .map_err(|_| self.error(node, "body sizing roots allocation failed"))?;
        for clause in requirements {
            let Clause::Predicate(predicate) = clause else {
                return Ok(None);
            };
            if predicate.bound == Bound::Sized {
                continue;
            }
            if !crate::sema::compiler_items::sizing_predicate(self.facts, *predicate)
                .map_err(|e| self.error(node, e))?
                || predicate
                    .contains_opaques(&self.facts.types)
                    .map_err(|e| self.error(node, e))?
                || !self.canonical_requirement_ready(node, *predicate)?
            {
                return Ok(None);
            }
            predicates.push(self.alias_predicate(node, *predicate)?);
        }
        if predicates.len() < 2 {
            return Ok(None);
        }
        Ok(Some(predicates))
    }
}
