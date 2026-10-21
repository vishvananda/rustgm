//! Solved bodies retain their finalization frontier while field declarations complete.
use super::{
    AdjustmentKind, Checker, Declarations, Dependency, Error, NodeId, NodeKey, Operation,
    RegionEquality, field_storage::Batch, hidden_results::HiddenResults,
};
use crate::{
    sema::traits::selection::Source,
    types::{GenericArgs, infer::Solved, parameters::predicates::Predicate},
};
use std::collections::HashMap;

pub(super) enum Finalization {
    Closures(HashMap<usize, Source>),
    Storage(Box<Publication>),
    Requirements(HiddenResults),
}
pub(super) struct Publication {
    hidden: HiddenResults,
    solved: Solved,
    predicates: Vec<Predicate>,
    arguments: Vec<GenericArgs>,
    evidence: HashMap<usize, Source>,
    storage: Option<Batch>,
}
impl Checker<'_> {
    pub(super) fn finish(&mut self, body: NodeId) -> Result<Option<Dependency>, Error> {
        if self.state.trait_goals.is_none() {
            for adjustment in &self.state.adjustments {
                if adjustment.kind == AdjustmentKind::Never {
                    self.state
                        .infer
                        .default_never(adjustment.expected)
                        .map_err(|e| self.error(body, e))?;
                }
            }
        }
        self.state
            .infer
            .default_defining_results(&self.facts.types)
            .map_err(|e| self.error(body, e))?;
        let evidence = match self.infer_trait_obligations(body)? {
            super::trait_goals::Progress::Ready(evidence) => evidence,
            super::trait_goals::Progress::Prepare(dependency) => return Ok(Some(dependency)),
        };
        self.finish_closures(evidence)
    }
    pub(super) fn finish_inference(
        &mut self,
        evidence: HashMap<usize, Source>,
    ) -> Result<Option<Dependency>, Error> {
        let body = self.state.body.node;
        self.publish_closure_progress()?;
        let (solved, predicates, arguments) = std::mem::take(&mut self.state.infer)
            .finish_with_arguments(
                &mut self.facts.types,
                self.state
                    .call_types
                    .obligations
                    .iter()
                    .map(|(_, predicate)| predicate),
                self.state
                    .call_types
                    .constants
                    .iter()
                    .map(|pending| pending.arguments.as_slice()),
            )
            .map_err(|e| self.error(body, e))?;
        self.publish_closure_body(&solved)?;
        let hidden = self.check_opaque_results(&solved)?;
        let storage = self.body_storage(&solved)?;
        self.advance_publication(Publication {
            hidden,
            solved,
            predicates,
            arguments,
            evidence,
            storage,
        })
    }
    pub(super) fn resume_finalization(
        &mut self,
        mut finalization: Finalization,
    ) -> Result<Option<Dependency>, Error> {
        if let Finalization::Storage(publication) = &mut finalization
            && let Some(storage) = &mut publication.storage
        {
            storage
                .resume(self.facts)
                .map_err(|e| self.error(self.state.body.node, e))?;
        }
        self.advance_finalization(finalization)
    }
    fn advance_finalization(
        &mut self,
        finalization: Finalization,
    ) -> Result<Option<Dependency>, Error> {
        match finalization {
            Finalization::Closures(evidence) => self.finish_closures(evidence),
            Finalization::Storage(publication) => self.advance_publication(*publication),
            Finalization::Requirements(hidden) => self.complete_requirements(hidden),
        }
    }
    fn advance_publication(
        &mut self,
        mut publication: Publication,
    ) -> Result<Option<Dependency>, Error> {
        let body = self.state.body.node;
        if let Some(storage) = &mut publication.storage
            && let Some(requirement) = self.advance_body_storage(storage)?
        {
            self.state.finalization = Some(Box::new(Finalization::Storage(Box::new(publication))));
            return Ok(Some(Dependency {
                requirement,
                source: self.key(body),
            }));
        }
        if let Some(storage) = publication.storage.take() {
            storage
                .publish(self.facts, self.state.body)
                .map_err(|e| self.error(body, e))?;
        }
        let hidden = self.publish_solved_body(publication)?;
        self.complete_requirements(hidden)
    }
    fn complete_requirements(
        &mut self,
        hidden: HiddenResults,
    ) -> Result<Option<Dependency>, Error> {
        if let Some(dependency) = self.advance_canonical_requirements()? {
            self.state.finalization = Some(Box::new(Finalization::Requirements(hidden)));
            return Ok(Some(dependency));
        }
        self.publish_hidden_results(hidden)?;
        if let Some(context) = self.state.canonical_trait_inference.take() {
            self.facts.body_work.canonical_steps += context.steps;
        }
        self.clear_body_inference();
        self.complete_body();
        Ok(None)
    }
    fn publish_solved_body(&mut self, publication: Publication) -> Result<HiddenResults, Error> {
        let body = self.state.body.node;
        let Publication {
            solved,
            predicates,
            arguments,
            evidence,
            hidden,
            ..
        } = publication;
        self.facts.body_work.inference_steps += solved.steps;
        self.validate_generic_uses(&solved, &predicates, &evidence)?;
        self.publish_constant_uses(arguments)?;
        self.publish_methods(&solved);
        self.publish_invocations(&solved);
        let body_key = self.key(body);
        self.publish_pattern_types(&solved)?;
        self.facts
            .region_equalities
            .extend(
                solved
                    .region_equalities
                    .iter()
                    .map(|(left, right)| RegionEquality {
                        body: body_key,
                        left: *left,
                        right: *right,
                    }),
            );
        self.publish_body_types(body, &solved)?;
        self.publish_constant_callables()?;
        self.publish_index_bounds(&solved)?;
        self.publish_adjustments(&solved, body_key)?;
        for (node, term) in &self.state.calls {
            let key = self.key(*node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid call node"))?
                .operation = Some(Operation::Call {
                signature: solved.types[term.0 as usize],
            });
        }
        for node in &self.state.annotation_checks {
            crate::types::bound_uses::check_pointer(&self.facts.types, self.ty(*node)?)
                .map_err(|e| self.error(*node, e))?;
        }
        self.state.annotation_checks.clear();
        self.state.annotation_children.clear();
        self.state.type_annotations.clear();
        self.validate_operations()?;
        self.validate_assignments()?;
        self.validate_values()?;
        self.publish_reference_alignment()?;
        self.finish_patterns()?;
        // Canonical nominal checks may allocate universal aliases after the main
        // inference graph has finished. Publish them under this body's identity.
        self.facts.region_equalities.extend(
            std::mem::take(&mut self.state.infer.region_equalities)
                .into_iter()
                .map(|(left, right)| RegionEquality {
                    body: body_key,
                    left,
                    right,
                }),
        );
        Ok(hidden)
    }
    fn clear_body_inference(&mut self) {
        for node in self.state.touched.drain(..) {
            self.state.terms.remove(NodeKey {
                file: self.state.file,
                node,
            });
            self.state.diverges.remove(node.0 as usize);
        }
        self.state.bindings.clear();
        self.state.checks.clear();
        self.state.negated.clear();
        self.state.adjustments.clear();
        self.state.reference_adjustments.clear();
        self.state.unsizing_adjustments.clear();
        self.state.pointer_adjustments.clear();
        self.state.callable_adjustments.clear();
        self.state.pointer_casts.clear();
        self.state.reference_expected.clear();
        self.state.address_dereferences.clear();
        self.state.raw_address_places.clear();
        self.state.unsizing_contexts.clear();
        self.state.reference_work.clear();
        self.state.reference_steps = 0;
        self.state.break_roots.clear();
        self.state.discarded_adjustments.clear();
        self.state.calls.clear();
        self.state.assignments.clear();
        self.state.breaks.clear();
    }
}
