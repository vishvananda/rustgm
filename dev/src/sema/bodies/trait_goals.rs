//! Resolve call goals before inference completion, retaining actual assumption evidence.
use super::{Checker, Error, NodeId};
use crate::sema::traits::{
    assumptions::Assumptions,
    selection::{self, Caller, Evidence, Selection, Source},
};
use std::collections::HashMap;
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
                environment: self.method_environment.as_ref(),
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
                    body: self.body,
                    left,
                    right,
                }
            }));
        Ok(())
    }

    pub(super) fn infer_trait_obligations(
        &mut self,
        body: NodeId,
    ) -> Result<HashMap<usize, Source>, Error> {
        let mut pending: Vec<_> = self
            .call_types
            .obligations
            .iter()
            .enumerate()
            .filter_map(|(index, (_, predicate))| predicate.trait_arguments().map(|_| index))
            .collect();
        let mut evidence = HashMap::new();
        if pending.is_empty() && !self.projections.active() {
            return Ok(evidence);
        }
        let assumptions = self.trait_assumptions(body)?.unwrap_or_default();
        while !pending.is_empty() || self.projections.active() {
            let progress = self.advance_projections(body)?;
            let before = pending.len();
            let mut next = Vec::new();
            for index in pending {
                let first_region = self.trait_region_start(body)?;
                let (node, predicate) = &self.call_types.obligations[index];
                let node = *node;
                let caller = Caller {
                    assumptions: &assumptions,
                    environment: self.method_environment.as_ref(),
                    first_region,
                    retain_sized: false,
                };
                let selected = if self.call_types.assumed_obligations.contains(&index) {
                    selection::select_assumption(
                        &mut self.facts.types,
                        &mut self.infer,
                        predicate,
                        &caller,
                    )
                    .map_err(|error| error.to_string())
                } else {
                    selection::select_available(
                        self.facts,
                        &mut self.infer,
                        predicate,
                        caller,
                        self.availability,
                    )
                }
                .map_err(|e| self.error(node, e))?;
                if let Selection::Proven(proof) = selected {
                    evidence.insert(index, self.retain_trait_evidence(node, proof)?);
                } else {
                    next.push(index);
                }
            }
            if next.len() == before && !progress && (!next.is_empty() || self.projections.active())
            {
                if self
                    .infer
                    .default_numbers()
                    .map_err(|e| self.error(body, e))?
                {
                    pending = next;
                    continue;
                }
                let Some(&index) = next.first() else {
                    return Err(self.error(body, "associated projection inputs remain unresolved"));
                };
                let node = self.call_types.obligations[index].0;
                return Err(self.error(node,"trait obligation requires an unambiguous matching assumption or implementation"));
            }
            pending = next;
        }
        Ok(evidence)
    }

    fn trait_region_start(&self, node: NodeId) -> Result<u32, Error> {
        u32::try_from(self.facts.body_regions.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .filter(|n| *n < 0x8000_0000)
            .ok_or_else(|| self.error(node, "trait region identity space exhausted"))
    }

    fn retain_trait_evidence(&mut self, node: NodeId, evidence: Evidence) -> Result<Source, Error> {
        let source = self.key(node);
        let selected = evidence.source;
        let mut proof = crate::sema::traits::obligations::Obligations::new(evidence.regions.start);
        proof.append(evidence).map_err(|e| self.error(node, e))?;
        proof
            .retain_body(self.facts, self.body, source)
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
        if let Some(assumptions) = &self.trait_environment {
            return Ok(Some(Arc::clone(assumptions)));
        }
        let predicates: Vec<_> = self
            .method_environment
            .iter()
            .flat_map(|environment| environment.trait_predicates())
            .copied()
            .collect();
        if predicates.is_empty() {
            return Ok(None);
        }
        let mut assumptions = Assumptions::default();
        for predicate in predicates {
            let predicate = if predicate
                .escaping_binders(&self.facts.types)
                .map_err(|e| self.error(node, e))?
                != 0
            {
                self.alias_predicate(node, predicate)?
            } else {
                predicate
            };
            assumptions
                .insert(&self.facts.types, predicate)
                .map_err(|e| self.error(node, e))?;
        }
        let assumptions = Arc::new(assumptions);
        self.trait_environment = Some(Arc::clone(&assumptions));
        Ok(Some(assumptions))
    }
}
