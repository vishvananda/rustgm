//! Resolve call goals before inference completion, retaining actual assumption evidence.
mod inference;
use super::{Checker, Error, NodeId};
use crate::sema::traits::{
    assumptions::Assumptions,
    obligations::Reservation,
    selection::{self, Caller, Evidence, Selection, Source},
};
pub(super) use inference::{Pending, Progress};
use std::sync::Arc;

impl Checker<'_> {
    pub(super) fn prove_canonical_trait(
        &mut self,
        node: NodeId,
        predicate: crate::types::parameters::predicates::Predicate,
    ) -> Result<(), Error> {
        let predicate = self.alias_predicate(node, predicate)?;
        let assumptions = self.trait_assumptions(node)?.unwrap_or_default();
        let first_region = self.trait_region_start(node)?;
        let key = self.key(node);
        let context = self
            .state
            .canonical_trait_inference
            .get_or_insert_with(Default::default);
        let mut substitution =
            crate::types::infer::Instantiation::new(context, &self.facts.types, &[]).map_err(
                |e| crate::sema::model::Declarations::error(self.report, key, e.to_string()),
            )?;
        let goal = context
            .instantiate_predicate(&mut self.facts.types, predicate, &mut substitution)
            .map_err(|e| {
                crate::sema::model::Declarations::error(self.report, key, e.to_string())
            })?;
        let evidence = selection::select_available(
            self.facts,
            context,
            &goal,
            Caller {
                assumptions: &assumptions,
                environment: self.state.method_environment.as_ref(),
                first_region,
                retain_sized: false,
            },
            self.availability,
        );
        let equalities = std::mem::take(&mut context.region_equalities);
        let Selection::Proven(evidence) = evidence.map_err(|e| self.error(node, e))? else {
            return Err(self.error(
                node,
                "user trait obligation requires a matching assumption or implementation",
            ));
        };
        self.retain_trait_evidence(node, evidence)?;
        self.facts
            .region_equalities
            .extend(equalities.into_iter().map(|(left, right)| {
                crate::sema::model::RegionEquality {
                    body: self.state.body,
                    left,
                    right,
                }
            }));
        Ok(())
    }

    pub(super) fn trait_region_start(&self, node: NodeId) -> Result<u32, Error> {
        u32::try_from(self.facts.body_regions.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .filter(|n| *n < 0x8000_0000)
            .ok_or_else(|| self.error(node, "trait region identity space exhausted"))
    }

    fn retain_trait_evidence(&mut self, node: NodeId, evidence: Evidence) -> Result<Source, Error> {
        self.retain_reserved_trait_evidence(node, evidence, Reservation::default())
    }

    fn retain_reserved_trait_evidence(
        &mut self,
        node: NodeId,
        evidence: Evidence,
        reservation: Reservation,
    ) -> Result<Source, Error> {
        let source = self.key(node);
        let selected = evidence.source;
        reservation
            .publish(evidence.proof, self.facts, self.state.body, source)
            .map_err(|e| self.error(node, e))?;
        if let Source::Implementation { owner, arguments } = selected {
            self.facts
                .generics
                .trait_selections
                .entry(source)
                .or_default()
                .push(crate::sema::generics::ImplementationSelection { owner, arguments });
        }
        Ok(selected)
    }

    pub(super) fn trait_assumptions(
        &mut self,
        node: NodeId,
    ) -> Result<Option<Arc<Assumptions>>, Error> {
        if let Some(assumptions) = &self.state.trait_environment {
            return Ok(Some(Arc::clone(assumptions)));
        }
        let clauses: Vec<_> = self
            .state
            .method_environment
            .iter()
            .flat_map(|environment| environment.trait_clauses())
            .collect();
        if clauses.is_empty() {
            return Ok(None);
        }
        let mut assumptions = Assumptions::default();
        for clause in clauses {
            let clause = if clause
                .escaping_binders(&self.facts.types)
                .map_err(|e| self.error(node, e))?
                != 0
            {
                self.alias_clause(node, clause)?
            } else {
                clause
            };
            assumptions
                .insert_clause(&self.facts.types, clause)
                .map_err(|e| self.error(node, e))?;
        }
        let assumptions = Arc::new(assumptions);
        self.state.trait_environment = Some(Arc::clone(&assumptions));
        Ok(Some(assumptions))
    }
}
